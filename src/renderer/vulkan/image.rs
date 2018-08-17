// This file is part of Hashlife3d.
//
// Hashlife3d is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Hashlife3d is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with Hashlife3d.  If not, see <https://www.gnu.org/licenses/>
use super::{
    api, null_or_zero, BufferWrapper, CommandBufferSubmitTracker, DescriptorPoolWrapper,
    DescriptorSetLayoutWrapper, DescriptorSetWrapper, DeviceImageSet, DeviceMemoryPoolAllocation,
    DeviceMemoryPools, DeviceWrapper, Result, StagingImageSet, TextureIndex, VulkanError,
    FRAGMENT_SAMPLERS_BINDING, FRAGMENT_SAMPLERS_BINDING_DESCRIPTOR_COUNT,
};
use renderer::image::{Image, Pixel};
use renderer::math;
use std::mem;
use std::ptr::{copy_nonoverlapping, null, NonNull};
use std::sync::Arc;

pub struct ImageWrapper {
    pub device: Arc<DeviceWrapper>,
    pub image: api::VkImage,
    pub destroy_on_drop: bool,
    pub device_memory: Option<DeviceMemoryPoolAllocation>,
}

impl ImageWrapper {
    pub fn new_depth(
        device: Arc<DeviceWrapper>,
        format: api::VkFormat,
        width: u32,
        height: u32,
    ) -> Result<Self> {
        let mut image = null_or_zero();
        match unsafe {
            device.vkCreateImage.unwrap()(
                device.device,
                &api::VkImageCreateInfo {
                    sType: api::VK_STRUCTURE_TYPE_IMAGE_CREATE_INFO,
                    pNext: null(),
                    flags: 0,
                    imageType: api::VK_IMAGE_TYPE_2D,
                    format: format,
                    extent: api::VkExtent3D {
                        width: width,
                        height: height,
                        depth: 1,
                    },
                    mipLevels: 1,
                    arrayLayers: 1,
                    samples: api::VK_SAMPLE_COUNT_1_BIT,
                    tiling: api::VK_IMAGE_TILING_OPTIMAL,
                    usage: api::VK_IMAGE_USAGE_DEPTH_STENCIL_ATTACHMENT_BIT,
                    sharingMode: api::VK_SHARING_MODE_EXCLUSIVE,
                    queueFamilyIndexCount: 0,
                    pQueueFamilyIndices: null(),
                    initialLayout: api::VK_IMAGE_LAYOUT_UNDEFINED,
                },
                null(),
                &mut image,
            )
        } {
            api::VK_SUCCESS => Ok(ImageWrapper {
                device: device,
                image: image,
                destroy_on_drop: true,
                device_memory: None,
            }),
            result => Err(VulkanError::VulkanError(result)),
        }
    }
    pub fn new_image_set_member(
        device: Arc<DeviceWrapper>,
        width: u32,
        height: u32,
        layers: u32,
    ) -> Result<Self> {
        let mut image = null_or_zero();
        match unsafe {
            device.vkCreateImage.unwrap()(
                device.device,
                &api::VkImageCreateInfo {
                    sType: api::VK_STRUCTURE_TYPE_IMAGE_CREATE_INFO,
                    pNext: null(),
                    flags: IMAGE_SET_IMAGE_CREATE_FLAGS,
                    imageType: IMAGE_SET_IMAGE_TYPE,
                    format: IMAGE_SET_FORMAT,
                    extent: api::VkExtent3D {
                        width: width,
                        height: height,
                        depth: 1,
                    },
                    mipLevels: 1,
                    arrayLayers: layers,
                    samples: api::VK_SAMPLE_COUNT_1_BIT,
                    tiling: IMAGE_SET_IMAGE_TILING,
                    usage: IMAGE_SET_IMAGE_USAGE,
                    sharingMode: api::VK_SHARING_MODE_EXCLUSIVE,
                    queueFamilyIndexCount: 0,
                    pQueueFamilyIndices: null(),
                    initialLayout: api::VK_IMAGE_LAYOUT_UNDEFINED,
                },
                null(),
                &mut image,
            )
        } {
            api::VK_SUCCESS => Ok(ImageWrapper {
                device: device,
                image: image,
                destroy_on_drop: true,
                device_memory: None,
            }),
            result => Err(VulkanError::VulkanError(result)),
        }
    }
    pub unsafe fn allocate_and_bind_memory(
        mut self,
        device_memory_pools: &DeviceMemoryPools,
        preferred_properties: Option<api::VkMemoryPropertyFlags>,
        required_properties: api::VkMemoryPropertyFlags,
    ) -> Result<Self> {
        assert!(self.device_memory.is_none());
        let mut memory_requirements = mem::zeroed();
        self.device.vkGetImageMemoryRequirements.unwrap()(
            self.device.device,
            self.image,
            &mut memory_requirements,
        );
        let memory_allocation = device_memory_pools.allocate_from_memory_requirements(
            memory_requirements,
            preferred_properties,
            required_properties,
        )?;
        match self.device.vkBindImageMemory.unwrap()(
            self.device.device,
            self.image,
            memory_allocation.get_device_memory().get_device_memory(),
            memory_allocation.get_offset(),
        ) {
            api::VK_SUCCESS => {
                self.device_memory = Some(memory_allocation);
                Ok(self)
            }
            result => Err(VulkanError::VulkanError(result)),
        }
    }
}

