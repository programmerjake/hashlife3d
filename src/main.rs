#![no_main]
#![feature(termination_trait_lib)]
extern crate libc;
mod hashtable;
mod sdl;
mod world3d;
pub use self::sdl::SDL_main;
use sdl::event::Event;
use world3d::{State, World};
use std::mem;

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
    {
        let window = sdl::window::Window::new("Title", 640, 480);
        loop {
            match event_source.next() {
                Event::Quit { .. } => break,
                event => println!("unhandled event: {:?}", event),
            }
        }
        mem::drop(window);
    }
    world_thread.join().unwrap()
}
