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
extern crate num_traits;
extern crate voxels_image as image;
extern crate voxels_math as math;
extern crate voxels_sdl as sdl;
use math::Mappable;
use std::error;
use std::fmt;
use std::marker::PhantomData;
use std::ops;
use std::u8;

/// for N textures, ranges from 1 to N, with 0 reserved for not using a texture
pub type TextureId = u16;

pub const NO_TEXTURE: TextureId = 0;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct VertexBufferElement {
    pub position: [f32; 3],
    pub color: [u8; 4],
    pub texture_coord: [f32; 2],
    pub texture_id: TextureId,
}

impl VertexBufferElement {
    pub fn new(
        position: math::Vec3,
        color: math::Vec4<u8>,
        texture_coord: math::Vec2,
        texture_id: TextureId,
    ) -> Self {
        Self {
            position: position.into(),
            color: color.into(),
            texture_coord: texture_coord.into(),
            texture_id: texture_id,
        }
    }
}

impl fmt::Debug for VertexBufferElement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let position = math::Vec3::new(self.position[0], self.position[1], self.position[2]);
        let color = math::Vec4::new(self.color[0], self.color[1], self.color[2], self.color[3]);
        let texture_coord = math::Vec2::new(self.texture_coord[0], self.texture_coord[1]);
        struct TextureIdWrapper(TextureId);
        impl fmt::Debug for TextureIdWrapper {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                if self.0 == NO_TEXTURE {
                    f.write_str("NO_TEXTURE")
                } else {
                    self.0.fmt(f)
                }
            }
        }
        f.debug_struct("VertexBufferElement")
            .field("position", &position)
            .field("color", &color.map(|v| v as f32 / u8::MAX as f32))
            .field("texture_coord", &texture_coord)
            .field("texture_id", &TextureIdWrapper(self.texture_id))
            .finish()
    }
}

pub type IndexBufferElement = u16;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum FenceTryWaitResult {
    Ready,
    WouldBlock,
}

pub trait Fence: Send + Sync + Sized + Clone {
    type Error: error::Error + Send + Sync + 'static;
    fn try_wait(&self) -> Result<FenceTryWaitResult, Self::Error>;
    fn wait(self) -> Result<(), Self::Error>;
}

pub trait GenericArray<Element: Send + Sync + Clone + 'static>: Send + Sync + Sized {
    fn len(&self) -> usize;
    fn slice<Range: ops::RangeBounds<usize>>(self, range: Range) -> Slice<Element, Self> {
        Slice::new(self).slice(range)
    }
    fn slice_ref<Range: ops::RangeBounds<usize>>(&self, range: Range) -> Slice<Element, &Self> {
        Slice::new(&*self).slice(range)
    }
    fn slice_mut<Range: ops::RangeBounds<usize>>(
        &mut self,
        range: Range,
    ) -> Slice<Element, &mut Self> {
        Slice::new(self).slice(range)
    }
}

impl<'a, Element: Send + Sync + Clone + 'static, T: GenericArray<Element>> GenericArray<Element>
    for &'a T
{
    fn len(&self) -> usize {
        (**self).len()
    }
}

impl<'a, Element: Send + Sync + Clone + 'static, T: GenericArray<Element>> GenericArray<Element>
    for &'a mut T
{
    fn len(&self) -> usize {
        (**self).len()
    }
}

#[derive(Copy, Clone, Debug)]
pub struct UnbackedGenericArray<Element: Send + Sync + Clone + 'static> {
    len: usize,
    _elements: PhantomData<&'static Element>,
}

impl<Element: Send + Sync + Clone + 'static> UnbackedGenericArray<Element> {
    pub fn new(len: usize) -> Self {
        Self {
            len: len,
            _elements: PhantomData,
        }
    }
}

impl<'a, Element: Send + Sync + Clone + 'static, T: GenericArray<Element>> From<&'a T>
    for UnbackedGenericArray<Element>
{
    fn from(generic_array: &T) -> Self {
        UnbackedGenericArray::new(generic_array.len())
    }
}