impl Drop for ImageWrapper {
    fn drop(&mut self) {
        if self.destroy_on_drop {
            unsafe {
                self.device.vkDestroyImage.unwrap()(self.device.device, self.image, null());
            }
        }
    }
}

pub struct ImageViewWrapper {
    pub image: Arc<ImageWrapper>,
    pub image_view: api::VkImageView,
}

impl Drop for ImageViewWrapper {
    fn drop(&mut self) {
        unsafe {
            self.image.device.vkDestroyImageView.unwrap()(
                self.image.device.device,
                self.image_view,
                null(),
            );
        }
    }
}

pub struct SamplerWrapper {
    pub device: Arc<DeviceWrapper>,
    pub sampler: api::VkSampler,
}

impl Drop for SamplerWrapper {
    fn drop(&mut self) {
        unsafe {
            self.device.vkDestroySampler.unwrap()(self.device.device, self.sampler, null());
        }
    }
}

impl SamplerWrapper {
    pub unsafe fn new(device: Arc<DeviceWrapper>) -> Result<Self> {
        let mut sampler = null_or_zero();
        match device.vkCreateSampler.unwrap()(
            device.device,
            &api::VkSamplerCreateInfo {
                sType: api::VK_STRUCTURE_TYPE_SAMPLER_CREATE_INFO,
                pNext: null(),
                flags: 0,
                magFilter: api::VK_FILTER_NEAREST,
                minFilter: api::VK_FILTER_NEAREST,
                mipmapMode: api::VK_SAMPLER_MIPMAP_MODE_NEAREST,
                addressModeU: api::VK_SAMPLER_ADDRESS_MODE_CLAMP_TO_EDGE,
                addressModeV: api::VK_SAMPLER_ADDRESS_MODE_CLAMP_TO_EDGE,
                addressModeW: api::VK_SAMPLER_ADDRESS_MODE_CLAMP_TO_EDGE,
                mipLodBias: 0.0,
                anisotropyEnable: api::VK_FALSE,
                maxAnisotropy: 0.0,
                compareEnable: api::VK_FALSE,
                compareOp: api::VK_COMPARE_OP_ALWAYS,
                minLod: 0.0,
                maxLod: 0.0,
                borderColor: api::VK_BORDER_COLOR_FLOAT_TRANSPARENT_BLACK,
                unnormalizedCoordinates: api::VK_FALSE,
            },
            null(),
            &mut sampler,
        ) {
            api::VK_SUCCESS => Ok(Self {
                device: device,
                sampler: sampler,
            }),
            result => Err(VulkanError::VulkanError(result)),
        }
    }
}

pub struct VulkanStagingImageSetImplementation {
    pub buffer: BufferWrapper,
    pub buffer_allocation: DeviceMemoryPoolAllocation,
    pub device_image_set: VulkanDeviceImageSetImplementation,
    pub mapped_memory: NonNull<[u8]>,
}

unsafe impl Send for VulkanStagingImageSetImplementation {}

pub struct VulkanStagingImageSet(VulkanStagingImageSetImplementation);

pub fn into_vulkan_staging_image_set_implementation(
    v: VulkanStagingImageSet,
) -> VulkanStagingImageSetImplementation {
    v.0
}

pub const IMAGE_SET_FORMAT: api::VkFormat = api::VK_FORMAT_R8G8B8A8_SRGB;

#[allow(dead_code)]
fn assert_pixel_is_vec4_u8(v: &Pixel) -> &math::Vec4<u8> {
    v
}

