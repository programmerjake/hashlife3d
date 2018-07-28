#[macro_use]
mod instance_functions;
mod device;
mod error;
mod fence;
mod instance;
mod queue;
mod semaphore;
mod surface;
use self::device::*;
use self::error::*;
use self::fence::*;
use self::instance::*;
use self::instance_functions::*;
use self::queue::*;
use self::semaphore::*;
use self::surface::*;
use super::*;
use sdl;
use std::ffi::CStr;
use std::mem;
use std::ptr::*;
use std::sync::Arc;
use std::time::Duration;
use std::u32;

trait NullOrZero {
    fn null_or_zero() -> Self;
}

impl NullOrZero for u64 {
    fn null_or_zero() -> u64 {
        0
    }
}

impl<T> NullOrZero for *mut T {
    fn null_or_zero() -> *mut T {
        null_mut()
    }
}

impl<T> NullOrZero for *const T {
    fn null_or_zero() -> *const T {
        null()
    }
}

fn null_or_zero<T: NullOrZero>() -> T {
    NullOrZero::null_or_zero()
}

#[allow(dead_code)]
#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
mod api {
    include!(concat!(env!("OUT_DIR"), "/vulkan-bindings.rs"));
}

fn vk_make_version(major: u32, minor: u32, patch: u32) -> u32 {
    assert!(major < 0x1000);
    assert!(minor < 0x1000);
    assert!(patch < 0x4000);
    major << 22 | minor << 12 | patch
}

#[derive(Clone)]
pub struct VulkanDeviceReference {
    device: Arc<DeviceWrapper>,
}

pub struct VulkanPausedDevice {
    surface_state: SurfaceState,
}

pub struct VulkanDevice {
    device_reference: VulkanDeviceReference,
    surface_state: SurfaceState,
    queue: VulkanQueue,
    present_queue: api::VkQueue,
}

fn get_wait_timeout(duration: Duration) -> u64 {
    if duration > Duration::from_nanos(u64::MAX) {
        u64::MAX
    } else {
        1000_000_000 * duration.as_secs() + duration.subsec_nanos() as u64
    }
}

struct ShaderModuleWrapper {
    device: Arc<DeviceWrapper>,
    shader_module: api::VkShaderModule,
}

impl Drop for ShaderModuleWrapper {
    fn drop(&mut self) {
        unsafe {
            self.device.vkDestroyShaderModule.unwrap()(
                self.device.device,
                self.shader_module,
                null(),
            );
        }
    }
}

struct RenderPassWrapper {
    device: Arc<DeviceWrapper>,
    render_pass: api::VkRenderPass,
}

impl Drop for RenderPassWrapper {
    fn drop(&mut self) {
        unsafe {
            self.device.vkDestroyRenderPass.unwrap()(self.device.device, self.render_pass, null());
        }
    }
}

struct GraphicsPipelineWrapper {
    device: Arc<DeviceWrapper>,
    pipeline: api::VkPipeline,
    pipeline_layout: PipelineLayoutWrapper,
    vertex_shader: ShaderModuleWrapper,
    fragment_shader: ShaderModuleWrapper,
    render_pass: RenderPassWrapper,
}

impl Drop for GraphicsPipelineWrapper {
    fn drop(&mut self) {
        unsafe { self.device.vkDestroyPipeline.unwrap()(self.device.device, self.pipeline, null()) }
    }
}

struct DescriptorSetLayoutWrapper {
    device: Arc<DeviceWrapper>,
    descriptor_set_layout: api::VkDescriptorSetLayout,
}

impl Drop for DescriptorSetLayoutWrapper {
    fn drop(&mut self) {
        unsafe {
            self.device.vkDestroyDescriptorSetLayout.unwrap()(
                self.device.device,
                self.descriptor_set_layout,
                null(),
            )
        }
    }
}

struct PipelineLayoutWrapper {
    device: Arc<DeviceWrapper>,
    pipeline_layout: api::VkPipelineLayout,
    descriptor_set_layouts: Vec<DescriptorSetLayoutWrapper>,
}