impl<Element: Send + Sync + Clone + 'static> GenericArray<Element>
    for UnbackedGenericArray<Element>
{
    fn len(&self) -> usize {
        self.len
    }
}

pub trait UninitializedDeviceGenericArray<Element: Send + Sync + Clone + 'static>:
    GenericArray<Element>
{
}

pub trait DeviceGenericArray<Element: Send + Sync + Clone + 'static>:
    GenericArray<Element>
{
}

mod staging_lock_guards {
    use super::*;

    pub trait StagingReadLockGuardImplementation {
        type Element: Send + Sync + Clone + 'static;
        unsafe fn get(&self) -> *const [Self::Element];
    }

    pub trait StagingWriteLockGuardImplementation {
        type Element: Send + Sync + Clone + 'static;
        unsafe fn get(&self) -> *const [Self::Element];
        unsafe fn get_mut(&mut self) -> *mut [Self::Element];
    }

    pub struct StagingReadLockGuard<'a, I: StagingReadLockGuardImplementation> {
        implementation: I,
        start: usize,
        len: usize,
        _phantom: PhantomData<&'a [I::Element]>,
    }

    pub struct StagingWriteLockGuard<'a, I: StagingWriteLockGuardImplementation> {
        implementation: I,
        start: usize,
        len: usize,
        _phantom: PhantomData<&'a mut [I::Element]>,
    }

    impl<'a, I: StagingReadLockGuardImplementation> StagingReadLockGuard<'a, I> {
        pub unsafe fn new(implementation: I) -> Self {
            let len = (*implementation.get()).len();
            Self {
                implementation: implementation,
                start: 0,
                len: len,
                _phantom: PhantomData,
            }
        }
        pub fn slice(mut this: Self, start: usize, len: usize) -> Self {
            assert!(start <= this.len);
            assert!(len <= this.len);
            assert!(start + len <= this.len);
            this.start += start;
            this.len = len;
            this
        }
        fn get(&self) -> &'a [I::Element] {
            unsafe { &(*self.implementation.get())[self.start..][..self.len] }
        }
    }

    impl<'a, I: StagingWriteLockGuardImplementation> StagingWriteLockGuard<'a, I> {
        pub unsafe fn new(implementation: I) -> Self {
            let len = (*implementation.get()).len();
            Self {
                implementation: implementation,
                start: 0,
                len: len,
                _phantom: PhantomData,
            }
        }
        pub fn slice(mut this: Self, start: usize, len: usize) -> Self {
            assert!(start <= this.len);
            assert!(len <= this.len);
            assert!(start + len <= this.len);
            this.start += start;
            this.len = len;
            this
        }
        fn get(&self) -> &'a [I::Element] {
            unsafe { &(*self.implementation.get())[self.start..][..self.len] }
        }
        fn get_mut(&mut self) -> &'a mut [I::Element] {
            unsafe { &mut (*self.implementation.get_mut())[self.start..][..self.len] }
        }
    }

    impl<'a, I: StagingReadLockGuardImplementation> ops::Deref for StagingReadLockGuard<'a, I> {
        type Target = [I::Element];
        fn deref(&self) -> &[I::Element] {
            self.get()
        }
    }

    impl<'a, I: StagingWriteLockGuardImplementation> ops::Deref for StagingWriteLockGuard<'a, I> {
        type Target = [I::Element];
        fn deref(&self) -> &[I::Element] {
            self.get()
        }
    }

    impl<'a, I: StagingWriteLockGuardImplementation> ops::DerefMut for StagingWriteLockGuard<'a, I> {
        fn deref_mut(&mut self) -> &mut [I::Element] {
            self.get_mut()
        }
    }
}

pub use staging_lock_guards::*;

pub trait StagingGenericArray<Element: Send + Sync + Clone + 'static>:
    GenericArray<Element>
{
    type ReadLockGuardImplementation: StagingReadLockGuardImplementation<Element = Element>;
    type WriteLockGuardImplementation: StagingWriteLockGuardImplementation<Element = Element>;
    fn read<'a>(&'a self) -> StagingReadLockGuard<'a, Self::ReadLockGuardImplementation>;
    fn write<'a>(&'a self) -> StagingWriteLockGuard<'a, Self::WriteLockGuardImplementation>;
}

