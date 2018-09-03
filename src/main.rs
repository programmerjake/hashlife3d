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
use world3d::{State, World};

type Block = u32;

fn write_state(state: &State<Block, hashtable::DefaultBuildHasher>) {
    let range = 1 << 4;
    for z in -range..range {
        println!("z={}", z);
        for y in -range..range {
            for x in -range..range {
                print!(
                    "{}",
                    [' ', '#'][state.get(math::Vec3::new(x, y, z)) as usize]
                );
            }
            println!();
        }
    }
}

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
                        break running_state.render_state.into_device().pause();
                    }
                    event @ Event::Quit { .. } => {
                        running_state.render_state.send_event(event);
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

// FIXME: remove
#[cfg(all(unix, not(unix)))]
fn render_main_loop<PD: renderer::PausedDevice>(
    paused_device: PD,
    event_source: &sdl::event::EventSource,
) {
    let mut rotating = true;
    struct Running<D: renderer::Device> {
        device: D,
        render_command_buffer: D::RenderCommandBuffer,
        last_fps_report: time::Instant,
        frame_count_since_last_fps_report: u32,
    }
    impl<D: renderer::Device> Running<D> {
        fn new(mut device: D) -> Result<Self, D::Error> {
            let mut mesh = geometry::Mesh::new();
            let nx_texture_id = resources::images::tiles::TEST_NX.texture_id().unwrap();
            let px_texture_id = resources::images::tiles::TEST_PX.texture_id().unwrap();
            let ny_texture_id = resources::images::tiles::TEST_NY.texture_id().unwrap();
            let py_texture_id = resources::images::tiles::TEST_PY.texture_id().unwrap();
            let nz_texture_id = resources::images::tiles::TEST_NZ.texture_id().unwrap();
            let pz_texture_id = resources::images::tiles::TEST_PZ.texture_id().unwrap();
            fn get_color(x: i32, y: i32, z: i32) -> math::Vec4<u8> {
                math::Vec4::new(
                    if x != 2 { 0xFF } else { 0x80 },
                    if y != 2 { 0xFF } else { 0x80 },
                    if z != 2 { 0xFF } else { 0x80 },
                    0xFF,
                )
            }
            let size = 3;
            for x in -size..=size {
                for y in -size..=size {
                    for z in -size..=size {
                        mesh.add_cube(
                            math::Vec3::new(x as f32, y as f32, z as f32) - math::Vec3::splat(0.5),
                            get_color(x + 0, y + 0, z + 0),
                            get_color(x + 0, y + 0, z + 1),
                            get_color(x + 0, y + 1, z + 0),
                            get_color(x + 0, y + 1, z + 1),
                            get_color(x + 1, y + 0, z + 0),
                            get_color(x + 1, y + 0, z + 1),
                            get_color(x + 1, y + 1, z + 0),
                            get_color(x + 1, y + 1, z + 1),
                            if x == -size {
                                Some(nx_texture_id)
                            } else {
                                None
                            },
                            if x == size { Some(px_texture_id) } else { None },
                            if y == -size {
                                Some(ny_texture_id)
                            } else {
                                None
                            },
                            if y == size { Some(py_texture_id) } else { None },
                            if z == -size {
                                Some(nz_texture_id)
                            } else {
                                None
                            },
                            if z == size { Some(pz_texture_id) } else { None },
                        );
                    }
                }
            }
            let mut loader_command_buffer_builder =
                device.create_loader_command_buffer_builder()?;
            let mut render_command_buffer_builder =
                device.create_render_command_buffer_builder()?;
            let (staging_vertex_buffer, staging_index_buffer) =
                mesh.create_staging_buffers(device.get_device_ref())?;
            let device_index_buffer = loader_command_buffer_builder.initialize_index_buffer(
                staging_index_buffer.slice_ref(..),
                device.create_device_index_buffer_like(&staging_index_buffer)?,
            )?;
            let device_vertex_buffer = loader_command_buffer_builder.initialize_vertex_buffer(
                staging_vertex_buffer.slice_ref(..),
                device.create_device_vertex_buffer_like(&staging_vertex_buffer)?,
            )?;
            let staging_image_set =
                resources::images::tiles::create_tiles_image_set(device.get_device_ref())?;
            let device_image_set = loader_command_buffer_builder.initialize_image_set(
                staging_image_set.slice_ref(..),
                device.create_device_image_set_like(&staging_image_set)?,
            )?;
            render_command_buffer_builder.set_image_set(&device_image_set);
            render_command_buffer_builder.draw(
                device_vertex_buffer.slice_ref(..),
                device_index_buffer.slice_ref(..),
            );
            let loader_command_buffer = loader_command_buffer_builder.finish()?;
            let render_command_buffer = render_command_buffer_builder.finish()?;
            device.submit_loader_command_buffers(&mut vec![loader_command_buffer])?;
            Ok(Self {
                device: device,
                render_command_buffer: render_command_buffer,
                last_fps_report: time::Instant::now(),
                frame_count_since_last_fps_report: 0,
            })
        }
    }
    struct Paused<PD: renderer::PausedDevice> {
        device: PD,
    }
    enum State<D: renderer::Device<PausedDevice = PD>, PD: renderer::PausedDevice<Device = D>> {
        Running(Running<D>),
        Paused(Paused<PD>),
    }
    let mut state_enum = State::Paused(Paused {
        device: paused_device,
    });
    let start_instant = time::Instant::now();
    loop {
        match state_enum {
            State::Running(mut state) => {
                if let Some(event) = event_source.poll() {
                    match event {
                        event @ Event::WindowHidden { .. } => {
                            println!("event: {:?}", event);
                            state_enum = State::Paused(Paused {
                                device: state.device.pause(),
                            });
                            continue;
                        }
                        event @ Event::Quit { .. } => {
                            println!("event: {:?}", event);
                            return;
                        }
                        event @ Event::KeyDown {
                            scancode: sdl::event::Scancode::Space,
                            ..
                        } => {
                            println!("event: {:?}", event);
                            rotating = !rotating;
                        }
                        event => println!("unhandled event: {:?}", event),
                    }
                } else {
                    {
                        let current_time = time::Instant::now();
                        let elapsed_time = current_time.duration_since(state.last_fps_report);
                        let elapsed_time = elapsed_time.subsec_nanos() as f32 / 1e9
                            + elapsed_time.as_secs() as f32;
                        state.frame_count_since_last_fps_report += 1;
                        if elapsed_time >= 5.0 {
                            let fps = state.frame_count_since_last_fps_report as f32 / elapsed_time;
                            state.last_fps_report = current_time;
                            state.frame_count_since_last_fps_report = 0;
                            println!("FPS: {}", fps);
                        }
                    }
                    let elapsed_time = start_instant.elapsed();
                    let time =
                        elapsed_time.subsec_nanos() as f32 / 1e9 + elapsed_time.as_secs() as f32;
                    let near = 0.1;
                    let far = 10.0;
                    let mut transform_matrix;
                    if true {
                        transform_matrix = math::Mat4::<f32>::perspective_projection(
                            -near, near, -near, near, near, far,
                        );
                    } else {
                        transform_matrix = math::Mat4::<f32>::orthographic_projection(
                            -1.0, 1.0, -1.0, 1.0, near, far,
                        );
                    }
                    if rotating {
                        transform_matrix =
                            transform_matrix.translate(math::Vec3::new(0.0, 0.0, -9.0));
                        transform_matrix = transform_matrix.rotate(
                            (time * 30.0).to_radians(),
                            math::Vec3::new(1.0, 0.5, 1.0f32).normalize().unwrap(),
                        );
                        transform_matrix = transform_matrix.rotate(
                            (time * 60.0).to_radians(),
                            math::Vec3::new(1.0, -0.5, 1.0f32).normalize().unwrap(),
                        );
                    } else {
                        transform_matrix =
                            transform_matrix.translate(math::Vec3::new(0.0, 0.0, -9.0));
                    }
                    state
                        .device
                        .render_frame(
                            math::Vec4::new(0.1, 0.1, 0.1, 1.0),
                            &mut Vec::new(),
                            &[RenderCommandBufferGroup {
                                render_command_buffers: &[state.render_command_buffer.clone()],
                                final_transform: transform_matrix,
                            }],
                        ).unwrap();
                }
                state_enum = State::Running(state);
            }
            State::Paused(state) => {
                match event_source.next() {
                    event @ Event::Quit { .. } => {
                        println!("event while paused: {:?}", event);
                        return;
                    }
                    event @ Event::WindowShown { .. } => {
                        println!("event while paused: {:?}", event);
                        state_enum = State::Running(
                            Running::new(renderer::Device::resume(state.device).unwrap()).unwrap(),
                        );
                        continue;
                    }
                    event => println!("unhandled event while paused: {:?}", event),
                }
                state_enum = State::Paused(state);
            }
        }
    }
}

#[allow(dead_code)]
#[no_mangle]
pub fn rust_main(event_source: sdl::event::EventSource) {
    let world_thread = std::thread::spawn(|| {
        if false {
            let mut world = World::new(
                |blocks: &[[[Block; 3]; 3]; 3]| blocks[1][1][1],
                Default::default(),
            );
            let mut state = State::create_empty(&mut world);
            world.gc();
            let _glider = [(-1, 0), (0, 0), (1, 0), (1, 1), (0, 2)];
            let _lwss = [
                (1, 0),
                (2, 0),
                (3, 0),
                (4, 0),
                (0, 1),
                (1, 1),
                (2, 1),
                (3, 1),
                (4, 1),
                (5, 1),
                (0, 2),
                (1, 2),
                (2, 2),
                (3, 2),
                (5, 2),
                (6, 2),
                (4, 3),
                (5, 3),
            ];
            for &(x, y) in &_lwss {
                state.set(&mut world, math::Vec3::new(x - 5, y, 0), 1 as Block);
            }
            //println!("{:#?}", state);
            write_state(&state);
            state.step(&mut world, 0);
            write_state(&state);
            for log2_step_size in 0..4 {
                for _ in 1..3 {
                    state.step(&mut world, log2_step_size);
                    write_state(&state);
                }
            }
        }
    });
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
    world_thread.join().unwrap()
}

#[allow(dead_code)]
fn assert_rust_main_is_right_type() -> sdl::RustMainType {
    rust_main
}