impl Drop for PipelineLayoutWrapper {
    fn drop(&mut self) {
        unsafe {
            self.device.vkDestroyPipelineLayout.unwrap()(
                self.device.device,
                self.pipeline_layout,
                null(),
            )
        }
    }
}

enum ShaderSource {
    MainVertex,
    MainFragment,
}

impl VulkanDeviceReference {
    fn get_shader(&self, shader_source: ShaderSource) -> Result<ShaderModuleWrapper> {
        let shader_source = match shader_source {
            ShaderSource::MainVertex => {
                include_bytes!(concat!(env!("OUT_DIR"), "/vulkan_main.vert.spv")) as &[u8]
            }
            ShaderSource::MainFragment => {
                include_bytes!(concat!(env!("OUT_DIR"), "/vulkan_main.frag.spv")) as &[u8]
            }
        };
        assert_eq!(shader_source.len() % mem::size_of::<u32>(), 0);
        assert!(shader_source.len() != 0);
        let mut shader_source_buf: Vec<u32> = Vec::new(); // copy to new memory to ensure it's aligned properly
        shader_source_buf.resize(shader_source.len() / mem::size_of::<u32>(), 0);
        unsafe {
            copy_nonoverlapping(
                shader_source.as_ptr(),
                shader_source_buf.as_mut_ptr() as *mut u8,
                shader_source.len(),
            );
        }
        let mut shader_module = null_or_zero();
        match unsafe {
            self.device.vkCreateShaderModule.unwrap()(
                self.device.device,
                &api::VkShaderModuleCreateInfo {
                    sType: api::VK_STRUCTURE_TYPE_SHADER_MODULE_CREATE_INFO,
                    pNext: null(),
                    flags: 0,
                    codeSize: shader_source.len(),
                    pCode: shader_source_buf.as_ptr(),
                },
                null(),
                &mut shader_module,
            )
        } {
            api::VK_SUCCESS => Ok(ShaderModuleWrapper {
                device: self.device.clone(),
                shader_module: shader_module,
            }),
            result => Err(VulkanError::VulkanError(result)),
        }
    }
}

impl DeviceReference for VulkanDeviceReference {
    type Semaphore = VulkanSemaphore;
    type Fence = VulkanFence;
    type Error = VulkanError;
    fn create_fence(&self, initial_state: FenceState) -> Result<VulkanFence> {
        let mut fence = null_or_zero();
        match unsafe {
            self.device.vkCreateFence.unwrap()(
                self.device.device,
                &api::VkFenceCreateInfo {
                    sType: api::VK_STRUCTURE_TYPE_FENCE_CREATE_INFO,
                    pNext: null(),
                    flags: match initial_state {
                        FenceState::Unsignaled => 0,
                        FenceState::Signaled => api::VK_FENCE_CREATE_SIGNALED_BIT,
                    },
                },
                null(),
                &mut fence,
            )
        } {
            api::VK_SUCCESS => Ok(VulkanFence {
                fence: fence,
                device: self.device.clone(),
            }),
            result => Err(VulkanError::VulkanError(result)),
        }
    }
}

impl PausedDevice for VulkanPausedDevice {
    type Device = VulkanDevice;
    fn get_window(&self) -> &sdl::window::Window {
        &self.surface_state.surface.window
    }
}

const fragment_textures_binding: u32 = 0;
const fragment_textures_binding_descriptor_count: u32 = 8;

#[repr(C)]
#[repr(align(16))]
struct AlignedMat4([[f32; 4]; 4]);

impl From<math::Mat4> for AlignedMat4 {
    fn from(v: math::Mat4) -> AlignedMat4 {
        AlignedMat4(v.into())
    }
}

/// must match PushConstants in shaders/vulkan_main.vert
#[repr(C)]
struct PushConstants {
    transform: AlignedMat4,
}