pub trait Buffer<Element: Send + Sync + Copy + 'static>: GenericArray<Element> {}

pub trait ImageSet: GenericArray<image::Image> {
    fn dimensions(&self) -> math::Vec2<u32>;
}

pub trait UninitializedDeviceImageSet:
    ImageSet + UninitializedDeviceGenericArray<image::Image>
{
}

pub trait DeviceImageSet: ImageSet + DeviceGenericArray<image::Image> {}

pub trait StagingImageSet: ImageSet + StagingGenericArray<image::Image> {}

pub trait DeviceBuffer<Element: Send + Sync + Copy + 'static>:
    Buffer<Element> + DeviceGenericArray<Element>
{
}

pub trait UninitializedDeviceBuffer<Element: Send + Sync + Copy + 'static>:
    Buffer<Element> + UninitializedDeviceGenericArray<Element>
{
}

pub trait StagingBuffer<Element: Send + Sync + Copy + 'static>:
    Buffer<Element> + StagingGenericArray<Element>
{
}

mod slices {
    use super::*;

    #[derive(Clone, Copy)]
    pub struct Slice<Element: Send + Sync + Clone + 'static, T: GenericArray<Element>> {
        underlying: T,
        start: usize,
        len: usize,
        _phantom: PhantomData<&'static Element>,
    }

    impl<Element: Send + Sync + Clone + 'static, T: GenericArray<Element>> Slice<Element, T> {
        pub fn new(underlying: T) -> Self {
            let len = underlying.len();
            Self {
                underlying: underlying,
                start: 0,
                len: len,
                _phantom: PhantomData,
            }
        }
        pub fn underlying(&self) -> &T {
            &self.underlying
        }
        pub fn underlying_mut(&mut self) -> &mut T {
            &mut self.underlying
        }
        pub fn into_underlying(self) -> T {
            self.underlying
        }
        pub fn start(&self) -> usize {
            self.start
        }
        pub fn len(&self) -> usize {
            self.len
        }
        fn slice_helper<Range: ops::RangeBounds<usize>>(
            &self,
            range: Range,
        ) -> Slice<Element, UnbackedGenericArray<Element>> {
            use std::ops::Bound::*;
            let range_start = match range.start_bound() {
                Included(&start) => start,
                Excluded(&start) => start.checked_add(1).unwrap(),
                Unbounded => 0,
            };
            let range_end = match range.end_bound() {
                Included(&end) => end.checked_add(1).unwrap(),
                Excluded(&end) => end,
                Unbounded => self.len,
            };
            assert!(range_start <= range_end);
            assert!(range_start <= self.len);
            assert!(range_end <= self.len);
            Slice {
                underlying: (&self.underlying).into(),
                start: self.start + range_start,
                len: range_end - range_start,
                _phantom: PhantomData,
            }
        }
        pub fn slice<Range: ops::RangeBounds<usize>>(self, range: Range) -> Self {
            let Slice {
                underlying: _,
                start,
                len,
                _phantom: _,
            } = self.slice_helper(range);
            Slice {
                underlying: self.underlying,
                start: start,
                len: len,
                _phantom: PhantomData,
            }
        }
    }

    impl<Element: Send + Sync + Clone + 'static, T: StagingGenericArray<Element>> Slice<Element, T> {
        pub fn read<'a>(&'a self) -> StagingReadLockGuard<'a, T::ReadLockGuardImplementation> {
            StagingReadLockGuard::slice(self.underlying.read(), self.start, self.len)
        }
        pub fn write<'a>(&'a self) -> StagingWriteLockGuard<'a, T::WriteLockGuardImplementation> {
            StagingWriteLockGuard::slice(self.underlying.write(), self.start, self.len)
        }
    }
}

pub use slices::*;

