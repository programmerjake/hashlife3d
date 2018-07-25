use super::super::sdl;
use super::*;
use std::error::Error;
use std::ffi::CStr;
use std::fmt;
use std::mem;
use std::os::raw::*;
use std::ptr::*;
use std::result;
use std::sync::Arc;
use std::time::Duration;
use std::u32;

#[allow(dead_code)]
#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
mod api {
    include!(concat!(env!("OUT_DIR"), "/vulkan-bindings.rs"));
}

pub struct VulkanSemaphore {}

impl Drop for VulkanSemaphore {
    fn drop(&mut self) {
        unimplemented!()
    }
}

impl Semaphore for VulkanSemaphore {}

pub struct VulkanFence {
    device: Arc<DeviceWrapper>,
    fence: api::VkFence,
}

unsafe impl Send for VulkanFence {}

impl Fence for VulkanFence {}

impl Drop for VulkanFence {
    fn drop(&mut self) {
        unsafe {
            self.device.vkDestroyFence.unwrap()(self.device.device, self.fence, null());
        }
    }
}

pub struct VulkanQueue {
    queue: api::VkQueue,
}

impl Queue for VulkanQueue {}

#[allow(non_snake_case)]
struct InstanceFunctions {
    vkGetInstanceProcAddr: api::PFN_vkGetInstanceProcAddr,
    vkCreateInstance: api::PFN_vkCreateInstance,
}

unsafe impl Sync for InstanceFunctions {}
unsafe impl Send for InstanceFunctions {}

pub unsafe fn get_instance_fn(
    vk_get_instance_proc_addr: api::PFN_vkGetInstanceProcAddr,
    instance: api::VkInstance,
    name: &[u8],
) -> api::PFN_vkVoidFunction {
    let name = CStr::from_bytes_with_nul(name).unwrap();
    match vk_get_instance_proc_addr.unwrap()(instance, name.as_ptr()) {
        Some(retval) => Some(retval),
        None => panic!(
            "vkGetInstanceProcAddr failed: function not found: {}",
            name.to_string_lossy()
        ),
    }
}

macro_rules! get_instance_fn {
    ($vk_get_instance_proc_addr:expr, $instance:expr, $name:ident) => {{
        use self::api::*;
        mem::transmute::<api::PFN_vkVoidFunction, concat_idents!(PFN_, $name)>(
            self::get_instance_fn(
                $vk_get_instance_proc_addr,
                $instance,
                concat!(stringify!($name), "\0").as_bytes(),
            ),
        )
    }};
}

macro_rules! get_global_fn {
    ($vk_get_instance_proc_addr:expr, $name:ident) => {
        get_instance_fn!($vk_get_instance_proc_addr, null_mut(), $name)
    };
}

impl InstanceFunctions {
    unsafe fn new(vk_get_instance_proc_addr: *const c_void) -> Self {
        let vk_get_instance_proc_addr: api::PFN_vkGetInstanceProcAddr =
            mem::transmute(vk_get_instance_proc_addr);
        assert!(vk_get_instance_proc_addr.is_some());
        Self {
            vkGetInstanceProcAddr: vk_get_instance_proc_addr,
            vkCreateInstance: get_global_fn!(vk_get_instance_proc_addr, vkCreateInstance),
        }
    }
}

#[allow(non_snake_case)]
struct InstanceWrapper {
    instance: api::VkInstance,
    _instance_functions: InstanceFunctions,
    vkDestroyInstance: api::PFN_vkDestroyInstance,
    vkCreateDevice: api::PFN_vkCreateDevice,
    vkGetDeviceProcAddr: api::PFN_vkGetDeviceProcAddr,
    vkDestroySurfaceKHR: api::PFN_vkDestroySurfaceKHR,
    vkEnumeratePhysicalDevices: api::PFN_vkEnumeratePhysicalDevices,
    vkGetPhysicalDeviceSurfaceSupportKHR: api::PFN_vkGetPhysicalDeviceSurfaceSupportKHR,
    vkGetPhysicalDeviceQueueFamilyProperties: api::PFN_vkGetPhysicalDeviceQueueFamilyProperties,
    vkEnumerateDeviceExtensionProperties: api::PFN_vkEnumerateDeviceExtensionProperties,
}

unsafe impl Send for InstanceWrapper {}

unsafe impl Sync for InstanceWrapper {}

