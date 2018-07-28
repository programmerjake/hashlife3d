#![no_main]
#![feature(termination_trait_lib)]
#![feature(concat_idents)]
#![feature(vec_resize_with)]
extern crate libc;
mod hashtable;
mod renderer;
mod sdl;
mod world3d;
pub use self::sdl::SDL_main;
use renderer::*;
use sdl::event::Event;
use std::error;
use world3d::{State, World};

#[no_mangle]
#[cfg(
    not(
        any(
            target_os = "windows",
            target_os = "ios",
            target_os = "android"
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
    }
    impl<D: renderer::Device> Running<D> {
        fn new(device: D) -> Result<Self, D::Error> {
            Ok(Self { device: device })
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
    loop {
        match state_enum {
            State::Running(state) => {
                match event_source.next() {
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
            device_factory
                .create("", None, (640, 480), 0)
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
    struct BackendVisitor<'a> {
        main_loop: Option<MainLoop>,
        event_source: &'a sdl::event::EventSource,
    }
    impl<'a> renderer::BackendVisitor for BackendVisitor<'a> {
        fn visit<B: Backend>(&mut self, backend: B) -> renderer::BackendVisitorResult {
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
    if let BackendVisitorResult::Continue = renderer::for_each_backend(&mut BackendVisitor {
        main_loop: Some(MainLoop {}),
        event_source: event_source,
    }) {
        panic!("all graphics backends failed to start");
    }
    world_thread.join().unwrap()
}
