#![feature(termination_trait_lib)]
#![feature(concat_idents)]
#![feature(vec_resize_with)]
#![cfg_attr(not(test), no_main)]
extern crate libc;
mod hashtable;
mod renderer;
mod sdl;
mod world3d;
use self::math::Dot;
use self::math::Mappable;
#[cfg(not(test))]
pub use self::sdl::SDL_main;
use renderer::math;
use renderer::*;
use sdl::event::Event;
use std::error;
use std::time;
use world3d::{State, World};

#[no_mangle]
#[cfg(
    not(
        any(
            target_os = "windows",
            target_os = "ios",
            target_os = "android",
            test
        )
    )
)]
pub extern "C" fn main(argc: libc::c_int, argv: *mut *mut libc::c_char) -> libc::c_int {
    SDL_main(argc, argv)
}

type Block = u32;

fn write_state(state: &State<Block, hashtable::DefaultBuildHasher>) {
    let range = 1 << 4;
    for z in -range..range {
        println!("z={}", z);
        for y in -range..range {
            for x in -range..range {
                print!("{}", [' ', '#'][state.get(x, y, z) as usize]);
            }
            println!();
        }
    }
}

fn render_main_loop<PD: renderer::PausedDevice>(
    paused_device: PD,
    event_source: &sdl::event::EventSource,
) {
    struct Running<D: renderer::Device> {
        device: D,
        render_command_buffer: D::RenderCommandBuffer,
        last_fps_report: time::Instant,
        frame_count_since_last_fps_report: u32,
    }
    impl<D: renderer::Device> Running<D> {
        fn new(mut device: D) -> Result<Self, D::Error> {
            let mut indices: Vec<IndexBufferElement> = Vec::new();
            let mut vertices: Vec<VertexBufferElement> = Vec::new();
            {
                let mut append_vertex = |vertex: VertexBufferElement| {
                    let index = vertices.len();
                    vertices.push(vertex);
                    index as IndexBufferElement
                };
                let mut render_quad =
                    |v0: math::Vec3<f32>,
                     v1: math::Vec3<f32>,
                     v2: math::Vec3<f32>,
                     v3: math::Vec3<f32>,
                     color: math::Vec4<u8>| {
                        let v0 = append_vertex(VertexBufferElement::new(
                            v0,
                            color,
                            Default::default(),
                            0,
                        ));
                        let v1 = append_vertex(VertexBufferElement::new(
                            v1,
                            color,
                            Default::default(),
                            0,
                        ));
                        let v2 = append_vertex(VertexBufferElement::new(
                            v2,
                            color,
                            Default::default(),
                            0,
                        ));
                        let v3 = append_vertex(VertexBufferElement::new(
                            v3,
                            color,
                            Default::default(),
                            0,
                        ));
                        indices.push(v0);
                        indices.push(v1);
                        indices.push(v2);
                        indices.push(v2);
                        indices.push(v3);
                        indices.push(v0);
                    };
                let lights: &[(math::Vec3<f32>, math::Vec3<f32>)] = &[
                    (
                        math::Vec3::<f32>::new(1.0, -0.3, -0.3).normalize().unwrap(),
                        math::Vec3::<f32>::new(1.0, 0.0, 0.0),
                    ),
                    (
                        math::Vec3::<f32>::new(-0.3, 1.0, -0.3).normalize().unwrap(),
                        math::Vec3::<f32>::new(0.0, 1.0, 0.0),
                    ),
                    (
                        math::Vec3::<f32>::new(-0.3, -0.3, 1.0).normalize().unwrap(),
                        math::Vec3::<f32>::new(0.0, 0.0, 1.0),
                    ),
                ];
                let get_color = |normal: math::Vec3<f32>| {
                    let mut retval = math::Vec3::splat(0.3);
                    for light in lights {
                        let amount = normal.dot(light.0).max(0.0);
                        retval += math::Vec3::splat(amount) * light.1;
                    }
                    let retval = retval.map(|v| (v * 255.0).max(0.0).min(255.0) as u8);
                    math::Vec4::new(retval.x, retval.y, retval.z, 255)
                };
                render_quad(
                    math::Vec3::new(1.0, -1.0, -1.0),
                    math::Vec3::new(1.0, -1.0, 1.0),
                    math::Vec3::new(-1.0, -1.0, 1.0),
                    math::Vec3::new(-1.0, -1.0, -1.0),
                    get_color(math::Vec3::new(0.0, -1.0, 0.0)),
                );
                render_quad(
                    math::Vec3::new(1.0, 1.0, -1.0),
                    math::Vec3::new(-1.0, 1.0, -1.0),
                    math::Vec3::new(-1.0, 1.0, 1.0),
                    math::Vec3::new(1.0, 1.0, 1.0),
                    get_color(math::Vec3::new(0.0, 1.0, 0.0)),
                );
                render_quad(
                    math::Vec3::new(1.0, -1.0, -1.0),
                    math::Vec3::new(1.0, 1.0, -1.0),
                    math::Vec3::new(1.0, 1.0, 1.0),
                    math::Vec3::new(1.0, -1.0, 1.0),
                    get_color(math::Vec3::new(1.0, 0.0, 0.0)),
                );
                render_quad(
                    math::Vec3::new(1.0, -1.0, 1.0),
                    math::Vec3::new(1.0, 1.0, 1.0),
                    math::Vec3::new(-1.0, 1.0, 1.0),
                    math::Vec3::new(-1.0, -1.0, 1.0),
                    get_color(math::Vec3::new(0.0, 0.0, 1.0)),
                );
                render_quad(
                    math::Vec3::new(-1.0, -1.0, 1.0),
                    math::Vec3::new(-1.0, 1.0, 1.0),
                    math::Vec3::new(-1.0, 1.0, -1.0),
                    math::Vec3::new(-1.0, -1.0, -1.0),
                    get_color(math::Vec3::new(-1.0, 0.0, 0.0)),
                );
                render_quad(
                    math::Vec3::new(1.0, 1.0, -1.0),
                    math::Vec3::new(1.0, -1.0, -1.0),
                    math::Vec3::new(-1.0, -1.0, -1.0),
                    math::Vec3::new(-1.0, 1.0, -1.0),
                    get_color(math::Vec3::new(0.0, 0.0, -1.0)),
                );
            }
            let mut loader_command_buffer_builder =
                device.create_loader_command_buffer_builder()?;
            let mut render_command_buffer_builder =
                device.create_render_command_buffer_builder()?;
            let mut index_buffer = device.create_staging_index_buffer(indices.len())?;
            for (index, element) in indices.iter().enumerate() {
                index_buffer.write(index, *element);
            }
            let index_buffer =
                loader_command_buffer_builder.copy_index_buffer_to_device(index_buffer)?;
            let mut vertex_buffer = device.create_staging_vertex_buffer(vertices.len())?;
            for (index, element) in vertices.iter().enumerate() {
                vertex_buffer.write(index, *element);
            }
            let vertex_buffer =
                loader_command_buffer_builder.copy_vertex_buffer_to_device(vertex_buffer)?;
            render_command_buffer_builder.set_buffers(vertex_buffer, index_buffer);
            render_command_buffer_builder.draw(indices.len() as u32, 0, 0);
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
                    if true {
                        transform_matrix =
                            transform_matrix.translate(math::Vec3::new(0.0, 0.0, -5.0));
                        transform_matrix = transform_matrix.rotate(
                            (time * 90.0).to_radians(),
                            math::Vec3::new(1.0, 0.5, 1.0f32).normalize().unwrap(),
                        );
                        transform_matrix = transform_matrix.rotate(
                            (time * 180.0).to_radians(),
                            math::Vec3::new(1.0, -0.5, 1.0f32).normalize().unwrap(),
                        );
                    } else {
                        transform_matrix =
                            transform_matrix.translate(math::Vec3::new(0.0, 0.0, -3.0));
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
fn rust_main(event_source: &sdl::event::EventSource) {
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
                state.set(&mut world, x - 5, y, 0, 1 as Block);
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
        event_source: event_source,
    }) {
        if let Some(name) = selected_backend {
            panic!("unknown backend: {}", name);
        } else {
            panic!("all graphics backends failed to start");
        }
    }
    world_thread.join().unwrap()
}