pub const IMAGE_SET_IMAGE_TYPE: api::VkImageType = api::VK_IMAGE_TYPE_2D;
pub const IMAGE_SET_IMAGE_TILING: api::VkImageTiling = api::VK_IMAGE_TILING_OPTIMAL;
pub const IMAGE_SET_IMAGE_USAGE: api::VkImageUsageFlags =
    api::VK_IMAGE_USAGE_TRANSFER_DST_BIT | api::VK_IMAGE_USAGE_SAMPLED_BIT;
pub const IMAGE_SET_IMAGE_CREATE_FLAGS: api::VkImageCreateFlags = 0;

pub fn get_image_set_max_total_layer_count(
    image_set_image_format_properties: &api::VkImageFormatProperties,
    width: u32,
    height: u32,
) -> Result<u32> {
    if !width.is_power_of_two() || !height.is_power_of_two() {
        return Err(VulkanError::ImageMustHavePowerOfTwoDimensions);
    }
    if width > image_set_image_format_properties.maxExtent.width
        || height > image_set_image_format_properties.maxExtent.height
    {
        return Err(VulkanError::ImageIsTooBig);
    }
    let max = TextureIndex::max_value() as u32;
    match image_set_image_format_properties
        .maxArrayLayers
        .checked_mul(FRAGMENT_SAMPLERS_BINDING_DESCRIPTOR_COUNT)
    {
        Some(retval) if retval <= max => Ok(retval),
        _ => Ok(max),
    }
}

pub unsafe fn create_staging_image_set(
    device: Arc<DeviceWrapper>,
    device_memory_pools: &DeviceMemoryPools,
    image_set_image_format_properties: &api::VkImageFormatProperties,
    width: u32,
    height: u32,
    total_layer_count: u32,
    samplers_descriptor_set_layout: Arc<DescriptorSetLayoutWrapper>,
) -> Result<VulkanStagingImageSet> {
    if total_layer_count
        > get_image_set_max_total_layer_count(image_set_image_format_properties, width, height)?
    {
        return Err(VulkanError::ImageSetHasTooManyImages);
    }
    let image_layer_count = image_set_image_format_properties.maxArrayLayers;
    let last_image_layer_count = total_layer_count % image_layer_count;
    let valid_image_count =
        total_layer_count / image_layer_count + if last_image_layer_count != 0 { 1 } else { 0 };
    assert!(valid_image_count <= FRAGMENT_SAMPLERS_BINDING_DESCRIPTOR_COUNT);
    let size = width as api::VkDeviceSize
        * height as api::VkDeviceSize
        * total_layer_count as api::VkDeviceSize
        * mem::size_of::<Pixel>() as api::VkDeviceSize;
    let (buffer, buffer_allocation) = BufferWrapper::new(
        device.clone(),
        size,
        api::VK_BUFFER_USAGE_TRANSFER_SRC_BIT,
        api::VK_SHARING_MODE_EXCLUSIVE,
        &[],
    )?.allocate_and_bind_memory(
        device_memory_pools,
        mem::align_of::<Pixel>(),
        None,
        api::VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | api::VK_MEMORY_PROPERTY_HOST_COHERENT_BIT,
    )?;
    let mut images = Vec::new();
    let mut samplers = Vec::new();
    let mut descriptor_image_infos = Vec::new();
    for i in 0..FRAGMENT_SAMPLERS_BINDING_DESCRIPTOR_COUNT {
        let image = Arc::new(
            ImageWrapper::new_image_set_member(
                device.clone(),
                width,
                height,
                if i >= valid_image_count {
                    1
                } else if i + 1 == valid_image_count {
                    last_image_layer_count
                } else {
                    image_layer_count
                },
            )?.allocate_and_bind_memory(
                device_memory_pools,
                None,
                api::VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT,
            )?,
        );
        let mut image_view = null_or_zero();
        let image_view = match device.vkCreateImageView.unwrap()(
            device.device,
            &api::VkImageViewCreateInfo {
                sType: api::VK_STRUCTURE_TYPE_IMAGE_VIEW_CREATE_INFO,
                pNext: null(),
                flags: 0,
                image: image.image,
                viewType: api::VK_IMAGE_VIEW_TYPE_2D_ARRAY,
                format: IMAGE_SET_FORMAT,
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
                    layerCount: api::VK_REMAINING_ARRAY_LAYERS as u32,
                },
            },
            null(),
            &mut image_view,
        ) {
            api::VK_SUCCESS => ImageViewWrapper {
                image: image,
                image_view: image_view,
            },
            result => return Err(VulkanError::VulkanError(result)),
        };
        let sampler = SamplerWrapper::new(device.clone())?;
        descriptor_image_infos.push(api::VkDescriptorImageInfo {
            sampler: sampler.sampler,
            imageView: image_view.image_view,
            imageLayout: api::VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL,
        });
        images.push(image_view);
        samplers.push(sampler);
    }
    let descriptor_set = DescriptorSetWrapper::new(
        DescriptorPoolWrapper::new(
            device,
            1,
            &[api::VkDescriptorPoolSize {
                type_: api::VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER,
                descriptorCount: FRAGMENT_SAMPLERS_BINDING_DESCRIPTOR_COUNT,
            }],
        )?,
        samplers_descriptor_set_layout,
    )?;
    {
        let device = &descriptor_set.descriptor_pool.device;
        assert_eq!(
            descriptor_image_infos.len(),
            FRAGMENT_SAMPLERS_BINDING_DESCRIPTOR_COUNT as usize
        );
        let descriptor_writes = [api::VkWriteDescriptorSet {
            sType: api::VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET,
            pNext: null(),
            dstSet: descriptor_set.descriptor_set,
            dstBinding: FRAGMENT_SAMPLERS_BINDING,
            dstArrayElement: 0,
            descriptorCount: descriptor_image_infos.len() as u32,
            descriptorType: api::VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER,
            pImageInfo: descriptor_image_infos.as_ptr(),
            pBufferInfo: null(),
            pTexelBufferView: null(),
        }];
        device.vkUpdateDescriptorSets.unwrap()(
            device.device,
            descriptor_writes.len() as u32,
            descriptor_writes.as_ptr(),
            0,
            null(),
        );
    }
    let mapped_memory = buffer_allocation.get_mapped_memory().unwrap();
    Ok(VulkanStagingImageSet(VulkanStagingImageSetImplementation {
        buffer: buffer,
        buffer_allocation: buffer_allocation,
        device_image_set: VulkanDeviceImageSetImplementation {
            images: Arc::new(images),
            submit_tracker: None,
            width: width,
            height: height,
            total_layer_count: total_layer_count,
            image_layer_count: image_layer_count,
            last_image_layer_count: last_image_layer_count,
            valid_image_count: valid_image_count,
            samplers: Arc::new(samplers),
            descriptor_set: Arc::new(descriptor_set),
        },
        mapped_memory: mapped_memory,
    }))
}

