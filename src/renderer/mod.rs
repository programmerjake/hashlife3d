mod gles2;
pub mod math;
mod vulkan;
use super::sdl;
use std::error;
use std::time::Duration;
use std::u64;

#[repr(C)]
pub struct VertexBufferElement {
    pub position: [f32; 3],
    pub color: [u8; 4],
    pub texture_coord: [f32; 2],
    pub texture_index: u16,
}

impl VertexBufferElement {
    pub fn new(
        position: math::Vec3,
        color: math::Vec4<u8>,
        texture_coord: math::Vec2,
        texture_index: u16,
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

pub trait DeviceVertexBuffer: Sized + Send + Clone {}

pub trait StagingIndexBuffer: Sized + Send {
    fn len(&self) -> usize;
    fn write(&mut self, index: usize, value: IndexBufferElement);
}

pub trait DeviceIndexBuffer: Sized + Send + Clone {}

pub trait LoaderCommandBufferBuilder: Sized {
    type Error: error::Error + 'static;
    type CommandBuffer: CommandBuffer;
    type StagingVertexBuffer: StagingVertexBuffer;
    type DeviceVertexBuffer: DeviceVertexBuffer;
    type StagingIndexBuffer: StagingIndexBuffer;
    type DeviceIndexBuffer: DeviceIndexBuffer;
    fn copy_vertex_buffer_to_device(
        &mut self,
        staging_vertex_buffer: Self::StagingVertexBuffer,
    ) -> Result<Self::DeviceVertexBuffer, Self::Error>;
    fn copy_index_buffer_to_device(
        &mut self,
        staging_index_buffer: Self::StagingIndexBuffer,
    ) -> Result<Self::DeviceIndexBuffer, Self::Error>;
    fn finish(self) -> Result<Self::CommandBuffer, Self::Error>;
}

pub trait RenderCommandBufferBuilder: Sized {
    type Error: error::Error + 'static;
    type CommandBuffer: CommandBuffer + Clone;
    type DeviceVertexBuffer: DeviceVertexBuffer;
    type DeviceIndexBuffer: DeviceIndexBuffer;
    fn finish(self) -> Result<Self::CommandBuffer, Self::Error>;
}

pub trait CommandBuffer: Sized + 'static + Send {}

pub trait Semaphore: Send {}

pub trait Fence: Send {}

pub trait Queue {}

pub enum WaitResult {
    Success,
    Timeout,
}

pub enum FenceState {
    Signaled,
    Unsignaled,
}

pub trait DeviceReference: Send + Sync + Clone + 'static {
    type Semaphore: Semaphore;
    type Fence: Fence;
    type Error: error::Error + 'static;
    type StagingVertexBuffer: StagingVertexBuffer;
    type DeviceVertexBuffer: DeviceVertexBuffer;
    type StagingIndexBuffer: StagingIndexBuffer;
    type DeviceIndexBuffer: DeviceIndexBuffer;
    type RenderCommandBuffer: CommandBuffer + Clone;
    type RenderCommandBufferBuilder: RenderCommandBufferBuilder<
        CommandBuffer = Self::RenderCommandBuffer,
        Error = Self::Error,
        DeviceVertexBuffer = Self::DeviceVertexBuffer,
        DeviceIndexBuffer = Self::DeviceIndexBuffer,
    >;
    type LoaderCommandBuffer: CommandBuffer;
    type LoaderCommandBufferBuilder: LoaderCommandBufferBuilder<
        CommandBuffer = Self::LoaderCommandBuffer,
        Error = Self::Error,
        StagingVertexBuffer = Self::StagingVertexBuffer,
        DeviceVertexBuffer = Self::DeviceVertexBuffer,
        StagingIndexBuffer = Self::StagingIndexBuffer,
        DeviceIndexBuffer = Self::DeviceIndexBuffer,
    >;
    fn create_fence(&self, initial_state: FenceState) -> Result<Self::Fence, Self::Error>;
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
}

pub trait PausedDevice: Sized {
    type Device: Device<PausedDevice = Self>;
    fn get_window(&self) -> &sdl::window::Window;
}

pub trait Device: Sized {
    type Semaphore: Semaphore;
    type Fence: Fence;
    type Error: error::Error + 'static;
    type Reference: DeviceReference<
        Semaphore = Self::Semaphore,
        Fence = Self::Fence,
        Error = Self::Error,
        RenderCommandBuffer = Self::RenderCommandBuffer,
        RenderCommandBufferBuilder = Self::RenderCommandBufferBuilder,
        LoaderCommandBuffer = Self::LoaderCommandBuffer,
        LoaderCommandBufferBuilder = Self::LoaderCommandBufferBuilder,
        StagingVertexBuffer = Self::StagingVertexBuffer,
        DeviceVertexBuffer = Self::DeviceVertexBuffer,
        StagingIndexBuffer = Self::StagingIndexBuffer,
        DeviceIndexBuffer = Self::DeviceIndexBuffer,
    >;
    type Queue: Queue;
    type PausedDevice: PausedDevice<Device = Self>;
    type RenderCommandBuffer: CommandBuffer + Clone;
    type RenderCommandBufferBuilder: RenderCommandBufferBuilder<
        CommandBuffer = Self::RenderCommandBuffer,
        Error = Self::Error,
        DeviceVertexBuffer = Self::DeviceVertexBuffer,
        DeviceIndexBuffer = Self::DeviceIndexBuffer,
    >;
    type LoaderCommandBuffer: CommandBuffer;
    type LoaderCommandBufferBuilder: LoaderCommandBufferBuilder<
        CommandBuffer = Self::LoaderCommandBuffer,
        Error = Self::Error,
        StagingVertexBuffer = Self::StagingVertexBuffer,
        DeviceVertexBuffer = Self::DeviceVertexBuffer,
        StagingIndexBuffer = Self::StagingIndexBuffer,
        DeviceIndexBuffer = Self::DeviceIndexBuffer,
    >;
    type StagingVertexBuffer: StagingVertexBuffer;
    type DeviceVertexBuffer: DeviceVertexBuffer;
    type StagingIndexBuffer: StagingIndexBuffer;
    type DeviceIndexBuffer: DeviceIndexBuffer;
    fn pause(self) -> Self::PausedDevice;
    fn resume(paused_device: Self::PausedDevice) -> Result<Self, Self::Error>;
    fn get_window(&self) -> &sdl::window::Window;
    fn get_device_ref(&self) -> &Self::Reference;
    fn get_queue(&self) -> &Self::Queue;
    fn wait_for_fences_with_timeout(
        &self,
        fences: &[&Self::Fence],
        wait_for_all: bool,
        timeout: Duration,
    ) -> Result<WaitResult, Self::Error>;
    fn wait_for_fences(
        &self,
        fences: &[&Self::Fence],
        wait_for_all: bool,
    ) -> Result<(), Self::Error> {
        self.wait_for_fences_with_timeout(fences, wait_for_all, Duration::new(u64::MAX, 0))
            .map(|_| ())
    }
    fn wait_for_all_fences(&self, fences: &[&Self::Fence]) -> Result<(), Self::Error> {
        self.wait_for_fences(fences, true)
    }
    fn wait_for_any_fence(&self, fences: &[&Self::Fence]) -> Result<(), Self::Error> {
        self.wait_for_fences(fences, false)
    }
    fn wait_for_fence(&self, fence: &Self::Fence) -> Result<(), Self::Error> {
        self.wait_for_fences(&[fence], false)
    }
    fn wait_for_all_fences_with_timeout(
        &self,
        fences: &[&Self::Fence],
        timeout: Duration,
    ) -> Result<WaitResult, Self::Error> {
        self.wait_for_fences_with_timeout(fences, true, timeout)
    }
    fn wait_for_any_fence_with_timeout(
        &self,
        fences: &[&Self::Fence],
        timeout: Duration,
    ) -> Result<WaitResult, Self::Error> {
        self.wait_for_fences_with_timeout(fences, false, timeout)
    }
    fn wait_for_fence_with_timeout(
        &self,
        fence: &Self::Fence,
        timeout: Duration,
    ) -> Result<WaitResult, Self::Error> {
        self.wait_for_fences_with_timeout(&[fence], false, timeout)
    }
    fn create_fence(&self, initial_state: FenceState) -> Result<Self::Fence, Self::Error> {
        self.get_device_ref().create_fence(initial_state)
    }
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
