#[macro_use]
mod instance_functions;
mod buffer;
mod command_buffer;
mod device;
mod device_memory;
mod error;
mod fence;
mod image;
mod instance;
mod semaphore;
mod surface;
use self::buffer::*;
use self::command_buffer::*;
use self::device::*;
use self::device_memory::*;
use self::error::*;
use self::fence::*;
use self::image::*;
use self::instance::*;
use self::instance_functions::*;
use self::semaphore::*;
use self::surface::*;
use super::*;
use sdl;
use std::any::Any;
use std::collections::VecDeque;
use std::ffi::CStr;
use std::mem;
use std::os::raw::c_void;
use std::ptr::*;
use std::rc::Rc;
use std::sync::Arc;
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
    render_queue_index: u32,
    render_pass: Arc<RenderPassWrapper>,
    device_memory_pools: Arc<DeviceMemoryPools>,
    pipeline_layout: Arc<PipelineLayoutWrapper>,
    graphics_pipeline: Option<Arc<GraphicsPipelineWrapper>>,
}

pub struct VulkanPausedDevice {
    surface_state: SurfaceState,
}

struct SwapchainWrapper {
    device: Arc<DeviceWrapper>,
    _surface: Rc<SurfaceWrapper>,
    swapchain: api::VkSwapchainKHR,
}

impl Drop for SwapchainWrapper {
    fn drop(&mut self) {
        unsafe {
            self.device.vkDestroySwapchainKHR.unwrap()(self.device.device, self.swapchain, null());
        }
    }
}

struct SwapchainState {
    swapchain: SwapchainWrapper,
    dimensions: (u32, u32),
    framebuffers: Vec<FramebufferWrapper>,
}

pub struct VulkanDevice {
    device_reference: VulkanDeviceReference,
    surface_state: Option<SurfaceState>,
    render_queue: api::VkQueue,
    present_queue: api::VkQueue,
    swapchain: Option<Arc<SwapchainState>>,
    in_progress_operations: VecDeque<(FenceWrapper, Vec<Box<Any>>)>,
    in_progress_present_semaphores: VecDeque<SemaphoreWrapper>,
}

mod shader_module {
    use super::*;

    pub struct ShaderModuleWrapper {
        pub device: Arc<DeviceWrapper>,
        pub shader_module: api::VkShaderModule,
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
}

use self::shader_module::ShaderModuleWrapper;

mod render_pass {
    use super::*;

    pub struct RenderPassWrapper {
        pub device: Arc<DeviceWrapper>,
        pub render_pass: api::VkRenderPass,
    }

    impl Drop for RenderPassWrapper {
        fn drop(&mut self) {
            unsafe {
                self.device.vkDestroyRenderPass.unwrap()(
                    self.device.device,
                    self.render_pass,
                    null(),
                );
            }
        }
    }
}

use self::render_pass::RenderPassWrapper;

mod graphics_pipeline {
    use super::*;

    pub struct GraphicsPipelineWrapper {
        pub device: Arc<DeviceWrapper>,
        pub pipeline: api::VkPipeline,
        pub pipeline_layout: Arc<PipelineLayoutWrapper>,
        pub _vertex_shader: ShaderModuleWrapper,
        pub _fragment_shader: ShaderModuleWrapper,
        pub render_pass: Arc<RenderPassWrapper>,
    }

