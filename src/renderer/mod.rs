mod gles2;
pub mod math;
mod vulkan;
use super::sdl;
use std::error;
use std::time::Duration;
use std::u64;

pub trait Semaphore: Send {}

pub trait Fence: Send {}

pub trait Queue {}

pub enum WaitResult {
    Success,
    Timeout,
}

pub enum ShaderSource {
    MainVertex,
}

pub trait Shader {}

pub enum FenceState {
    Signaled,
    Unsignaled,
}

pub trait DeviceReference: Send + Sync + Clone + 'static {
    type Semaphore: Semaphore;
    type Fence: Fence;
    type Error: error::Error;
    type Shader: Shader;
    fn get_shader(&self, shader_source: ShaderSource) -> Result<Self::Shader, Self::Error>;
    fn create_fence(&self, initial_state: FenceState) -> Result<Self::Fence, Self::Error>;
}

pub trait PausedDevice {
    fn get_window(&self) -> &sdl::window::Window;
}

pub trait Device: Sized {
    type Reference: DeviceReference;
    type Queue: Queue;
    type PausedDevice: PausedDevice;
    fn pause(self) -> Self::PausedDevice;
    fn resume(
        paused_device: Self::PausedDevice,
    ) -> Result<Self, <Self::Reference as DeviceReference>::Error>;
    fn get_window(&self) -> &sdl::window::Window;
    fn get_device_ref(&self) -> Self::Reference;
    fn get_queue(&self) -> &Self::Queue;
    fn wait_for_fences_with_timeout(
        &self,
        fences: &[&<Self::Reference as DeviceReference>::Fence],
        wait_for_all: bool,
        timeout: Duration,
    ) -> Result<WaitResult, <Self::Reference as DeviceReference>::Error>;
    fn wait_for_fences(
        &self,
        fences: &[&<Self::Reference as DeviceReference>::Fence],
        wait_for_all: bool,
    ) -> Result<(), <Self::Reference as DeviceReference>::Error> {
        self.wait_for_fences_with_timeout(fences, wait_for_all, Duration::new(u64::MAX, 0))
            .map(|_| ())
    }
    fn wait_for_all_fences(
        &self,
        fences: &[&<Self::Reference as DeviceReference>::Fence],
    ) -> Result<(), <Self::Reference as DeviceReference>::Error> {
        self.wait_for_fences(fences, true)
    }
    fn wait_for_any_fence(
        &self,
        fences: &[&<Self::Reference as DeviceReference>::Fence],
    ) -> Result<(), <Self::Reference as DeviceReference>::Error> {
        self.wait_for_fences(fences, false)
    }
    fn wait_for_fence(
        &self,
        fence: &<Self::Reference as DeviceReference>::Fence,
    ) -> Result<(), <Self::Reference as DeviceReference>::Error> {
        self.wait_for_fences(&[fence], false)
    }
    fn wait_for_all_fences_with_timeout(
        &self,
        fences: &[&<Self::Reference as DeviceReference>::Fence],
        timeout: Duration,
    ) -> Result<WaitResult, <Self::Reference as DeviceReference>::Error> {
        self.wait_for_fences_with_timeout(fences, true, timeout)
    }
    fn wait_for_any_fence_with_timeout(
        &self,
        fences: &[&<Self::Reference as DeviceReference>::Fence],
        timeout: Duration,
    ) -> Result<WaitResult, <Self::Reference as DeviceReference>::Error> {
        self.wait_for_fences_with_timeout(fences, false, timeout)
    }
    fn wait_for_fence_with_timeout(
        &self,
        fence: &<Self::Reference as DeviceReference>::Fence,
        timeout: Duration,
    ) -> Result<WaitResult, <Self::Reference as DeviceReference>::Error> {
        self.wait_for_fences_with_timeout(&[fence], false, timeout)
    }
}

pub trait DeviceFactory {
    type Device: Device;
    fn create<T: Into<String>>(
        &self,
        title: T,
        position: Option<(i32, i32)>,
        size: (u32, u32),
        flags: u32,
    ) -> Result<Self::Device, <<Self::Device as Device>::Reference as DeviceReference>::Error>;
}

pub trait MainLoop {
    fn startup<DF: DeviceFactory>(
        &self,
        device_factory: DF,
    ) -> Result<DF::Device, Box<error::Error>> {
        device_factory
            .create("", None, (640, 480), 0)
            .map_err(|v| Box::new(v).into())
    }
    fn main_loop<D: Device>(self, device: D, event_source: &sdl::event::EventSource);
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
