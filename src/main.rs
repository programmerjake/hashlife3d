mod hashtable;
mod world3d;
use std::env;
use world3d::{State, World};

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

fn main() {
    if cfg!(debug_assertions) {
        env::set_var("RUST_BACKTRACE", "1");
    }
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