    impl Drop for GraphicsPipelineWrapper {
        fn drop(&mut self) {
            unsafe {
                self.device.vkDestroyPipeline.unwrap()(self.device.device, self.pipeline, null())
            }
        }
    }
}

use self::graphics_pipeline::GraphicsPipelineWrapper;

mod descriptor_set_layout {
    use super::*;
    pub struct DescriptorSetLayoutWrapper {
        pub device: Arc<DeviceWrapper>,
        pub descriptor_set_layout: api::VkDescriptorSetLayout,
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
}

use self::descriptor_set_layout::DescriptorSetLayoutWrapper;

mod pipeline_layout {
    use super::*;
    pub struct PipelineLayoutWrapper {
        pub device: Arc<DeviceWrapper>,
        pub pipeline_layout: api::VkPipelineLayout,
        pub _descriptor_set_layouts: Vec<DescriptorSetLayoutWrapper>,
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
}

use self::pipeline_layout::PipelineLayoutWrapper;

struct FramebufferWrapper {
    device: Arc<DeviceWrapper>,
    _image_views: Vec<Arc<ImageViewWrapper>>,
    framebuffer: api::VkFramebuffer,
}

impl Drop for FramebufferWrapper {
    fn drop(&mut self) {
        unsafe {
            self.device.vkDestroyFramebuffer.unwrap()(self.device.device, self.framebuffer, null());
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
    type Error = VulkanError;
    type LoaderCommandBuffer = VulkanLoaderCommandBuffer;
    type LoaderCommandBufferBuilder = VulkanLoaderCommandBufferBuilder;
    type RenderCommandBuffer = VulkanRenderCommandBuffer;
    type RenderCommandBufferBuilder = VulkanRenderCommandBufferBuilder;
    type StagingVertexBuffer = VulkanStagingVertexBuffer;
    type DeviceVertexBuffer = VulkanDeviceVertexBuffer;
    type StagingIndexBuffer = VulkanStagingIndexBuffer;
    type DeviceIndexBuffer = VulkanDeviceIndexBuffer;
    fn create_loader_command_buffer_builder(&self) -> Result<VulkanLoaderCommandBufferBuilder> {
        unsafe { VulkanLoaderCommandBufferBuilder::new(&self.device, self.render_queue_index) }
    }
    fn create_render_command_buffer_builder(&self) -> Result<VulkanRenderCommandBufferBuilder> {
        unsafe {
            Ok(VulkanRenderCommandBufferBuilder::new(
                self.device.clone(),
                self.render_queue_index,
                self.render_pass.clone(),
                self.pipeline_layout.clone(),
                self.graphics_pipeline.clone().unwrap(),
            ))
        }
    }
    fn create_staging_vertex_buffer(&self, len: usize) -> Result<VulkanStagingVertexBuffer> {
        unsafe {
            create_staging_vertex_buffer(self.device.clone(), &*self.device_memory_pools, len)
        }
    }
    fn create_staging_index_buffer(&self, len: usize) -> Result<VulkanStagingIndexBuffer> {
        unsafe { create_staging_index_buffer(self.device.clone(), &*self.device_memory_pools, len) }
    }
}

impl PausedDevice for VulkanPausedDevice {
    type Device = VulkanDevice;
    fn get_window(&self) -> &sdl::window::Window {
        &self.surface_state.surface.window
    }
}

const FRAGMENT_TEXTURES_BINDING: u32 = 0;
const FRAGMENT_TEXTURES_BINDING_DESCRIPTOR_COUNT: u32 = 8;

#[repr(C)]
#[repr(align(16))]
struct AlignedMat4([[f32; 4]; 4]);

impl From<math::Mat4> for AlignedMat4 {
    fn from(v: math::Mat4) -> AlignedMat4 {
        AlignedMat4(v.into())
    }
}

impl<'a> From<&'a math::Mat4> for AlignedMat4 {
    fn from(v: &'a math::Mat4) -> AlignedMat4 {
        AlignedMat4(v.into())
    }
}

impl<'a> From<&'a mut math::Mat4> for AlignedMat4 {
    fn from(v: &'a mut math::Mat4) -> AlignedMat4 {
        AlignedMat4(v.into())
    }
}

/// must match PushConstants in shaders/vulkan_main.vert
#[repr(C)]
struct PushConstants {
    initial_transform: AlignedMat4,
    final_transform: AlignedMat4,
}

unsafe fn set_push_constants(
    device: &Arc<DeviceWrapper>,
    command_buffer: api::VkCommandBuffer,
    layout: api::VkPipelineLayout,
    stage_flags: api::VkShaderStageFlags,
    push_constants: PushConstants,
) {
    return; // FIXME: change back
    device.vkCmdPushConstants.unwrap()(
        command_buffer,
        layout,
        stage_flags,
        0,
        mem::size_of::<PushConstants>() as u32,
        &push_constants as *const PushConstants as *const c_void,
    );
}

unsafe fn set_push_constants_initial_transform(
    device: &Arc<DeviceWrapper>,
    command_buffer: api::VkCommandBuffer,
    layout: api::VkPipelineLayout,
    stage_flags: api::VkShaderStageFlags,
    push_constants_initial_transform: AlignedMat4,
) {
    return; // FIXME: change back
    let push_constants: PushConstants = mem::uninitialized();
    let offset = &push_constants.initial_transform as *const AlignedMat4 as usize
        - &push_constants as *const PushConstants as usize;
    mem::forget(push_constants);
    device.vkCmdPushConstants.unwrap()(
        command_buffer,
        layout,
        stage_flags,
        offset as u32,
        mem::size_of::<AlignedMat4>() as u32,
        &push_constants_initial_transform as *const AlignedMat4 as *const c_void,
    );
}

#[derive(PartialEq, Eq, Debug)]
enum FormatKind {
    Normalized,
    FullRange,
}

trait FormatFromType: 'static + Sized {
    fn get(format_kind: FormatKind) -> api::VkFormat;
}

impl<T: FormatFromType> FormatFromType for [T; 1] {
    fn get(format_kind: FormatKind) -> api::VkFormat {
        <T as FormatFromType>::get(format_kind)
    }
}

impl FormatFromType for u16 {
    fn get(format_kind: FormatKind) -> api::VkFormat {
        match format_kind {
            FormatKind::FullRange => api::VK_FORMAT_R16_UINT,
            FormatKind::Normalized => api::VK_FORMAT_R16_UNORM,
        }
    }
}

impl FormatFromType for [u8; 4] {
    fn get(format_kind: FormatKind) -> api::VkFormat {
        match format_kind {
            FormatKind::FullRange => api::VK_FORMAT_R8G8B8A8_UINT,
            FormatKind::Normalized => api::VK_FORMAT_R8G8B8A8_UNORM,
        }
    }
}

impl FormatFromType for f32 {
    fn get(format_kind: FormatKind) -> api::VkFormat {
        assert_eq!(format_kind, FormatKind::FullRange);
        api::VK_FORMAT_R32_SFLOAT
    }
}

impl FormatFromType for [f32; 2] {
    fn get(format_kind: FormatKind) -> api::VkFormat {
        assert_eq!(format_kind, FormatKind::FullRange);
        api::VK_FORMAT_R32G32_SFLOAT
    }
}

impl FormatFromType for [f32; 3] {
    fn get(format_kind: FormatKind) -> api::VkFormat {
        assert_eq!(format_kind, FormatKind::FullRange);
        api::VK_FORMAT_R32G32B32_SFLOAT
    }
}

impl FormatFromType for [f32; 4] {
    fn get(format_kind: FormatKind) -> api::VkFormat {
        assert_eq!(format_kind, FormatKind::FullRange);
        api::VK_FORMAT_R32G32B32A32_SFLOAT
    }
}

fn get_vulkan_format_from_type<'a, T: 'static + Sized + FormatFromType>(
    _: &'a T,
    format_kind: FormatKind,
) -> api::VkFormat {
    <T as FormatFromType>::get(format_kind)
}

macro_rules! get_vertex_input_attribute_description {
    ($location:expr, $binding:expr, $format_kind:expr, $member:tt) => {{
        let retval;
        let vertex_buffer_element: VertexBufferElement = unsafe { mem::uninitialized() };
        {
            let member_ref: &_ = &(vertex_buffer_element.$member);
            retval = api::VkVertexInputAttributeDescription {
                location: $location,
                binding: $binding,
                format: get_vulkan_format_from_type(member_ref, $format_kind),
                offset: (member_ref as *const _ as usize
                    - &vertex_buffer_element as *const _ as usize) as u32,
            };
            assert!(
                retval.offset as usize + mem::size_of_val(member_ref)
                    <= mem::size_of::<VertexBufferElement>()
            );
        }
        mem::forget(vertex_buffer_element);
        retval
    }};
}

const COLOR_ATTACHEMENT_INDEX: usize = 0;
const DEPTH_ATTACHEMENT_INDEX: usize = 1;

fn create_render_pass(
    device: Arc<DeviceWrapper>,
    surface_state: &SurfaceState,
) -> Result<RenderPassWrapper> {
    let mut render_pass = null_or_zero();
    let mut attachments: [api::VkAttachmentDescription; 2] = unsafe { mem::uninitialized() };
    attachments[COLOR_ATTACHEMENT_INDEX] = api::VkAttachmentDescription {
        flags: 0,
        format: surface_state.surface_format.format,
        samples: api::VK_SAMPLE_COUNT_1_BIT,
        loadOp: api::VK_ATTACHMENT_LOAD_OP_CLEAR,
        storeOp: api::VK_ATTACHMENT_STORE_OP_STORE,
        stencilLoadOp: api::VK_ATTACHMENT_LOAD_OP_DONT_CARE,
        stencilStoreOp: api::VK_ATTACHMENT_STORE_OP_DONT_CARE,
        initialLayout: api::VK_IMAGE_LAYOUT_UNDEFINED,
        finalLayout: api::VK_IMAGE_LAYOUT_PRESENT_SRC_KHR,
    };
    attachments[DEPTH_ATTACHEMENT_INDEX] = api::VkAttachmentDescription {
        flags: 0,
        format: surface_state.depth_format,
        samples: api::VK_SAMPLE_COUNT_1_BIT,
        loadOp: api::VK_ATTACHMENT_LOAD_OP_CLEAR,
        storeOp: api::VK_ATTACHMENT_STORE_OP_DONT_CARE,
        stencilLoadOp: api::VK_ATTACHMENT_LOAD_OP_DONT_CARE,
        stencilStoreOp: api::VK_ATTACHMENT_STORE_OP_DONT_CARE,
        initialLayout: api::VK_IMAGE_LAYOUT_UNDEFINED,
        finalLayout: api::VK_IMAGE_LAYOUT_DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
    };
    let color_attachments = [api::VkAttachmentReference {
        attachment: COLOR_ATTACHEMENT_INDEX as u32,
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
            attachment: DEPTH_ATTACHEMENT_INDEX as u32,
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

fn create_descriptor_set_layout(device: Arc<DeviceWrapper>) -> Result<DescriptorSetLayoutWrapper> {
    let bindings = [api::VkDescriptorSetLayoutBinding {
        binding: FRAGMENT_TEXTURES_BINDING,
        descriptorType: api::VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER,
        descriptorCount: FRAGMENT_TEXTURES_BINDING_DESCRIPTOR_COUNT,
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

fn create_pipeline_layout(device: Arc<DeviceWrapper>) -> Result<PipelineLayoutWrapper> {
    // FIXME: change back to having 1 descriptor set layout
    //let descriptor_set_layouts = vec![create_descriptor_set_layout(device.clone())?];
    let descriptor_set_layouts: Vec<DescriptorSetLayoutWrapper> = vec![];
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
    let push_constant_ranges = []; // FIXME: change back
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
            _descriptor_set_layouts: descriptor_set_layouts,
        }),
        result => Err(VulkanError::VulkanError(result)),
    }
}

impl VulkanDevice {
    fn get_shader(&self, shader_source: ShaderSource) -> Result<ShaderModuleWrapper> {
        self.get_device_ref().get_shader(shader_source)
    }
    fn create_graphics_pipeline(&self) -> Result<GraphicsPipelineWrapper> {
        let render_pass = self.device_reference.render_pass.clone();
        let device = self.device_reference.device.clone();
        let vertex_shader = self.get_shader(ShaderSource::MainVertex)?;
        let fragment_shader = self.get_shader(ShaderSource::MainFragment)?;
        let pipeline_layout = self.device_reference.pipeline_layout.clone();
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
        let vertex_attribute_descriptions = [
            get_vertex_input_attribute_description!(0, 0, FormatKind::FullRange, position),
            get_vertex_input_attribute_description!(1, 0, FormatKind::Normalized, color),
            get_vertex_input_attribute_description!(2, 0, FormatKind::FullRange, texture_coord),
            get_vertex_input_attribute_description!(3, 0, FormatKind::FullRange, texture_index),
        ];
        let vertex_attribute_descriptions = []; // FIXME: change back
        let attachments = [api::VkPipelineColorBlendAttachmentState {
            blendEnable: api::VK_FALSE, // FIXME: change back to true
            srcColorBlendFactor: api::VK_BLEND_FACTOR_SRC_ALPHA,
            dstColorBlendFactor: api::VK_BLEND_FACTOR_DST_ALPHA,
            colorBlendOp: api::VK_BLEND_OP_ADD,
            srcAlphaBlendFactor: api::VK_BLEND_FACTOR_ZERO,
            dstAlphaBlendFactor: api::VK_BLEND_FACTOR_CONSTANT_ALPHA,
            alphaBlendOp: api::VK_BLEND_OP_ADD,
            colorWriteMask: api::VK_COLOR_COMPONENT_R_BIT
                | api::VK_COLOR_COMPONENT_G_BIT
                | api::VK_COLOR_COMPONENT_B_BIT
                | api::VK_COLOR_COMPONENT_A_BIT,
        }];
        let dynamic_states = [
            api::VK_DYNAMIC_STATE_VIEWPORT,
            api::VK_DYNAMIC_STATE_SCISSOR,
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
                    pVertexInputState: &api::VkPipelineVertexInputStateCreateInfo {
                        sType: api::VK_STRUCTURE_TYPE_PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
                        pNext: null(),
                        flags: 0,
                        vertexBindingDescriptionCount: 1,
                        pVertexBindingDescriptions: &api::VkVertexInputBindingDescription {
                            binding: 0,
                            stride: mem::size_of::<VertexBufferElement>() as u32,
                            inputRate: api::VK_VERTEX_INPUT_RATE_VERTEX,
                        },
                        vertexAttributeDescriptionCount: vertex_attribute_descriptions.len() as u32,
                        pVertexAttributeDescriptions: vertex_attribute_descriptions.as_ptr(),
                    },
                    pInputAssemblyState: &api::VkPipelineInputAssemblyStateCreateInfo {
                        sType: api::VK_STRUCTURE_TYPE_PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
                        pNext: null(),
                        flags: 0,
                        topology: api::VK_PRIMITIVE_TOPOLOGY_TRIANGLE_LIST,
                        primitiveRestartEnable: api::VK_FALSE,
                    },
                    pTessellationState: null(),
                    pViewportState: &api::VkPipelineViewportStateCreateInfo {
                        sType: api::VK_STRUCTURE_TYPE_PIPELINE_VIEWPORT_STATE_CREATE_INFO,
                        pNext: null(),
                        flags: 0,
                        viewportCount: 1,
                        pViewports: null(),
                        scissorCount: 1,
                        pScissors: null(),
                    },
                    pRasterizationState: &api::VkPipelineRasterizationStateCreateInfo {
                        sType: api::VK_STRUCTURE_TYPE_PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
                        pNext: null(),
                        flags: 0,
                        depthClampEnable: api::VK_FALSE,
                        rasterizerDiscardEnable: api::VK_FALSE,
                        polygonMode: api::VK_POLYGON_MODE_FILL,
                        cullMode: api::VK_CULL_MODE_NONE, // FIXME: change back to back
                        frontFace: api::VK_FRONT_FACE_CLOCKWISE,
                        depthBiasEnable: api::VK_FALSE,
                        depthBiasConstantFactor: 0.0,
                        depthBiasClamp: 0.0,
                        depthBiasSlopeFactor: 0.0,
                        lineWidth: 1.0,
                    },
                    pMultisampleState: &api::VkPipelineMultisampleStateCreateInfo {
                        sType: api::VK_STRUCTURE_TYPE_PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
                        pNext: null(),
                        flags: 0,
                        rasterizationSamples: api::VK_SAMPLE_COUNT_1_BIT,
                        sampleShadingEnable: api::VK_FALSE,
                        minSampleShading: 0.0,
                        pSampleMask: null(),
                        alphaToCoverageEnable: api::VK_FALSE,
                        alphaToOneEnable: api::VK_FALSE,
                    },
                    pDepthStencilState: &api::VkPipelineDepthStencilStateCreateInfo {
                        sType: api::VK_STRUCTURE_TYPE_PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO,
                        pNext: null(),
                        flags: 0,
                        depthTestEnable: api::VK_FALSE, // FIXME: change back to true
                        depthWriteEnable: api::VK_TRUE,
                        depthCompareOp: api::VK_COMPARE_OP_ALWAYS, // FIXME: change back to less
                        depthBoundsTestEnable: api::VK_FALSE,
                        stencilTestEnable: api::VK_FALSE,
                        front: mem::zeroed(),
                        back: mem::zeroed(),
                        minDepthBounds: 0.0,
                        maxDepthBounds: 0.0,
                    },
                    pColorBlendState: &api::VkPipelineColorBlendStateCreateInfo {
                        sType: api::VK_STRUCTURE_TYPE_PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
                        pNext: null(),
                        flags: 0,
                        logicOpEnable: api::VK_FALSE,
                        logicOp: api::VK_LOGIC_OP_COPY,
                        attachmentCount: attachments.len() as u32,
                        pAttachments: attachments.as_ptr(),
                        blendConstants: [0.0, 0.0, 0.0, 1.0],
                    },
                    pDynamicState: &api::VkPipelineDynamicStateCreateInfo {
                        sType: api::VK_STRUCTURE_TYPE_PIPELINE_DYNAMIC_STATE_CREATE_INFO,
                        pNext: null(),
                        flags: 0,
                        dynamicStateCount: dynamic_states.len() as u32,
                        pDynamicStates: dynamic_states.as_ptr(),
                    },
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
                _vertex_shader: vertex_shader,
                _fragment_shader: fragment_shader,
                render_pass: render_pass,
            }),
            result => Err(VulkanError::VulkanError(result)),
        }
    }
    fn create_swapchain(
        &self,
        previous_swapchain: Option<Arc<SwapchainState>>,
    ) -> Result<Option<SwapchainState>> {
        let mut dimensions = (0, 0);
        unsafe {
            sdl::api::SDL_Vulkan_GetDrawableSize(
                self.get_window().get(),
                &mut dimensions.0,
                &mut dimensions.1,
            );
        }
        let dimensions = match dimensions {
            (0, _) | (_, 0) => return Ok(None),
            dimensions => (dimensions.0 as u32, dimensions.1 as u32),
        };
        let device = self.device_reference.device.clone();
        let mut swapchain = null_or_zero();
        let swapchain = match unsafe {
            device.vkCreateSwapchainKHR.unwrap()(
                device.device,
                &api::VkSwapchainCreateInfoKHR {
                    sType: api::VK_STRUCTURE_TYPE_SWAPCHAIN_CREATE_INFO_KHR,
                    pNext: null(),
                    flags: 0,
                    surface: self.surface_state.as_ref().unwrap().surface.surface,
                    minImageCount: self
                        .surface_state
                        .as_ref()
                        .unwrap()
                        .swapchain_desired_image_count,
                    imageFormat: self.surface_state.as_ref().unwrap().surface_format.format,
                    imageColorSpace: self
                        .surface_state
                        .as_ref()
                        .unwrap()
                        .surface_format
                        .colorSpace,
                    imageExtent: api::VkExtent2D {
                        width: dimensions.0,
                        height: dimensions.1,
                    },
                    imageArrayLayers: 1,
                    imageUsage: api::VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT,
                    imageSharingMode: api::VK_SHARING_MODE_EXCLUSIVE,
                    queueFamilyIndexCount: 0,
                    pQueueFamilyIndices: null(),
                    preTransform: self.surface_state.as_ref().unwrap().swapchain_pre_transform,
                    compositeAlpha: self
                        .surface_state
                        .as_ref()
                        .unwrap()
                        .swapchain_composite_alpha,
                    presentMode: self.surface_state.as_ref().unwrap().present_mode,
                    clipped: api::VK_TRUE,
                    oldSwapchain: match previous_swapchain {
                        Some(v) => v.swapchain.swapchain,
                        None => null_or_zero(),
                    },
                },
                null(),
                &mut swapchain,
            )
        } {
            api::VK_SUCCESS => SwapchainWrapper {
                device: device.clone(),
                _surface: self.surface_state.as_ref().unwrap().surface.clone(),
                swapchain: swapchain,
            },
            result => return Err(VulkanError::VulkanError(result)),
        };
        let mut images = Vec::new();
        {
            let mut image_count = 0;
            match unsafe {
                device.vkGetSwapchainImagesKHR.unwrap()(
                    device.device,
                    swapchain.swapchain,
                    &mut image_count,
                    null_mut(),
                )
            } {
                api::VK_SUCCESS => (),
                result => return Err(VulkanError::VulkanError(result)),
            }
            images.resize(image_count as usize, null_or_zero());
            match unsafe {
                device.vkGetSwapchainImagesKHR.unwrap()(
                    device.device,
                    swapchain.swapchain,
                    &mut image_count,
                    images.as_mut_ptr(),
                )
            } {
                api::VK_SUCCESS => (),
                result => return Err(VulkanError::VulkanError(result)),
            }
        }
        let images: Vec<_> = images
            .into_iter()
            .map(|image| {
                Arc::new(ImageWrapper {
                    device: device.clone(),
                    image: image,
                    destroy_on_drop: false,
                    device_memory: None,
                })
            }).collect();
        let mut framebuffers = Vec::with_capacity(images.len());
        for color_image in &images {
            let depth_image = Arc::new(unsafe {
                ImageWrapper::new_depth(
                    device.clone(),
                    self.surface_state.as_ref().unwrap().depth_format,
                    dimensions.0,
                    dimensions.1,
                )?.allocate_and_bind_memory(
                    &*self.device_reference.device_memory_pools,
                    None,
                    api::VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT,
                )
            }?);
            let mut color_image_view = null_or_zero();
            let color_image_view = match unsafe {
                device.vkCreateImageView.unwrap()(
                    device.device,
                    &api::VkImageViewCreateInfo {
                        sType: api::VK_STRUCTURE_TYPE_IMAGE_VIEW_CREATE_INFO,
                        pNext: null(),
                        flags: 0,
                        image: color_image.image,
                        viewType: api::VK_IMAGE_VIEW_TYPE_2D,
                        format: self.surface_state.as_ref().unwrap().surface_format.format,
                        components: api::VkComponentMapping {
                            r: api::VK_COMPONENT_SWIZZLE_IDENTITY,
                            g: api::VK_COMPONENT_SWIZZLE_IDENTITY,
                            b: api::VK_COMPONENT_SWIZZLE_IDENTITY,
                            a: api::VK_COMPONENT_SWIZZLE_IDENTITY,
                        },
                        subresourceRange: api::VkImageSubresourceRange {
                            aspectMask: api::VK_IMAGE_ASPECT_COLOR_BIT,
                            baseMipLevel: 0,
                            levelCount: 1,
                            baseArrayLayer: 0,
                            layerCount: 1,
                        },
                    },
                    null(),
                    &mut color_image_view,
                )
            } {
                api::VK_SUCCESS => Arc::new(ImageViewWrapper {
                    image: color_image.clone(),
                    image_view: color_image_view,
                }),
                result => return Err(VulkanError::VulkanError(result)),
            };
            let mut depth_image_view = null_or_zero();
            let depth_image_view = match unsafe {
                device.vkCreateImageView.unwrap()(
                    device.device,
                    &api::VkImageViewCreateInfo {
                        sType: api::VK_STRUCTURE_TYPE_IMAGE_VIEW_CREATE_INFO,
                        pNext: null(),
                        flags: 0,
                        image: depth_image.image,
                        viewType: api::VK_IMAGE_VIEW_TYPE_2D,
                        format: self.surface_state.as_ref().unwrap().depth_format,
                        components: api::VkComponentMapping {
                            r: api::VK_COMPONENT_SWIZZLE_IDENTITY,
                            g: api::VK_COMPONENT_SWIZZLE_IDENTITY,
                            b: api::VK_COMPONENT_SWIZZLE_IDENTITY,
                            a: api::VK_COMPONENT_SWIZZLE_IDENTITY,
                        },
                        subresourceRange: api::VkImageSubresourceRange {
                            aspectMask: api::VK_IMAGE_ASPECT_DEPTH_BIT,
                            baseMipLevel: 0,
                            levelCount: 1,
                            baseArrayLayer: 0,
                            layerCount: 1,
                        },
                    },
                    null(),
                    &mut depth_image_view,
                )
            } {
                api::VK_SUCCESS => Arc::new(ImageViewWrapper {
                    image: depth_image.clone(),
                    image_view: depth_image_view,
                }),
                result => return Err(VulkanError::VulkanError(result)),
            };
            let mut image_views = [None, None];
            image_views[COLOR_ATTACHEMENT_INDEX] = Some(color_image_view);
            image_views[DEPTH_ATTACHEMENT_INDEX] = Some(depth_image_view);
            let image_views: Vec<_> = image_views
                .into_iter()
                .map(|v| v.clone().unwrap())
                .collect();
            let image_view_handles: Vec<_> = image_views.iter().map(|v| v.image_view).collect();
            let mut framebuffer = null_or_zero();
            let framebuffer = match unsafe {
                device.vkCreateFramebuffer.unwrap()(
                    device.device,
                    &api::VkFramebufferCreateInfo {
                        sType: api::VK_STRUCTURE_TYPE_FRAMEBUFFER_CREATE_INFO,
                        pNext: null(),
                        flags: 0,
                        renderPass: self
                            .device_reference
                            .graphics_pipeline
                            .as_ref()
                            .unwrap()
                            .render_pass
                            .render_pass,
                        attachmentCount: image_views.len() as u32,
                        pAttachments: image_view_handles.as_ptr(),
                        width: dimensions.0,
                        height: dimensions.1,
                        layers: 1,
                    },
                    null(),
                    &mut framebuffer,
                )
            } {
                api::VK_SUCCESS => FramebufferWrapper {
                    device: device.clone(),
                    _image_views: image_views,
                    framebuffer: framebuffer,
                },
                result => return Err(VulkanError::VulkanError(result)),
            };
            framebuffers.push(framebuffer);
        }
        Ok(Some(SwapchainState {
            swapchain: swapchain,
            framebuffers: framebuffers,
            dimensions: dimensions,
        }))
    }
    fn free_finished_objects(&mut self) -> Result<()> {
        let device = &self.device_reference.device;
        loop {
            if let Some(front) = self.in_progress_operations.front() {
                match unsafe { device.vkGetFenceStatus.unwrap()(device.device, front.0.fence) } {
                    api::VK_SUCCESS => (),
                    api::VK_NOT_READY => break,
                    result => return Err(VulkanError::VulkanError(result)),
                }
            } else {
                break;
            }
            self.in_progress_operations.pop_front();
        }
        Ok(())
    }
}

impl Drop for VulkanDevice {
    fn drop(&mut self) {
        let device = &self.device_reference.device;
        unsafe {
            device.vkDeviceWaitIdle.unwrap()(device.device);
        }
    }
}

impl Device for VulkanDevice {
    type Error = VulkanError;
    type Reference = VulkanDeviceReference;
    type PausedDevice = VulkanPausedDevice;
    type LoaderCommandBuffer = VulkanLoaderCommandBuffer;
    type LoaderCommandBufferBuilder = VulkanLoaderCommandBufferBuilder;
    type RenderCommandBuffer = VulkanRenderCommandBuffer;
    type RenderCommandBufferBuilder = VulkanRenderCommandBufferBuilder;
    type StagingVertexBuffer = VulkanStagingVertexBuffer;
    type DeviceVertexBuffer = VulkanDeviceVertexBuffer;
    type StagingIndexBuffer = VulkanStagingIndexBuffer;
    type DeviceIndexBuffer = VulkanDeviceIndexBuffer;
    fn pause(mut self) -> VulkanPausedDevice {
        VulkanPausedDevice {
            surface_state: self.surface_state.take().unwrap(),
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
            present_mode,
            swapchain_desired_image_count,
            swapchain_pre_transform,
            swapchain_composite_alpha,
            physical_device_memory_properties,
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
        let device = Arc::new(device);
        let surface_state = SurfaceState {
            surface: surface,
            physical_device: physical_device,
            present_queue_index: present_queue_index,
            render_queue_index: render_queue_index,
            surface_format: surface_format,
            depth_format: depth_format,
            present_mode: present_mode,
            swapchain_desired_image_count: swapchain_desired_image_count,
            swapchain_pre_transform: swapchain_pre_transform,
            swapchain_composite_alpha: swapchain_composite_alpha,
            physical_device_memory_properties: physical_device_memory_properties,
        };
        let mut retval = VulkanDevice {
            device_reference: VulkanDeviceReference {
                device: device.clone(),
                render_queue_index: render_queue_index,
                device_memory_pools: Arc::new(unsafe {
                    DeviceMemoryPools::new(device.clone(), physical_device_memory_properties)
                }),
                render_pass: Arc::new(create_render_pass(device.clone(), &surface_state)?),
                pipeline_layout: Arc::new(create_pipeline_layout(device)?),
                graphics_pipeline: None,
            },
            surface_state: Some(surface_state),
            render_queue: render_queue,
            present_queue: present_queue,
            swapchain: None,
            in_progress_operations: VecDeque::new(),
            in_progress_present_semaphores: VecDeque::new(),
        };
        retval.device_reference.graphics_pipeline =
            Some(Arc::new(retval.create_graphics_pipeline()?));
        retval.swapchain = retval.create_swapchain(None)?.map(|v| Arc::new(v));
        return Ok(retval);
    }
    fn get_window(&self) -> &sdl::window::Window {
        &self.surface_state.as_ref().unwrap().surface.window
    }
    fn get_device_ref(&self) -> &VulkanDeviceReference {
        &self.device_reference
    }
    fn submit_loader_command_buffers(
        &mut self,
        loader_command_buffers: &mut Vec<VulkanLoaderCommandBuffer>,
    ) -> Result<()> {
        submit_loader_command_buffers(self, loader_command_buffers)
    }
    fn render_frame(
        &mut self,
        clear_color: math::Vec4<f32>,
        loader_command_buffers: &mut Vec<VulkanLoaderCommandBuffer>,
        render_command_buffer_groups: &[RenderCommandBufferGroup<VulkanRenderCommandBuffer>],
    ) -> Result<()> {
        unsafe {
            render_frame(
                self,
                clear_color,
                loader_command_buffers,
                render_command_buffer_groups,
            )
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
            // FIXME: remove api dump layer
            /*CStr::from_bytes_with_nul(b"VK_LAYER_LUNARG_api_dump\0")
                .unwrap()
                .as_ptr(),*/
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
        let mut present_modes = Vec::new();
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
                & api::VK_FORMAT_FEATURE_DEPTH_STENCIL_ATTACHMENT_BIT)
                != 0
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
            let required_image_usage = api::VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT;
            if (required_image_usage & surface_capabilities.supportedUsageFlags)
                != required_image_usage
            {
                continue;
            }
            let swapchain_pre_transform;
            if (surface_capabilities.supportedTransforms
                & api::VK_SURFACE_TRANSFORM_IDENTITY_BIT_KHR)
                != 0
            {
                swapchain_pre_transform = api::VK_SURFACE_TRANSFORM_IDENTITY_BIT_KHR;
            } else if (surface_capabilities.supportedTransforms
                & api::VK_SURFACE_TRANSFORM_INHERIT_BIT_KHR)
                != 0
            {
                swapchain_pre_transform = api::VK_SURFACE_TRANSFORM_INHERIT_BIT_KHR;
            } else {
                continue;
            }
            let mut swapchain_composite_alpha = None;
            for &flag in &[
                api::VK_COMPOSITE_ALPHA_OPAQUE_BIT_KHR,
                api::VK_COMPOSITE_ALPHA_POST_MULTIPLIED_BIT_KHR,
                api::VK_COMPOSITE_ALPHA_PRE_MULTIPLIED_BIT_KHR,
                api::VK_COMPOSITE_ALPHA_INHERIT_BIT_KHR,
            ] {
                if (surface_capabilities.supportedCompositeAlpha & flag) != 0 {
                    swapchain_composite_alpha = Some(flag);
                    break;
                }
            }
            let swapchain_composite_alpha =
                if let Some(swapchain_composite_alpha) = swapchain_composite_alpha {
                    swapchain_composite_alpha
                } else {
                    continue;
                };
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
                        (format_properties.optimalTilingFeatures & required) == required
                    }).map(|v| *v);
            }
            let mut present_mode_count = 0;
            match unsafe {
                instance.vkGetPhysicalDeviceSurfacePresentModesKHR.unwrap()(
                    physical_device,
                    surface.surface,
                    &mut present_mode_count,
                    null_mut(),
                )
            } {
                api::VK_SUCCESS => (),
                result => return Err(VulkanError::VulkanError(result)),
            }
            present_modes.clear();
            present_modes.resize(present_mode_count as usize, unsafe { mem::zeroed() });
            match unsafe {
                instance.vkGetPhysicalDeviceSurfacePresentModesKHR.unwrap()(
                    physical_device,
                    surface.surface,
                    &mut present_mode_count,
                    present_modes.as_mut_ptr(),
                )
            } {
                api::VK_SUCCESS => (),
                result => return Err(VulkanError::VulkanError(result)),
            }
            let mut present_mode = api::VK_PRESENT_MODE_FIFO_KHR;
            for &mode in &present_modes {
                match (mode, present_mode) {
                    (api::VK_PRESENT_MODE_IMMEDIATE_KHR, api::VK_PRESENT_MODE_FIFO_KHR) => {
                        present_mode = mode;
                    }
                    (api::VK_PRESENT_MODE_MAILBOX_KHR, _) => {
                        present_mode = mode;
                        break;
                    }
                    _ => {}
                }
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
            let mut swapchain_desired_image_count = surface_capabilities.minImageCount + 1;
            if swapchain_desired_image_count < 3 {
                swapchain_desired_image_count = 3;
            }
            if swapchain_desired_image_count > surface_capabilities.maxImageCount
                && surface_capabilities.maxImageCount != 0
            {
                swapchain_desired_image_count = surface_capabilities.maxImageCount;
            }
            let mut physical_device_memory_properties = unsafe { mem::zeroed() };
            unsafe {
                instance.vkGetPhysicalDeviceMemoryProperties.unwrap()(
                    physical_device,
                    &mut physical_device_memory_properties,
                );
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
                            surface: Rc::new(surface),
                            physical_device: physical_device,
                            present_queue_index: present_queue_index,
                            render_queue_index: render_queue_index,
                            surface_format: surface_format,
                            depth_format: depth_format,
                            present_mode: present_mode,
                            swapchain_desired_image_count: swapchain_desired_image_count,
                            swapchain_pre_transform: swapchain_pre_transform,
                            swapchain_composite_alpha: swapchain_composite_alpha,
                            physical_device_memory_properties: physical_device_memory_properties,
                        },
                    });
                }
                _ => continue,
            }
        }
        Err(VulkanError::NoMatchingPhysicalDevice)
    }
}
