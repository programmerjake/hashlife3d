#![allow(dead_code)]
#![allow(unused_imports)]
use super::api;
use libc::c_char;
use libc::c_int;
use std::env;
use std::mem::drop;
use std::panic::catch_unwind;
use std::process::Termination;

struct CallOnDrop<F: FnOnce()> {
    f: Option<F>,
}

impl<F: FnOnce()> CallOnDrop<F> {
    fn new(f: F) -> CallOnDrop<F> {
        Self { f: Some(f) }
    }
}

impl<F: FnOnce()> Drop for CallOnDrop<F> {
    fn drop(&mut self) {
        (self.f.take().unwrap())();
    }
}

unsafe fn run_main() {
    if cfg!(debug_assertions) {
        env::set_var("RUST_BACKTRACE", "1");
    }
    env::set_var("SDL_VIDEODRIVER", "wayland");
    if api::SDL_Init(api::SDL_INIT_VIDEO) != 0 {
        eprintln!(
            "SDL_Init failed: {}\nTrying default SDL driver.",
            super::get_error()
        );
        env::remove_var("SDL_VIDEODRIVER");
        if api::SDL_Init(api::SDL_INIT_VIDEO) != 0 {
            panic!("SDL_Init failed: {}", super::get_error());
        }
    }
    let sdl = CallOnDrop::new(|| api::SDL_Quit());
    let retval = ::rust_main(&super::event::make_event_source());
    drop(sdl);
    retval
}

#[cfg(not(test))]
#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn SDL_main(_argc: c_int, _argv: *mut *mut c_char) -> c_int {
    catch_unwind(|| unsafe { run_main() }).report()
}