impl VulkanDevice {
    fn get_shader(&self, shader_source: ShaderSource) -> Result<ShaderModuleWrapper> {
        self.get_device_ref().get_shader(shader_source)
    }
    fn create_render_pass(&self) -> Result<RenderPassWrapper> {
        let device = self.device_reference.device.clone();
        let mut render_pass = null_or_zero();
        const depth_attachement_index: u32 = 0;
        const color_attachement_index: u32 = 1;
        let attachments: [api::VkAttachmentDescription; 2] = unsafe { mem::uninitialized() };
        attachments[color_attachement_index as usize] = api::VkAttachmentDescription {
            flags: 0,
            format: self.surface_state.surface_format.format,
            samples: api::VK_SAMPLE_COUNT_1_BIT,
            loadOp: api::VK_ATTACHMENT_LOAD_OP_CLEAR,
            storeOp: api::VK_ATTACHMENT_STORE_OP_STORE,
            stencilLoadOp: api::VK_ATTACHMENT_LOAD_OP_DONT_CARE,
            stencilStoreOp: api::VK_ATTACHMENT_STORE_OP_DONT_CARE,
            initialLayout: api::VK_IMAGE_LAYOUT_UNDEFINED,
            finalLayout: api::VK_IMAGE_LAYOUT_PRESENT_SRC_KHR,
        };
        attachments[depth_attachement_index as usize] = api::VkAttachmentDescription {
            flags: 0,
            format: self.surface_state.depth_format,
            samples: api::VK_SAMPLE_COUNT_1_BIT,
            loadOp: api::VK_ATTACHMENT_LOAD_OP_CLEAR,
            storeOp: api::VK_ATTACHMENT_STORE_OP_DONT_CARE,
            stencilLoadOp: api::VK_ATTACHMENT_LOAD_OP_DONT_CARE,
            stencilStoreOp: api::VK_ATTACHMENT_STORE_OP_DONT_CARE,
            initialLayout: api::VK_IMAGE_LAYOUT_UNDEFINED,
            finalLayout: api::VK_IMAGE_LAYOUT_UNDEFINED,
        };
        let color_attachments = [api::VkAttachmentReference {
            attachment: color_attachement_index,
            layout: api::VK_IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL,
        }];
        let subpasses = [api::VkSubpassDescription {
            flags: 0,
            pipelineBindPoint: api::VK_PIPELINE_BIND_POINT_GRAPHICS,
            inputAttachmentCount: 0,
            pInputAttachments: null(),
            colorAttachmentCount: color_attachments.len() as u32,
            pColorAttachments: color_attachments.as_ptr(),
            pResolveAttachments: null(),
            pDepthStencilAttachment: &api::VkAttachmentReference {
                attachment: depth_attachement_index,
                layout: api::VK_IMAGE_LAYOUT_DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
            },
            preserveAttachmentCount: 0,
            pPreserveAttachments: null(),
        }];
        let dependencies = [];
        match unsafe {
            device.vkCreateRenderPass.unwrap()(
                device.device,
                &api::VkRenderPassCreateInfo {
                    sType: api::VK_STRUCTURE_TYPE_RENDER_PASS_CREATE_INFO,
                    pNext: null(),
                    flags: 0,
                    attachmentCount: attachments.len() as u32,
                    pAttachments: attachments.as_ptr(),
                    subpassCount: subpasses.len() as u32,
                    pSubpasses: subpasses.as_ptr(),
                    dependencyCount: dependencies.len() as u32,
                    pDependencies: dependencies.as_ptr(),
                },
                null(),
                &mut render_pass,
            )
        } {
            api::VK_SUCCESS => Ok(RenderPassWrapper {
                device: device,
                render_pass: render_pass,
            }),
            result => Err(VulkanError::VulkanError(result)),
        }
    }
    fn create_descriptor_set_layout(&self) -> Result<DescriptorSetLayoutWrapper> {
        let device = self.device_reference.device.clone();
        let bindings = [api::VkDescriptorSetLayoutBinding {
            binding: fragment_textures_binding,
            descriptorType: api::VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER,
            descriptorCount: fragment_textures_binding_descriptor_count,
            stageFlags: api::VK_SHADER_STAGE_FRAGMENT_BIT,
            pImmutableSamplers: null(),
        }];
        let mut descriptor_set_layout = null_or_zero();
        match unsafe {
            device.vkCreateDescriptorSetLayout.unwrap()(
                device.device,
                &api::VkDescriptorSetLayoutCreateInfo {
                    sType: api::VK_STRUCTURE_TYPE_DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
                    pNext: null(),
                    flags: 0,
                    bindingCount: bindings.len() as u32,
                    pBindings: bindings.as_ptr(),
                },
                null(),
                &mut descriptor_set_layout,
            )
        } {
            api::VK_SUCCESS => Ok(DescriptorSetLayoutWrapper {
                device: device,
                descriptor_set_layout: descriptor_set_layout,
            }),
            result => Err(VulkanError::VulkanError(result)),
        }
    }
    fn create_pipeline_layout(
        &self,
        descriptor_set_layouts: Vec<DescriptorSetLayoutWrapper>,
    ) -> Result<PipelineLayoutWrapper> {
        let device = self.device_reference.device.clone();
        let mut pipeline_layout = null_or_zero();
        let vk_descriptor_set_layouts: Vec<_> = descriptor_set_layouts
            .iter()
            .map(|v| v.descriptor_set_layout)
            .collect();
        let push_constant_ranges = [api::VkPushConstantRange {
            stageFlags: api::VK_SHADER_STAGE_VERTEX_BIT,
            offset: 0,
            size: mem::size_of::<PushConstants>() as u32,
        }];
        match unsafe {
            device.vkCreatePipelineLayout.unwrap()(
                device.device,
                &api::VkPipelineLayoutCreateInfo {
                    sType: api::VK_STRUCTURE_TYPE_PIPELINE_LAYOUT_CREATE_INFO,
                    pNext: null(),
                    flags: 0,
                    setLayoutCount: vk_descriptor_set_layouts.len() as u32,
                    pSetLayouts: vk_descriptor_set_layouts.as_ptr(),
                    pushConstantRangeCount: push_constant_ranges.len() as u32,
                    pPushConstantRanges: push_constant_ranges.as_ptr(),
                },
                null(),
                &mut pipeline_layout,
            )
        } {
            api::VK_SUCCESS => Ok(PipelineLayoutWrapper {
                device: device,
                pipeline_layout: pipeline_layout,
                descriptor_set_layouts: descriptor_set_layouts,
            }),
            result => Err(VulkanError::VulkanError(result)),
        }
    }
    fn create_graphics_pipeline(&self) -> Result<GraphicsPipelineWrapper> {
        let device = self.device_reference.device.clone();
        let vertex_shader = self.get_shader(ShaderSource::MainVertex)?;
        let fragment_shader = self.get_shader(ShaderSource::MainFragment)?;
        let pipeline_layout =
            self.create_pipeline_layout(vec![self.create_descriptor_set_layout()?])?;
        let render_pass = self.create_render_pass()?;
        let mut pipeline = null_or_zero();
        let shader_entry_name = CStr::from_bytes_with_nul(b"main\0").unwrap();
        let stages = [
            api::VkPipelineShaderStageCreateInfo {
                sType: api::VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO,
                pNext: null(),
                flags: 0,
                stage: api::VK_SHADER_STAGE_VERTEX_BIT,
                module: vertex_shader.shader_module,
                pName: shader_entry_name.as_ptr(),
                pSpecializationInfo: null(),
            },
            api::VkPipelineShaderStageCreateInfo {
                sType: api::VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO,
                pNext: null(),
                flags: 0,
                stage: api::VK_SHADER_STAGE_FRAGMENT_BIT,
                module: fragment_shader.shader_module,
                pName: shader_entry_name.as_ptr(),
                pSpecializationInfo: null(),
            },
        ];
        match unsafe {
            device.vkCreateGraphicsPipelines.unwrap()(
                device.device,
                null_or_zero(),
                1,
                &api::VkGraphicsPipelineCreateInfo {
                    sType: api::VK_STRUCTURE_TYPE_GRAPHICS_PIPELINE_CREATE_INFO,
                    pNext: null(),
                    flags: 0,
                    stageCount: stages.len() as u32,
                    pStages: stages.as_ptr(),
                    pVertexInputState: &api::VkPipelineVertexInputStateCreateInfo {},
                    pInputAssemblyState: &api::VkPipelineInputAssemblyStateCreateInfo {},
                    pTessellationState: null(),
                    pViewportState: &api::VkPipelineViewportStateCreateInfo {},
                    pRasterizationState: &api::VkPipelineRasterizationStateCreateInfo {},
                    pMultisampleState: &api::VkPipelineMultisampleStateCreateInfo {},
                    pDepthStencilState: &api::VkPipelineDepthStencilStateCreateInfo {},
                    pColorBlendState: &api::VkPipelineColorBlendStateCreateInfo {},
                    pDynamicState: &api::VkPipelineDynamicStateCreateInfo {},
                    layout: pipeline_layout.pipeline_layout,
                    renderPass: render_pass.render_pass,
                    subpass: 0,
                    basePipelineHandle: null_or_zero(),
                    basePipelineIndex: -1,
                },
                null(),
                &mut pipeline,
            )
        } {
            api::VK_SUCCESS => Ok(GraphicsPipelineWrapper {
                device: device,
                pipeline: pipeline,
                pipeline_layout: pipeline_layout,
                vertex_shader: vertex_shader,
                fragment_shader: fragment_shader,
                render_pass: render_pass,
            }),
            result => Err(VulkanError::VulkanError(result)),
        }
    }
}