impl InstanceWrapper {
    pub unsafe fn new(
        instance_functions: InstanceFunctions,
        application_info: *const api::VkApplicationInfo,
        enabled_layer_names: &[*const c_char],
        enabled_extension_names: &[*const c_char],
    ) -> Result<Self> {
        let mut instance = null_mut();
        match instance_functions.vkCreateInstance.unwrap()(
            &api::VkInstanceCreateInfo {
                sType: api::VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO,
                pNext: null(),
                flags: 0,
                pApplicationInfo: application_info,
                enabledLayerCount: enabled_layer_names.len() as u32,
                ppEnabledLayerNames: enabled_layer_names.as_ptr(),
                enabledExtensionCount: enabled_extension_names.len() as u32,
                ppEnabledExtensionNames: enabled_extension_names.as_ptr(),
            },
            null(),
            &mut instance,
        ) {
            api::VK_SUCCESS => {
                let vk_get_instance_proc_addr = instance_functions.vkGetInstanceProcAddr;
                Ok(Self {
                    instance: instance,
                    _instance_functions: instance_functions,
                    vkDestroyInstance: get_instance_fn!(
                        vk_get_instance_proc_addr,
                        instance,
                        vkDestroyInstance
                    ),
                    vkCreateDevice: get_instance_fn!(
                        vk_get_instance_proc_addr,
                        instance,
                        vkCreateDevice
                    ),
                    vkGetDeviceProcAddr: get_instance_fn!(
                        vk_get_instance_proc_addr,
                        instance,
                        vkGetDeviceProcAddr
                    ),
                    vkDestroySurfaceKHR: get_instance_fn!(
                        vk_get_instance_proc_addr,
                        instance,
                        vkDestroySurfaceKHR
                    ),
                    vkEnumeratePhysicalDevices: get_instance_fn!(
                        vk_get_instance_proc_addr,
                        instance,
                        vkEnumeratePhysicalDevices
                    ),
                    vkGetPhysicalDeviceSurfaceSupportKHR: get_instance_fn!(
                        vk_get_instance_proc_addr,
                        instance,
                        vkGetPhysicalDeviceSurfaceSupportKHR
                    ),
                    vkGetPhysicalDeviceQueueFamilyProperties: get_instance_fn!(
                        vk_get_instance_proc_addr,
                        instance,
                        vkGetPhysicalDeviceQueueFamilyProperties
                    ),
                    vkEnumerateDeviceExtensionProperties: get_instance_fn!(
                        vk_get_instance_proc_addr,
                        instance,
                        vkEnumerateDeviceExtensionProperties
                    ),
                })
            }
            result => Err(VulkanError::VulkanError(result)),
        }
    }
}

impl Drop for InstanceWrapper {
    fn drop(&mut self) {
        unsafe {
            self.vkDestroyInstance.unwrap()(self.instance, null());
        }
    }
}

pub unsafe fn get_device_fn(
    vk_get_device_proc_addr: api::PFN_vkGetDeviceProcAddr,
    device: api::VkDevice,
    name: &[u8],
) -> api::PFN_vkVoidFunction {
    let name = CStr::from_bytes_with_nul(name).unwrap();
    match vk_get_device_proc_addr.unwrap()(device, name.as_ptr()) {
        Some(retval) => Some(retval),
        None => panic!(
            "vkGetDeviceProcAddr failed: function not found: {}",
            name.to_string_lossy()
        ),
    }
}

macro_rules! get_device_fn {
    ($vk_get_device_proc_addr:expr, $device:expr, $name:ident) => {{
        use self::api::*;
        mem::transmute::<api::PFN_vkVoidFunction, concat_idents!(PFN_, $name)>(self::get_device_fn(
            $vk_get_device_proc_addr,
            $device,
            concat!(stringify!($name), "\0").as_bytes(),
        ))
    }};
}

#[allow(non_snake_case)]
struct DeviceWrapper {
    device: api::VkDevice,
    _instance: Arc<InstanceWrapper>,
    vkDestroyDevice: api::PFN_vkDestroyDevice,
    vkDeviceWaitIdle: api::PFN_vkDeviceWaitIdle,
    vkWaitForFences: api::PFN_vkWaitForFences,
    vkCreateFence: api::PFN_vkCreateFence,
    vkDestroyFence: api::PFN_vkDestroyFence,
    vkGetDeviceQueue: api::PFN_vkGetDeviceQueue,
}

unsafe impl Sync for DeviceWrapper {}
unsafe impl Send for DeviceWrapper {}

