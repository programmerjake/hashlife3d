use super::api;
use sdl;
use std::error::Error;
use std::fmt;
use std::result;

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