pub trait LoaderCommandBufferBuilder: Sized {
    type Error: error::Error + Send + Sync + 'static;
    type CommandBuffer: CommandBuffer;
    type StagingVertexBuffer: StagingBuffer<VertexBufferElement>;
    type UninitializedDeviceVertexBuffer: UninitializedDeviceBuffer<VertexBufferElement>;
    type DeviceVertexBuffer: DeviceBuffer<VertexBufferElement>;
    type StagingIndexBuffer: StagingBuffer<IndexBufferElement>;
    type UninitializedDeviceIndexBuffer: UninitializedDeviceBuffer<IndexBufferElement>;
    type DeviceIndexBuffer: DeviceBuffer<IndexBufferElement>;
    type StagingImageSet: StagingImageSet;
    type UninitializedDeviceImageSet: UninitializedDeviceImageSet;
    type DeviceImageSet: DeviceImageSet;
    fn initialize_vertex_buffer(
        &mut self,
        staging_buffer: Slice<VertexBufferElement, &Self::StagingVertexBuffer>,
        device_buffer: Self::UninitializedDeviceVertexBuffer,
    ) -> Result<Self::DeviceVertexBuffer, Self::Error>;
    fn initialize_index_buffer(
        &mut self,
        staging_buffer: Slice<IndexBufferElement, &Self::StagingIndexBuffer>,
        device_buffer: Self::UninitializedDeviceIndexBuffer,
    ) -> Result<Self::DeviceIndexBuffer, Self::Error>;
    fn initialize_image_set(
        &mut self,
        staging_image_set: Slice<image::Image, &Self::StagingImageSet>,
        device_image_set: Self::UninitializedDeviceImageSet,
    ) -> Result<Self::DeviceImageSet, Self::Error>;
    fn copy_vertex_buffer_to_device(
        &mut self,
        staging_buffer: Slice<VertexBufferElement, &Self::StagingVertexBuffer>,
        device_buffer: Slice<VertexBufferElement, &Self::DeviceVertexBuffer>,
    ) -> Result<(), Self::Error>;
    fn copy_index_buffer_to_device(
        &mut self,
        staging_buffer: Slice<IndexBufferElement, &Self::StagingIndexBuffer>,
        device_buffer: Slice<IndexBufferElement, &Self::DeviceIndexBuffer>,
    ) -> Result<(), Self::Error>;
    fn copy_image_set_to_device(
        &mut self,
        staging_image_set: Slice<image::Image, &Self::StagingImageSet>,
        device_image_set: Slice<image::Image, &Self::DeviceImageSet>,
    ) -> Result<(), Self::Error>;
    fn finish(self) -> Result<Self::CommandBuffer, Self::Error>;
}

pub trait RenderCommandBufferBuilder: Sized {
    type Error: error::Error + Send + Sync + 'static;
    type CommandBuffer: CommandBuffer + Clone;
    type DeviceVertexBuffer: DeviceBuffer<VertexBufferElement>;
    type DeviceIndexBuffer: DeviceBuffer<IndexBufferElement>;
    type DeviceImageSet: DeviceImageSet;
    fn set_image_set(&mut self, image_set: &Self::DeviceImageSet);
    fn set_initial_transform(&mut self, transform: math::Mat4<f32>);
    fn draw(
        &mut self,
        vertex_buffer: Slice<VertexBufferElement, &Self::DeviceVertexBuffer>,
        index_buffer: Slice<IndexBufferElement, &Self::DeviceIndexBuffer>,
    );
    fn finish(self) -> Result<Self::CommandBuffer, Self::Error>;
}

pub trait CommandBuffer: Sized + 'static + Send + Sync {}