impl StagingImageSet for VulkanStagingImageSet {
    fn width(&self) -> u32 {
        self.0.device_image_set.width
    }
    fn height(&self) -> u32 {
        self.0.device_image_set.height
    }
    fn count(&self) -> u32 {
        self.0.device_image_set.total_layer_count
    }
    fn write(&mut self, image_index: TextureIndex, image: &Image) {
        assert_ne!(image_index, 0);
        let image_index = (image_index as usize) - 1;
        assert!(image_index < self.0.device_image_set.total_layer_count as usize);
        let width = self.0.device_image_set.width;
        let height = self.0.device_image_set.height;
        assert!(image.width() == width);
        assert!(image.height() == height);
        unsafe {
            let image_size = width as usize * height as usize * mem::size_of::<Pixel>();
            let src = (image.get_pixels() as &[Pixel]).as_ptr() as *const u8;
            let dest = self.0.mapped_memory.as_mut()[(image_size * image_index)..][..image_size]
                .as_mut_ptr();
            copy_nonoverlapping(src, dest, image_size);
        }
    }
}

#[derive(Clone)]
pub struct VulkanDeviceImageSetImplementation {
    pub images: Arc<Vec<ImageViewWrapper>>,
    pub submit_tracker: Option<CommandBufferSubmitTracker>,
    pub width: u32,
    pub height: u32,
    pub total_layer_count: u32,
    pub image_layer_count: u32,
    pub last_image_layer_count: u32,
    pub valid_image_count: u32,
    pub samplers: Arc<Vec<SamplerWrapper>>,
    pub descriptor_set: Arc<DescriptorSetWrapper>,
}

#[derive(Clone)]
pub struct VulkanDeviceImageSet(VulkanDeviceImageSetImplementation);

pub fn create_device_image_set(v: VulkanDeviceImageSetImplementation) -> VulkanDeviceImageSet {
    VulkanDeviceImageSet(v)
}

pub fn into_vulkan_device_image_set_implementation(
    v: VulkanDeviceImageSet,
) -> VulkanDeviceImageSetImplementation {
    v.0
}

impl DeviceImageSet for VulkanDeviceImageSet {
    fn width(&self) -> u32 {
        self.0.width
    }
    fn height(&self) -> u32 {
        self.0.height
    }
    fn count(&self) -> u32 {
        self.0.total_layer_count
    }
}
