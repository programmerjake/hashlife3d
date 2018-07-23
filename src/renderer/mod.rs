pub mod vulkan;
use super::sdl::window::Window;
use std::time::Duration;
use std::u64;

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

pub trait Device: Sync {
    type Semaphore: Semaphore;
    type Fence: Fence;
    type Queue: Queue;
    type Error;
    fn get_window(&self) -> &Window;
    fn create_fence(&self, initial_state: FenceState) -> Result<Self::Fence, Self::Error>;
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
}

pub trait DeviceFactory {
    type Device: Device;
    fn create<T: Into<String>>(
        &self,
        title: T,
        position: Option<(i32, i32)>,
        size: (u32, u32),
        flags: u32,
    ) -> Result<Self::Device, <Self::Device as Device>::Error>;
}