pub trait DeviceReference: Send + Sync + Clone + 'static {
    type Error: error::Error + Send + Sync + 'static;
    type Fence: Fence<Error = Self::Error>;
    type StagingVertexBuffer: StagingBuffer<VertexBufferElement>;
    type UninitializedDeviceVertexBuffer: UninitializedDeviceBuffer<VertexBufferElement>;
    type DeviceVertexBuffer: DeviceBuffer<VertexBufferElement>;
    type StagingIndexBuffer: StagingBuffer<IndexBufferElement>;
    type UninitializedDeviceIndexBuffer: UninitializedDeviceBuffer<IndexBufferElement>;
    type DeviceIndexBuffer: DeviceBuffer<IndexBufferElement>;
    type StagingImageSet: StagingImageSet;
    type UninitializedDeviceImageSet: UninitializedDeviceImageSet;
    type DeviceImageSet: DeviceImageSet;
    type RenderCommandBuffer: CommandBuffer + Clone;
    type RenderCommandBufferBuilder: RenderCommandBufferBuilder<
        CommandBuffer = Self::RenderCommandBuffer,
        Error = Self::Error,
        DeviceVertexBuffer = Self::DeviceVertexBuffer,
        DeviceIndexBuffer = Self::DeviceIndexBuffer,
        DeviceImageSet = Self::DeviceImageSet,
    >;
    type LoaderCommandBuffer: CommandBuffer;
    type LoaderCommandBufferBuilder: LoaderCommandBufferBuilder<
        CommandBuffer = Self::LoaderCommandBuffer,
        Error = Self::Error,
        StagingVertexBuffer = Self::StagingVertexBuffer,
        UninitializedDeviceVertexBuffer = Self::UninitializedDeviceVertexBuffer,
        DeviceVertexBuffer = Self::DeviceVertexBuffer,
        StagingIndexBuffer = Self::StagingIndexBuffer,
        DeviceIndexBuffer = Self::DeviceIndexBuffer,
        UninitializedDeviceIndexBuffer = Self::UninitializedDeviceIndexBuffer,
        StagingImageSet = Self::StagingImageSet,
        DeviceImageSet = Self::DeviceImageSet,
        UninitializedDeviceImageSet = Self::UninitializedDeviceImageSet,
    >;
    fn create_render_command_buffer_builder(
        &self,
    ) -> Result<Self::RenderCommandBufferBuilder, Self::Error>;
    fn create_loader_command_buffer_builder(
        &self,
    ) -> Result<Self::LoaderCommandBufferBuilder, Self::Error>;
    fn create_staging_vertex_buffer(
        &self,
        len: usize,
    ) -> Result<Self::StagingVertexBuffer, Self::Error>;
    fn create_device_vertex_buffer(
        &self,
        len: usize,
    ) -> Result<Self::UninitializedDeviceVertexBuffer, Self::Error>;
    fn create_staging_index_buffer(
        &self,
        len: usize,
    ) -> Result<Self::StagingIndexBuffer, Self::Error>;
    fn create_device_index_buffer(
        &self,
        len: usize,
    ) -> Result<Self::UninitializedDeviceIndexBuffer, Self::Error>;
    fn get_max_image_dimensions(&self) -> math::Vec2<u32>;
    fn get_max_image_count_in_image_set(
        &self,
        dimensions: math::Vec2<u32>,
    ) -> Result<usize, Self::Error>;
    fn create_staging_image_set(
        &self,
        dimensions: math::Vec2<u32>,
        count: usize,
    ) -> Result<Self::StagingImageSet, Self::Error>;
    fn create_device_image_set(
        &self,
        dimensions: math::Vec2<u32>,
        count: usize,
    ) -> Result<Self::UninitializedDeviceImageSet, Self::Error>;
    fn create_device_vertex_buffer_like(
        &self,
        staging_buffer: &Self::StagingVertexBuffer,
    ) -> Result<Self::UninitializedDeviceVertexBuffer, Self::Error> {
        self.create_device_vertex_buffer(staging_buffer.len())
    }
    fn create_device_index_buffer_like(
        &self,
        staging_buffer: &Self::StagingIndexBuffer,
    ) -> Result<Self::UninitializedDeviceIndexBuffer, Self::Error> {
        self.create_device_index_buffer(staging_buffer.len())
    }
    fn create_device_image_set_like(
        &self,
        staging_image_set: &Self::StagingImageSet,
    ) -> Result<Self::UninitializedDeviceImageSet, Self::Error> {
        self.create_device_image_set(staging_image_set.dimensions(), staging_image_set.len())
    }
}

