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
extern crate voxels_renderer_base as renderer;
extern crate voxels_renderer_gles2 as gles2;
extern crate voxels_renderer_vulkan as vulkan;
extern crate voxels_sdl as sdl;

use std::io;

pub use renderer::*;

pub trait MainLoop {
    fn startup<DF: DeviceFactory>(
        &self,
        device_factory: DF,
    ) -> Result<DF::PausedDevice, DF::Error> {
        device_factory.create("", None, (640, 480), 0)
    }
    fn main_loop<PD: PausedDevice>(self, paused_device: PD, event_source: &sdl::event::EventSource);
}

pub enum BackendRunResult<ML: MainLoop> {
    StartupFailed { error: io::Error, main_loop: ML },
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
                            error: error.to_io_error(),
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
