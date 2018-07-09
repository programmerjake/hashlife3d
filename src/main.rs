mod hashtable;
mod world;
use world::*;

fn main() {
    let world = World::new();
    let state = State::create_empty(&world);
    world.borrow_mut().gc();
    println!("{:#?}", state);
}