pub trait PausedDevice: Sized {
    type Device: Device<PausedDevice = Self>;
    fn get_window(&self) -> &sdl::window::Window;
}

pub struct RenderCommandBufferGroup<'a, RCB: CommandBuffer> {
    pub render_command_buffers: &'a [RCB],
    pub final_transform: math::Mat4<f32>,
}

pub trait Device: Sized {
    type Error: error::Error + Send + Sync + 'static;
    type Fence: Fence<Error = Self::Error>;
    type Reference: DeviceReference<
        Error = Self::Error,
        Fence = Self::Fence,
        RenderCommandBuffer = Self::RenderCommandBuffer,
        RenderCommandBufferBuilder = Self::RenderCommandBufferBuilder,
        LoaderCommandBuffer = Self::LoaderCommandBuffer,
        LoaderCommandBufferBuilder = Self::LoaderCommandBufferBuilder,
        StagingVertexBuffer = Self::StagingVertexBuffer,
        UninitializedDeviceVertexBuffer = Self::UninitializedDeviceVertexBuffer,
        DeviceVertexBuffer = Self::DeviceVertexBuffer,
        StagingIndexBuffer = Self::StagingIndexBuffer,
        DeviceIndexBuffer = Self::DeviceIndexBuffer,
        UninitializedDeviceIndexBuffer = Self::UninitializedDeviceIndexBuffer,
        StagingImageSet = Self::StagingImageSet,
        DeviceImageSet = Self::DeviceImageSet,
        UninitializedDeviceImageSet = Self::UninitializedDeviceImageSet,
    >;
    type PausedDevice: PausedDevice<Device = Self>;
    type RenderCommandBuffer: CommandBuffer + Clone;
    type RenderCommandBufferBuilder: RenderCommandBufferBuilder<
        CommandBuffer = Self::RenderCommandBuffer,
        Error = Self::Error,
        DeviceVertexBuffer = Self::DeviceVertexBuffer,
        DeviceIndexBuffer = Self::DeviceIndexBuffer,
        DeviceImageSet = Self::DeviceImageSet,
    >;
    type LoaderCommandBuffer: CommandBuffer;
    type LoaderCommandBufferBuilder: LoaderCommandBufferBuilder<
        CommandBuffer = Self::LoaderCommandBuffer,
        Error = Self::Error,
        StagingVertexBuffer = Self::StagingVertexBuffer,
        UninitializedDeviceVertexBuffer = Self::UninitializedDeviceVertexBuffer,
        DeviceVertexBuffer = Self::DeviceVertexBuffer,
        StagingIndexBuffer = Self::StagingIndexBuffer,
        DeviceIndexBuffer = Self::DeviceIndexBuffer,
        UninitializedDeviceIndexBuffer = Self::UninitializedDeviceIndexBuffer,
        StagingImageSet = Self::StagingImageSet,
        DeviceImageSet = Self::DeviceImageSet,
        UninitializedDeviceImageSet = Self::UninitializedDeviceImageSet,
    >;
    type StagingVertexBuffer: StagingBuffer<VertexBufferElement>;
    type UninitializedDeviceVertexBuffer: UninitializedDeviceBuffer<VertexBufferElement>;
    type DeviceVertexBuffer: DeviceBuffer<VertexBufferElement>;
    type StagingIndexBuffer: StagingBuffer<IndexBufferElement>;
    type UninitializedDeviceIndexBuffer: UninitializedDeviceBuffer<IndexBufferElement>;
    type DeviceIndexBuffer: DeviceBuffer<IndexBufferElement>;
    type StagingImageSet: StagingImageSet;
    type UninitializedDeviceImageSet: UninitializedDeviceImageSet;
    type DeviceImageSet: DeviceImageSet;
    fn pause(self) -> Self::PausedDevice;
    fn resume(paused_device: Self::PausedDevice) -> Result<Self, Self::Error>;
    fn get_window(&self) -> &sdl::window::Window;
    fn get_dimensions(&self) -> math::Vec2<u32>;
    fn get_device_ref(&self) -> &Self::Reference;
    fn submit_loader_command_buffers(
        &mut self,
        loader_command_buffers: &mut Vec<Self::LoaderCommandBuffer>,
    ) -> Result<Self::Fence, Self::Error>;
    fn render_frame(
        &mut self,
        clear_color: math::Vec4<f32>,
        loader_command_buffers: &mut Vec<Self::LoaderCommandBuffer>,
        render_command_buffer_groups: &[RenderCommandBufferGroup<Self::RenderCommandBuffer>],
    ) -> Result<Self::Fence, Self::Error>;
    fn create_render_command_buffer_builder(
        &self,
    ) -> Result<Self::RenderCommandBufferBuilder, Self::Error> {
        self.get_device_ref().create_render_command_buffer_builder()
    }
    fn create_loader_command_buffer_builder(
        &self,
    ) -> Result<Self::LoaderCommandBufferBuilder, Self::Error> {
        self.get_device_ref().create_loader_command_buffer_builder()
    }
    fn create_staging_vertex_buffer(
        &self,
        len: usize,
    ) -> Result<Self::StagingVertexBuffer, Self::Error> {
        self.get_device_ref().create_staging_vertex_buffer(len)
    }
    fn create_device_vertex_buffer(
        &self,
        len: usize,
    ) -> Result<Self::UninitializedDeviceVertexBuffer, Self::Error> {
        self.get_device_ref().create_device_vertex_buffer(len)
    }
    fn create_staging_index_buffer(
        &self,
        len: usize,
    ) -> Result<Self::StagingIndexBuffer, Self::Error> {
        self.get_device_ref().create_staging_index_buffer(len)
    }
    fn create_device_index_buffer(
        &self,
        len: usize,
    ) -> Result<Self::UninitializedDeviceIndexBuffer, Self::Error> {
        self.get_device_ref().create_device_index_buffer(len)
    }
    fn get_max_image_dimensions(&self) -> math::Vec2<u32> {
        self.get_device_ref().get_max_image_dimensions()
    }
    fn get_max_image_count_in_image_set(
        &self,
        dimensions: math::Vec2<u32>,
    ) -> Result<usize, Self::Error> {
        self.get_device_ref()
            .get_max_image_count_in_image_set(dimensions)
    }
    fn create_staging_image_set(
        &self,
        dimensions: math::Vec2<u32>,
        count: usize,
    ) -> Result<Self::StagingImageSet, Self::Error> {
        self.get_device_ref()
            .create_staging_image_set(dimensions, count)
    }
    fn create_device_image_set(
        &self,
        dimensions: math::Vec2<u32>,
        count: usize,
    ) -> Result<Self::UninitializedDeviceImageSet, Self::Error> {
        self.get_device_ref()
            .create_device_image_set(dimensions, count)
    }
    fn create_device_vertex_buffer_like(
        &self,
        staging_buffer: &Self::StagingVertexBuffer,
    ) -> Result<Self::UninitializedDeviceVertexBuffer, Self::Error> {
        self.create_device_vertex_buffer(staging_buffer.len())
    }
    fn create_device_index_buffer_like(
        &self,
        staging_buffer: &Self::StagingIndexBuffer,
    ) -> Result<Self::UninitializedDeviceIndexBuffer, Self::Error> {
        self.create_device_index_buffer(staging_buffer.len())
    }
    fn create_device_image_set_like(
        &self,
        staging_image_set: &Self::StagingImageSet,
    ) -> Result<Self::UninitializedDeviceImageSet, Self::Error> {
        self.create_device_image_set(staging_image_set.dimensions(), staging_image_set.len())
    }
}

pub trait DeviceFactory {
    type Error: error::Error + Send + Sync + 'static;
    type Device: Device<Error = Self::Error, PausedDevice = Self::PausedDevice>;
    type PausedDevice: PausedDevice<Device = Self::Device>;
    fn create<T: Into<String>>(
        &self,
        title: T,
        position: Option<(i32, i32)>,
        size: (u32, u32),
        flags: u32,
    ) -> Result<Self::PausedDevice, Self::Error>;
}
