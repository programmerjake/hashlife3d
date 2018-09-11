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
#![cfg_attr(not(test), no_main)]
#[macro_use]
extern crate enum_map;
extern crate voxels_image as image;
extern crate voxels_math as math;
extern crate voxels_renderer as renderer;
extern crate voxels_resources as resources;
extern crate voxels_sdl as sdl;

mod block;
mod chunk_cache;
mod game_state;
mod geometry;
mod hashtable;
mod registry;
mod world3d;
use registry::RegistryBuilder;
use renderer::*;
use sdl::event::Event;
use std::error;
#[allow(unused_imports)]
use std::os::raw::{c_char, c_int};

fn render_main_loop<PD: renderer::PausedDevice>(
    mut paused_device: PD,
    event_source: &sdl::event::EventSource,
) {
    let mut registry_builder = RegistryBuilder::new();
    block::register_blocks(&mut registry_builder);
    let registry = registry_builder.finish_startup();
    let mut game_state = game_state::GameState::new(registry.clone());
    struct Running<'a, D: renderer::Device> {
        render_state: game_state::RenderState<'a, D>,
    }
    loop {
        let mut running_state: Running<PD::Device> = loop {
            match event_source.next() {
                event @ Event::Quit { .. } => {
                    game_state.send_event(event);
                    return;
                }
                event @ Event::WindowShown { .. } => {
                    game_state.send_event(event);
                    break Running {
                        render_state: game_state::RenderState::new(
                            renderer::Device::resume(paused_device).unwrap(),
                            &mut game_state,
                            registry.clone(),
                        ).unwrap(),
                    };
                }
                event => println!("unhandled event while paused: {:?}", event),
            }
        };
        paused_device = loop {
            if let Some(event) = event_source.poll() {
                match event {
                    event @ Event::WindowHidden { .. } => {
                        running_state.render_state.send_event(event);
                        running_state.render_state.print_stats();
                        break running_state.render_state.into_device().pause();
                    }
                    event @ Event::Quit { .. } => {
                        running_state.render_state.send_event(event);
                        running_state.render_state.print_stats();
                        return;
                    }
                    event => running_state.render_state.send_event(event),
                }
            } else {
                running_state.render_state.render_frame().unwrap();
            }
        }
    }
}

#[allow(dead_code)]
#[no_mangle]
pub fn rust_main(event_source: sdl::event::EventSource) {
    struct MainLoop {}
    impl renderer::MainLoop for MainLoop {
        fn startup<DF: renderer::DeviceFactory>(
            &self,
            device_factory: DF,
        ) -> Result<DF::PausedDevice, Box<error::Error>> {
            let flags = sdl::api::SDL_WINDOW_RESIZABLE;
            //let flags = sdl::api::SDL_WINDOW_FULLSCREEN_DESKTOP;
            device_factory
                .create("Hashlife3d", None, (640, 480), flags)
                .map_err(|v| Box::new(v).into())
        }
        fn main_loop<PD: renderer::PausedDevice>(
            self,
            paused_device: PD,
            event_source: &sdl::event::EventSource,
        ) {
            render_main_loop(paused_device, event_source);
        }
    }
    struct BackendVisitor<'a, 'b> {
        main_loop: Option<MainLoop>,
        selected_backend: &'b mut Option<String>,
        event_source: &'a sdl::event::EventSource,
    }
    impl<'a, 'b> renderer::BackendVisitor for BackendVisitor<'a, 'b> {
        fn visit<B: Backend>(&mut self, backend: B) -> renderer::BackendVisitorResult {
            if let Some(name) = &self.selected_backend {
                if backend.get_name() != name {
                    return renderer::BackendVisitorResult::Continue;
                }
            }
            *self.selected_backend = None;
            eprintln!("starting using {}", backend.get_title());
            match backend.run_main_loop(self.main_loop.take().unwrap(), self.event_source) {
                renderer::BackendRunResult::StartupFailed { error, main_loop } => {
                    self.main_loop = Some(main_loop);
                    eprintln!("starting using {} failed: {}", backend.get_title(), error);
                    renderer::BackendVisitorResult::Continue
                }
                renderer::BackendRunResult::RanMainLoop => renderer::BackendVisitorResult::Break,
            }
        }
    }
    let mut selected_backend = None;
    if false {
        // FIXME: change back to dynamically selecting the backend
        selected_backend = Some(String::from("gles2"));
    }
    if let BackendVisitorResult::Continue = renderer::for_each_backend(&mut BackendVisitor {
        main_loop: Some(MainLoop {}),
        selected_backend: &mut selected_backend,
        event_source: &event_source,
    }) {
        if let Some(name) = selected_backend {
            panic!("unknown backend: {}", name);
        } else {
            panic!("all graphics backends failed to start");
        }
    }
}

#[allow(dead_code)]
fn assert_rust_main_is_right_type() -> sdl::RustMainType {
    rust_main
}

#[no_mangle]
#[cfg(not(any(
    target_os = "windows",
    target_os = "ios",
    target_os = "android",
    test
)))]
pub unsafe extern "C" fn main(argc: c_int, argv: *mut *mut c_char) -> c_int {
    sdl::SDL_main(argc, argv)
}