impl Device for VulkanDevice {
    type Semaphore = VulkanSemaphore;
    type Fence = VulkanFence;
    type Error = VulkanError;
    type Reference = VulkanDeviceReference;
    type Queue = VulkanQueue;
    type PausedDevice = VulkanPausedDevice;
    fn pause(self) -> VulkanPausedDevice {
        VulkanPausedDevice {
            surface_state: self.surface_state,
        }
    }
    fn resume(paused_device: VulkanPausedDevice) -> Result<Self> {
        let SurfaceState {
            surface,
            physical_device,
            present_queue_index,
            render_queue_index,
            surface_format,
            depth_format,
        } = paused_device.surface_state;
        let device_queue_create_infos = [
            api::VkDeviceQueueCreateInfo {
                sType: api::VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO,
                pNext: null(),
                flags: 0,
                queueFamilyIndex: present_queue_index,
                queueCount: 1,
                pQueuePriorities: [0.0].as_ptr(),
            },
            api::VkDeviceQueueCreateInfo {
                sType: api::VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO,
                pNext: null(),
                flags: 0,
                queueFamilyIndex: render_queue_index,
                queueCount: 1,
                pQueuePriorities: [0.0].as_ptr(),
            },
        ];
        let device_queue_create_infos = if present_queue_index == render_queue_index {
            &device_queue_create_infos[0..1]
        } else {
            &device_queue_create_infos[0..2]
        };
        let device = unsafe {
            DeviceWrapper::new(
                surface.instance.clone(),
                physical_device,
                device_queue_create_infos,
                &[
                    CStr::from_bytes_with_nul(api::VK_KHR_SWAPCHAIN_EXTENSION_NAME)
                        .unwrap()
                        .as_ptr(),
                ],
                None,
            )
        }?;
        let mut present_queue = null_mut();
        let mut render_queue = null_mut();
        unsafe {
            device.vkGetDeviceQueue.unwrap()(
                device.device,
                present_queue_index,
                0,
                &mut present_queue,
            )
        };
        unsafe {
            device.vkGetDeviceQueue.unwrap()(
                device.device,
                render_queue_index,
                0,
                &mut render_queue,
            )
        };
        let render_queue = VulkanQueue {
            queue: render_queue,
        };
        return Ok(VulkanDevice {
            device_reference: VulkanDeviceReference {
                device: Arc::new(device),
            },
            surface_state: SurfaceState {
                surface: surface,
                physical_device: physical_device,
                present_queue_index: present_queue_index,
                render_queue_index: render_queue_index,
                surface_format: surface_format,
                depth_format: depth_format,
            },
            queue: render_queue,
            present_queue: present_queue,
        });
    }
    fn get_window(&self) -> &sdl::window::Window {
        &self.surface_state.surface.window
    }
    fn get_device_ref(&self) -> &VulkanDeviceReference {
        &self.device_reference
    }
    fn get_queue(&self) -> &VulkanQueue {
        &self.queue
    }
    fn wait_for_fences_with_timeout(
        &self,
        fences: &[&VulkanFence],
        wait_for_all: bool,
        timeout: Duration,
    ) -> Result<WaitResult> {
        let mut final_fences = Vec::with_capacity(fences.len());
        for fence in fences {
            final_fences.push(fence.get());
        }
        assert_eq!(final_fences.len() as u32 as usize, final_fences.len());
        unsafe {
            match self.device_reference.device.vkWaitForFences.unwrap()(
                self.device_reference.device.device,
                final_fences.len() as u32,
                final_fences.as_ptr(),
                wait_for_all as api::VkBool32,
                get_wait_timeout(timeout),
            ) {
                api::VK_SUCCESS => Ok(WaitResult::Success),
                api::VK_TIMEOUT => Ok(WaitResult::Timeout),
                result => Err(VulkanError::VulkanError(result)),
            }
        }
    }
}

