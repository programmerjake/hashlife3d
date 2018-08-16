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
mod gles2;
pub mod image;
pub mod math;
mod vulkan;
use super::sdl;
use std::error;
use std::u64;

pub type TextureIndex = u16;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct VertexBufferElement {
    pub position: [f32; 3],
    pub color: [u8; 4],
    pub texture_coord: [f32; 2],
    pub texture_index: TextureIndex,
}

impl VertexBufferElement {
    pub fn new(
        position: math::Vec3,
        color: math::Vec4<u8>,
        texture_coord: math::Vec2,
        texture_index: TextureIndex,
    ) -> Self {
        Self {
            position: position.into(),
            color: color.into(),
            texture_coord: texture_coord.into(),
            texture_index: texture_index,
        }
    }
}

pub type IndexBufferElement = u16;

pub trait StagingVertexBuffer: Sized + Send {
    fn len(&self) -> usize;
    fn write(&mut self, index: usize, value: VertexBufferElement);
}

pub trait DeviceVertexBuffer: Sized + Send + Clone {
    fn len(&self) -> usize;
}

pub trait StagingIndexBuffer: Sized + Send {
    fn len(&self) -> usize;
    fn write(&mut self, index: usize, value: IndexBufferElement);
}

pub trait DeviceIndexBuffer: Sized + Send + Clone {
    fn len(&self) -> usize;
}

pub trait StagingImageSet: Sized + Send {
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn count(&self) -> u32;
    fn write(&mut self, image_index: TextureIndex, image: &image::Image);
}

pub trait DeviceImageSet: Sized + Send + Clone {
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn count(&self) -> u32;
}

pub trait LoaderCommandBufferBuilder: Sized {
    type Error: error::Error + 'static;
    type CommandBuffer: CommandBuffer;
    type StagingVertexBuffer: StagingVertexBuffer;
    type DeviceVertexBuffer: DeviceVertexBuffer;
    type StagingIndexBuffer: StagingIndexBuffer;
    type DeviceIndexBuffer: DeviceIndexBuffer;
    type StagingImageSet: StagingImageSet;
    type DeviceImageSet: DeviceImageSet;
    fn copy_vertex_buffer_to_device(
        &mut self,
        staging_vertex_buffer: Self::StagingVertexBuffer,
    ) -> Result<Self::DeviceVertexBuffer, Self::Error>;
    fn copy_index_buffer_to_device(
        &mut self,
        staging_index_buffer: Self::StagingIndexBuffer,
    ) -> Result<Self::DeviceIndexBuffer, Self::Error>;
    fn copy_image_set_to_device(
        &mut self,
        staging_image_set: Self::StagingImageSet,
    ) -> Result<Self::DeviceImageSet, Self::Error>;
    fn finish(self) -> Result<Self::CommandBuffer, Self::Error>;
}

pub trait RenderCommandBufferBuilder: Sized {
    type Error: error::Error + 'static;
    type CommandBuffer: CommandBuffer + Clone;
    type DeviceVertexBuffer: DeviceVertexBuffer;
    type DeviceIndexBuffer: DeviceIndexBuffer;
    type DeviceImageSet: DeviceImageSet;
    fn set_image_set(&mut self, image_set: Self::DeviceImageSet);
    fn set_buffers(
        &mut self,
        vertex_buffer: Self::DeviceVertexBuffer,
        index_buffer: Self::DeviceIndexBuffer,
    );
    fn set_initial_transform(&mut self, transform: math::Mat4<f32>);
    fn draw(&mut self, index_count: u32, first_index: u32, vertex_offset: u32);
    fn finish(self) -> Result<Self::CommandBuffer, Self::Error>;
}

pub trait CommandBuffer: Sized + 'static + Send {}