impl DeviceWrapper {
    pub unsafe fn new(
        instance: Arc<InstanceWrapper>,
        physical_device: api::VkPhysicalDevice,
        queue_create_infos: &[api::VkDeviceQueueCreateInfo],
        enabled_extension_names: &[*const c_char],
        enabled_features: Option<&api::VkPhysicalDeviceFeatures>,
    ) -> Result<Self> {
        let mut device = null_mut();
        match instance.vkCreateDevice.unwrap()(
            physical_device,
            &api::VkDeviceCreateInfo {
                sType: api::VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO,
                pNext: null(),
                flags: 0,
                queueCreateInfoCount: queue_create_infos.len() as u32,
                pQueueCreateInfos: queue_create_infos.as_ptr(),
                enabledLayerCount: 0,
                ppEnabledLayerNames: null(),
                enabledExtensionCount: enabled_extension_names.len() as u32,
                ppEnabledExtensionNames: enabled_extension_names.as_ptr(),
                pEnabledFeatures: match enabled_features {
                    Some(v) => v,
                    None => null(),
                },
            },
            null(),
            &mut device,
        ) {
            api::VK_SUCCESS => {
                let vk_get_device_proc_addr = instance.vkGetDeviceProcAddr;
                Ok(Self {
                    device: device,
                    _instance: instance,
                    vkDestroyDevice: get_device_fn!(
                        vk_get_device_proc_addr,
                        device,
                        vkDestroyDevice
                    ),
                    vkDeviceWaitIdle: get_device_fn!(
                        vk_get_device_proc_addr,
                        device,
                        vkDeviceWaitIdle
                    ),
                    vkWaitForFences: get_device_fn!(
                        vk_get_device_proc_addr,
                        device,
                        vkWaitForFences
                    ),
                    vkCreateFence: get_device_fn!(vk_get_device_proc_addr, device, vkCreateFence),
                    vkDestroyFence: get_device_fn!(vk_get_device_proc_addr, device, vkDestroyFence),
                    vkGetDeviceQueue: get_device_fn!(
                        vk_get_device_proc_addr,
                        device,
                        vkGetDeviceQueue
                    ),
                })
            }
            result => Err(VulkanError::VulkanError(result)),
        }
    }
}

impl Drop for DeviceWrapper {
    fn drop(&mut self) {
        unsafe {
            self.vkDeviceWaitIdle.unwrap()(self.device);
            self.vkDestroyDevice.unwrap()(self.device, null());
        }
    }
}

struct SurfaceWrapper {
    window: sdl::window::Window,
    instance: Arc<InstanceWrapper>,
    surface: api::VkSurfaceKHR,
}

impl Drop for SurfaceWrapper {
    fn drop(&mut self) {
        unsafe {
            self.instance.vkDestroySurfaceKHR.unwrap()(
                self.instance.instance,
                self.surface,
                null(),
            );
        }
    }
}

impl SurfaceWrapper {
    unsafe fn new(window: sdl::window::Window, instance: Arc<InstanceWrapper>) -> Result<Self> {
        let mut surface = null_mut();
        if sdl::api::SDL_Vulkan_CreateSurface(
            window.get(),
            instance.instance as sdl::api::VkInstance,
            &mut surface,
        ) == 0
        {
            Err(sdl::get_error().into())
        } else {
            Ok(Self {
                window: window,
                instance: instance,
                surface: surface as api::VkSurfaceKHR,
            })
        }
    }
}

#[derive(Clone)]
pub struct VulkanDeviceReference {
    device: Arc<DeviceWrapper>,
}