pub struct VulkanDeviceFactory<'a>(&'a sdl::event::EventSource);

impl<'a> VulkanDeviceFactory<'a> {
    pub fn new(event_source: &'a sdl::event::EventSource) -> Self {
        VulkanDeviceFactory(event_source)
    }
}

impl<'a> DeviceFactory for VulkanDeviceFactory<'a> {
    type Device = VulkanDevice;
    type PausedDevice = VulkanPausedDevice;
    type Error = VulkanError;
    fn create<T: Into<String>>(
        &self,
        title: T,
        position: Option<(i32, i32)>,
        size: (u32, u32),
        mut flags: u32,
    ) -> Result<VulkanPausedDevice> {
        assert_eq!(
            flags & (sdl::api::SDL_WINDOW_OPENGL | sdl::api::SDL_WINDOW_VULKAN),
            0
        );
        flags |= sdl::api::SDL_WINDOW_VULKAN;
        if unsafe { sdl::api::SDL_Vulkan_LoadLibrary(null()) } != 0 {
            return Err(sdl::get_error().into());
        }
        let window = sdl::window::Window::new(title, position, size, flags)?;
        let instance_functions =
            unsafe { InstanceFunctions::new(sdl::api::SDL_Vulkan_GetVkGetInstanceProcAddr()) };
        let mut extension_count = 0;
        if unsafe {
            sdl::api::SDL_Vulkan_GetInstanceExtensions(
                window.get(),
                &mut extension_count,
                null_mut(),
            )
        } == 0
        {
            return Err(sdl::get_error().into());
        }
        let mut extensions = Vec::new();
        extensions.resize(extension_count as usize, null());
        if unsafe {
            sdl::api::SDL_Vulkan_GetInstanceExtensions(
                window.get(),
                &mut extension_count,
                extensions.as_mut_ptr(),
            )
        } == 0
        {
            return Err(sdl::get_error().into());
        }
        let application_info = &api::VkApplicationInfo {
            sType: api::VK_STRUCTURE_TYPE_APPLICATION_INFO,
            pNext: null(),
            pApplicationName: null(),
            applicationVersion: 0,
            pEngineName: null(),
            engineVersion: 0,
            apiVersion: vk_make_version(1, 0, 0),
        };
        #[cfg(debug_assertions)]
        let layers = [
            CStr::from_bytes_with_nul(b"VK_LAYER_LUNARG_standard_validation\0")
                .unwrap()
                .as_ptr(),
        ];
        #[cfg(not(debug_assertions))]
        let layers = [];
        let instance = match unsafe {
            InstanceWrapper::new(instance_functions, application_info, &layers, &extensions)
        } {
            Ok(instance) => instance,
            Err(initial_error) => {
                if layers.len() == 0 {
                    return Err(initial_error);
                }
                eprintln!(
                    "failed to create Vulkan instance with layers enabled: {}",
                    initial_error
                );
                unsafe {
                    InstanceWrapper::new(instance_functions, application_info, &[], &extensions)
                }?
            }
        };
        let instance = Arc::new(instance);
        let surface = unsafe { SurfaceWrapper::new(window, instance.clone()) }?;
        let mut physical_device_count = 0;
        match unsafe {
            instance.vkEnumeratePhysicalDevices.unwrap()(
                instance.instance,
                &mut physical_device_count,
                null_mut(),
            )
        } {
            api::VK_SUCCESS => (),
            result => return Err(VulkanError::VulkanError(result)),
        }
        let mut physical_devices = Vec::new();
        physical_devices.resize(physical_device_count as usize, null_mut());
        match unsafe {
            instance.vkEnumeratePhysicalDevices.unwrap()(
                instance.instance,
                &mut physical_device_count,
                physical_devices.as_mut_ptr(),
            )
        } {
            api::VK_SUCCESS => (),
            result => return Err(VulkanError::VulkanError(result)),
        }
        let mut queue_family_properties_vec = Vec::new();
        let mut device_extensions = Vec::new();
        let mut surface_formats = Vec::new();
        for physical_device in physical_devices {
            let mut depth_32_format_properties = unsafe { mem::zeroed() };
            unsafe {
                instance.vkGetPhysicalDeviceFormatProperties.unwrap()(
                    physical_device,
                    api::VK_FORMAT_D32_SFLOAT,
                    &mut depth_32_format_properties,
                );
            }
            let depth_format;
            if (depth_32_format_properties.optimalTilingFeatures
                & api::VK_FORMAT_FEATURE_DEPTH_STENCIL_ATTACHMENT_BIT) != 0
            {
                depth_format = api::VK_FORMAT_D32_SFLOAT;
            } else {
                depth_format = api::VK_FORMAT_X8_D24_UNORM_PACK32;
            }
            let mut surface_capabilities = unsafe { mem::zeroed() };
            match unsafe {
                instance.vkGetPhysicalDeviceSurfaceCapabilitiesKHR.unwrap()(
                    physical_device,
                    surface.surface,
                    &mut surface_capabilities,
                )
            } {
                api::VK_SUCCESS => (),
                result => return Err(VulkanError::VulkanError(result)),
            }
            let mut surface_format_count = 0;
            match unsafe {
                instance.vkGetPhysicalDeviceSurfaceFormatsKHR.unwrap()(
                    physical_device,
                    surface.surface,
                    &mut surface_format_count,
                    null_mut(),
                )
            } {
                api::VK_SUCCESS => (),
                result => return Err(VulkanError::VulkanError(result)),
            }
            surface_formats.clear();
            surface_formats.resize(surface_format_count as usize, unsafe { mem::zeroed() });
            match unsafe {
                instance.vkGetPhysicalDeviceSurfaceFormatsKHR.unwrap()(
                    physical_device,
                    surface.surface,
                    &mut surface_format_count,
                    surface_formats.as_mut_ptr(),
                )
            } {
                api::VK_SUCCESS => (),
                result => return Err(VulkanError::VulkanError(result)),
            }
            assert!(surface_formats.len() != 0);
            let surface_format;
            if surface_formats.len() == 1 && surface_formats[0].format == api::VK_FORMAT_UNDEFINED {
                surface_format = Some(api::VkSurfaceFormatKHR {
                    format: api::VK_FORMAT_B8G8R8A8_SRGB,
                    colorSpace: api::VK_COLOR_SPACE_SRGB_NONLINEAR_KHR,
                });
            } else {
                surface_format = surface_formats
                    .iter()
                    .find(|&&format| {
                        let mut format_properties = unsafe { mem::zeroed() };
                        unsafe {
                            instance.vkGetPhysicalDeviceFormatProperties.unwrap()(
                                physical_device,
                                format.format,
                                &mut format_properties,
                            );
                        }
                        let required = api::VK_FORMAT_FEATURE_COLOR_ATTACHMENT_BLEND_BIT
                            | api::VK_FORMAT_FEATURE_COLOR_ATTACHMENT_BIT;
                        (format_properties.optimalTilingFeatures & required) != required
                    })
                    .map(|v| *v);
            }
            let mut queue_family_count = 0;
            unsafe {
                instance.vkGetPhysicalDeviceQueueFamilyProperties.unwrap()(
                    physical_device,
                    &mut queue_family_count,
                    null_mut(),
                );
            }
            queue_family_properties_vec.clear();
            queue_family_properties_vec
                .resize(queue_family_count as usize, unsafe { mem::zeroed() });
            unsafe {
                instance.vkGetPhysicalDeviceQueueFamilyProperties.unwrap()(
                    physical_device,
                    &mut queue_family_count,
                    queue_family_properties_vec.as_mut_ptr(),
                );
            }
            let mut present_queue_index = None;
            let mut render_queue_index = None;
            for queue_family_index in 0..queue_family_count {
                let queue_family_properties =
                    &queue_family_properties_vec[queue_family_index as usize];
                let mut surface_supported = 0;
                match unsafe {
                    instance.vkGetPhysicalDeviceSurfaceSupportKHR.unwrap()(
                        physical_device,
                        queue_family_index,
                        surface.surface,
                        &mut surface_supported,
                    )
                } {
                    api::VK_SUCCESS => (),
                    result => return Err(VulkanError::VulkanError(result)),
                }
                if queue_family_properties.queueFlags & api::VK_QUEUE_GRAPHICS_BIT != 0 {
                    render_queue_index = Some(queue_family_index);
                    if surface_supported != 0 {
                        present_queue_index = Some(queue_family_index);
                        break;
                    }
                }
                if surface_supported != 0 {
                    present_queue_index = Some(queue_family_index);
                }
            }
            let mut device_extension_count = 0;
            match unsafe {
                instance.vkEnumerateDeviceExtensionProperties.unwrap()(
                    physical_device,
                    null(),
                    &mut device_extension_count,
                    null_mut(),
                )
            } {
                api::VK_SUCCESS => (),
                result => return Err(VulkanError::VulkanError(result)),
            }
            device_extensions.clear();
            device_extensions
                .resize_with(device_extension_count as usize, || unsafe { mem::zeroed() });
            match unsafe {
                instance.vkEnumerateDeviceExtensionProperties.unwrap()(
                    physical_device,
                    null(),
                    &mut device_extension_count,
                    device_extensions.as_mut_ptr(),
                )
            } {
                api::VK_SUCCESS => (),
                result => return Err(VulkanError::VulkanError(result)),
            }
            let mut has_swapchain_extension = false;
            for device_extension in &device_extensions {
                if unsafe { CStr::from_ptr(device_extension.extensionName.as_ptr()) }
                    == CStr::from_bytes_with_nul(api::VK_KHR_SWAPCHAIN_EXTENSION_NAME).unwrap()
                {
                    has_swapchain_extension = true;
                    break;
                }
            }
            match (
                present_queue_index,
                render_queue_index,
                has_swapchain_extension,
                surface_format,
            ) {
                (
                    Some(present_queue_index),
                    Some(render_queue_index),
                    true,
                    Some(surface_format),
                ) => {
                    return Ok(VulkanPausedDevice {
                        surface_state: SurfaceState {
                            surface: surface,
                            physical_device: physical_device,
                            present_queue_index: present_queue_index,
                            render_queue_index: render_queue_index,
                            surface_format: surface_format,
                            depth_format: depth_format,
                        },
                    });
                }
                _ => continue,
            }
        }
        Err(VulkanError::NoMatchingPhysicalDevice)
    }
}