pub trait DeviceReference: Send + Sync + Clone + 'static {
    type Error: error::Error + 'static;
    type StagingVertexBuffer: StagingVertexBuffer;
    type DeviceVertexBuffer: DeviceVertexBuffer;
    type StagingIndexBuffer: StagingIndexBuffer;
    type DeviceIndexBuffer: DeviceIndexBuffer;
    type StagingImageSet: StagingImageSet;
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
        DeviceVertexBuffer = Self::DeviceVertexBuffer,
        StagingIndexBuffer = Self::StagingIndexBuffer,
        DeviceIndexBuffer = Self::DeviceIndexBuffer,
        StagingImageSet = Self::StagingImageSet,
        DeviceImageSet = Self::DeviceImageSet,
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
    fn create_staging_index_buffer(
        &self,
        len: usize,
    ) -> Result<Self::StagingIndexBuffer, Self::Error>;
    fn get_max_image_width(&self) -> u32;
    fn get_max_image_height(&self) -> u32;
    fn get_max_image_count_in_image_set(&self, width: u32, height: u32)
        -> Result<u32, Self::Error>;
    fn create_staging_image_set(
        &self,
        width: u32,
        height: u32,
        count: u32,
    ) -> Result<Self::StagingImageSet, Self::Error>;
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
    type Error: error::Error + 'static;
    type Reference: DeviceReference<
        Error = Self::Error,
        RenderCommandBuffer = Self::RenderCommandBuffer,
        RenderCommandBufferBuilder = Self::RenderCommandBufferBuilder,
        LoaderCommandBuffer = Self::LoaderCommandBuffer,
        LoaderCommandBufferBuilder = Self::LoaderCommandBufferBuilder,
        StagingVertexBuffer = Self::StagingVertexBuffer,
        DeviceVertexBuffer = Self::DeviceVertexBuffer,
        StagingIndexBuffer = Self::StagingIndexBuffer,
        DeviceIndexBuffer = Self::DeviceIndexBuffer,
        StagingImageSet = Self::StagingImageSet,
        DeviceImageSet = Self::DeviceImageSet,
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
        DeviceVertexBuffer = Self::DeviceVertexBuffer,
        StagingIndexBuffer = Self::StagingIndexBuffer,
        DeviceIndexBuffer = Self::DeviceIndexBuffer,
        StagingImageSet = Self::StagingImageSet,
        DeviceImageSet = Self::DeviceImageSet,
    >;
    type StagingVertexBuffer: StagingVertexBuffer;
    type DeviceVertexBuffer: DeviceVertexBuffer;
    type StagingIndexBuffer: StagingIndexBuffer;
    type DeviceIndexBuffer: DeviceIndexBuffer;
    type StagingImageSet: StagingImageSet;
    type DeviceImageSet: DeviceImageSet;
    fn pause(self) -> Self::PausedDevice;
    fn resume(paused_device: Self::PausedDevice) -> Result<Self, Self::Error>;
    fn get_window(&self) -> &sdl::window::Window;
    fn get_device_ref(&self) -> &Self::Reference;
    fn submit_loader_command_buffers(
        &mut self,
        loader_command_buffers: &mut Vec<Self::LoaderCommandBuffer>,
    ) -> Result<(), Self::Error>;
    fn render_frame(
        &mut self,
        clear_color: math::Vec4<f32>,
        loader_command_buffers: &mut Vec<Self::LoaderCommandBuffer>,
        render_command_buffer_groups: &[RenderCommandBufferGroup<Self::RenderCommandBuffer>],
    ) -> Result<(), Self::Error>;
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
    fn create_staging_index_buffer(
        &self,
        len: usize,
    ) -> Result<Self::StagingIndexBuffer, Self::Error> {
        self.get_device_ref().create_staging_index_buffer(len)
    }
    fn get_max_image_width(&self) -> u32 {
        self.get_device_ref().get_max_image_width()
    }
    fn get_max_image_height(&self) -> u32 {
        self.get_device_ref().get_max_image_height()
    }
    fn get_max_image_count_in_image_set(
        &self,
        width: u32,
        height: u32,
    ) -> Result<u32, Self::Error> {
        self.get_device_ref()
            .get_max_image_count_in_image_set(width, height)
    }
    fn create_staging_image_set(
        &self,
        width: u32,
        height: u32,
        count: u32,
    ) -> Result<Self::StagingImageSet, Self::Error> {
        self.get_device_ref()
            .create_staging_image_set(width, height, count)
    }
}

pub trait DeviceFactory {
    type Error: error::Error + 'static;
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

pub trait MainLoop {
    fn startup<DF: DeviceFactory>(
        &self,
        device_factory: DF,
    ) -> Result<DF::PausedDevice, Box<error::Error>> {
        device_factory
            .create("", None, (640, 480), 0)
            .map_err(|v| Box::new(v).into())
    }
    fn main_loop<PD: PausedDevice>(self, paused_device: PD, event_source: &sdl::event::EventSource);
}

pub enum BackendRunResult<ML: MainLoop> {
    StartupFailed {
        error: Box<error::Error>,
        main_loop: ML,
    },
    RanMainLoop,
}

pub trait Backend {
    fn get_name(&self) -> &'static str;
    fn get_title(&self) -> &'static str;
    fn run_main_loop<ML: MainLoop>(
        &self,
        main_loop: ML,
        event_source: &sdl::event::EventSource,
    ) -> BackendRunResult<ML>;
}

pub enum BackendVisitorResult {
    Continue,
    Break,
}

pub trait BackendVisitor {
    fn visit<B: Backend>(&mut self, backend: B) -> BackendVisitorResult;
}

pub fn for_each_backend<BV: BackendVisitor>(backend_visitor: &mut BV) -> BackendVisitorResult {
    macro_rules! visit_backend {
        ($device_factory:ty, $name:expr, $title:expr) => {{
            struct BackendStruct {}
            impl Backend for BackendStruct {
                fn get_name(&self) -> &'static str {
                    $name
                }
                fn get_title(&self) -> &'static str {
                    $title
                }
                fn run_main_loop<ML: MainLoop>(
                    &self,
                    main_loop: ML,
                    event_source: &sdl::event::EventSource,
                ) -> BackendRunResult<ML> {
                    match main_loop.startup(<$device_factory>::new(event_source)) {
                        Ok(device) => {
                            main_loop.main_loop(device, event_source);
                            BackendRunResult::RanMainLoop
                        }
                        Err(error) => BackendRunResult::StartupFailed {
                            error: error,
                            main_loop: main_loop,
                        },
                    }
                }
            }
            if let BackendVisitorResult::Break = backend_visitor.visit(BackendStruct {}) {
                return BackendVisitorResult::Break;
            }
        }};
    }
    visit_backend!(self::vulkan::VulkanDeviceFactory, "vulkan", "Vulkan");
    visit_backend!(self::gles2::GLES2DeviceFactory, "gles2", "OpenGL ES 2.0");
    BackendVisitorResult::Continue
}
