mod hashtable;
mod world;
use std::env;
use world::*;

fn write_state(state: &State) {
    let range = 1 << 4;
    for y in -range..range {
        for x in -range..range {
            print!("{}", [' ', '#'][state.get(x, y) as usize]);
        }
        println!();
    }
}

fn main() {
    if cfg!(debug_assertions) {
        env::set_var("RUST_BACKTRACE", "1");
    }
    let world = World::new();
    let mut state = State::create_empty(&world);
    world.borrow_mut().gc();
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
        state.set(x - 5, y, 1 as Block);
    }
    //println!("{:#?}", state);
    write_state(&state);
    state.step(0);
    write_state(&state);
    for log2_step_size in 0..4 {
        for _ in 1..3 {
            state.step(log2_step_size);
            write_state(&state);
        }
    }
}