struct SurfaceState {
    surface: SurfaceWrapper,
    present_queue_index: u32,
    render_queue_index: u32,
    physical_device: api::VkPhysicalDevice,
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

pub enum VulkanError {
    VulkanError(api::VkResult),
    SDLError(sdl::SDLError),
    NoMatchingPhysicalDevice,
}

impl From<sdl::SDLError> for VulkanError {
    fn from(v: sdl::SDLError) -> Self {
        VulkanError::SDLError(v)
    }
}

impl fmt::Display for VulkanError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            VulkanError::VulkanError(result) => {
                let name = match *result {
                    api::VK_SUCCESS => "VK_SUCCESS",
                    api::VK_NOT_READY => "VK_NOT_READY",
                    api::VK_TIMEOUT => "VK_TIMEOUT",
                    api::VK_EVENT_SET => "VK_EVENT_SET",
                    api::VK_EVENT_RESET => "VK_EVENT_RESET",
                    api::VK_INCOMPLETE => "VK_INCOMPLETE",
                    api::VK_ERROR_OUT_OF_HOST_MEMORY => "VK_ERROR_OUT_OF_HOST_MEMORY",
                    api::VK_ERROR_OUT_OF_DEVICE_MEMORY => "VK_ERROR_OUT_OF_DEVICE_MEMORY",
                    api::VK_ERROR_INITIALIZATION_FAILED => "VK_ERROR_INITIALIZATION_FAILED",
                    api::VK_ERROR_DEVICE_LOST => "VK_ERROR_DEVICE_LOST",
                    api::VK_ERROR_MEMORY_MAP_FAILED => "VK_ERROR_MEMORY_MAP_FAILED",
                    api::VK_ERROR_LAYER_NOT_PRESENT => "VK_ERROR_LAYER_NOT_PRESENT",
                    api::VK_ERROR_EXTENSION_NOT_PRESENT => "VK_ERROR_EXTENSION_NOT_PRESENT",
                    api::VK_ERROR_FEATURE_NOT_PRESENT => "VK_ERROR_FEATURE_NOT_PRESENT",
                    api::VK_ERROR_INCOMPATIBLE_DRIVER => "VK_ERROR_INCOMPATIBLE_DRIVER",
                    api::VK_ERROR_TOO_MANY_OBJECTS => "VK_ERROR_TOO_MANY_OBJECTS",
                    api::VK_ERROR_FORMAT_NOT_SUPPORTED => "VK_ERROR_FORMAT_NOT_SUPPORTED",
                    api::VK_ERROR_FRAGMENTED_POOL => "VK_ERROR_FRAGMENTED_POOL",
                    api::VK_ERROR_SURFACE_LOST_KHR => "VK_ERROR_SURFACE_LOST_KHR",
                    api::VK_ERROR_NATIVE_WINDOW_IN_USE_KHR => "VK_ERROR_NATIVE_WINDOW_IN_USE_KHR",
                    api::VK_SUBOPTIMAL_KHR => "VK_SUBOPTIMAL_KHR",
                    api::VK_ERROR_OUT_OF_DATE_KHR => "VK_ERROR_OUT_OF_DATE_KHR",
                    api::VK_ERROR_INCOMPATIBLE_DISPLAY_KHR => "VK_ERROR_INCOMPATIBLE_DISPLAY_KHR",
                    api::VK_ERROR_VALIDATION_FAILED_EXT => "VK_ERROR_VALIDATION_FAILED_EXT",
                    api::VK_ERROR_INVALID_SHADER_NV => "VK_ERROR_INVALID_SHADER_NV",
                    api::VK_ERROR_NOT_PERMITTED_EXT => "VK_ERROR_NOT_PERMITTED_EXT",
                    api::VK_ERROR_OUT_OF_POOL_MEMORY_KHR => "VK_ERROR_OUT_OF_POOL_MEMORY_KHR",
                    api::VK_ERROR_INVALID_EXTERNAL_HANDLE_KHR => {
                        "VK_ERROR_INVALID_EXTERNAL_HANDLE_KHR"
                    }
                    result => return write!(f, "<unknown VkResult: {}>", result),
                };
                f.write_str(name)
            }
            VulkanError::SDLError(error) => (error as &fmt::Display).fmt(f),
            VulkanError::NoMatchingPhysicalDevice => f.write_str("no matching physical device"),
        }
    }
}

impl fmt::Debug for VulkanError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            VulkanError::SDLError(error) => (error as &fmt::Debug).fmt(f),
            VulkanError::VulkanError(_) => (self as &fmt::Display).fmt(f),
            VulkanError::NoMatchingPhysicalDevice => f.write_str("NoMatchingPhysicalDevice"),
        }
    }
}

impl Error for VulkanError {}

pub type Result<T> = result::Result<T, VulkanError>;

impl DeviceReference for VulkanDeviceReference {
    type Semaphore = VulkanSemaphore;
    type Fence = VulkanFence;
    type Error = VulkanError;
    fn create_fence(&self, initial_state: FenceState) -> Result<VulkanFence> {
        let mut fence = 0;
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
    fn get_window(&self) -> &sdl::window::Window {
        &self.surface_state.surface.window
    }
}

impl Device for VulkanDevice {
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
            present_queue_index,
            render_queue_index,
            physical_device,
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
                present_queue_index: present_queue_index,
                render_queue_index: render_queue_index,
                physical_device: physical_device,
            },
            queue: render_queue,
            present_queue: present_queue,
        });
    }
    fn get_window(&self) -> &sdl::window::Window {
        &self.surface_state.surface.window
    }
    fn get_device_ref(&self) -> VulkanDeviceReference {
        self.device_reference.clone()
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
            final_fences.push(fence.fence);
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
    fn create<T: Into<String>>(
        &self,
        title: T,
        position: Option<(i32, i32)>,
        size: (u32, u32),
        mut flags: u32,
    ) -> Result<VulkanDevice> {
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
        let layers = [];
        let instance = unsafe {
            InstanceWrapper::new(
                instance_functions,
                &api::VkApplicationInfo {
                    sType: api::VK_STRUCTURE_TYPE_APPLICATION_INFO,
                    pNext: null(),
                    pApplicationName: null(),
                    applicationVersion: 0,
                    pEngineName: null(),
                    engineVersion: 0,
                    apiVersion: 0,
                },
                &layers,
                &extensions,
            )
        }?;
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
        for physical_device in physical_devices {
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
            ) {
                (Some(present_queue_index), Some(render_queue_index), true) => {
                    return VulkanDevice::resume(VulkanPausedDevice {
                        surface_state: SurfaceState {
                            surface: surface,
                            present_queue_index: present_queue_index,
                            render_queue_index: render_queue_index,
                            physical_device: physical_device,
                        },
                    });
                }
                _ => continue,
            }
        }
        Err(VulkanError::NoMatchingPhysicalDevice)
    }
}
