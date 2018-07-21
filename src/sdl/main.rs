use super::api;
use libc::c_char;
use libc::c_int;
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

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn SDL_main(_argc: c_int, _argv: *mut *mut c_char) -> c_int {
    catch_unwind(|| unsafe {
        if api::SDL_Init(api::SDL_INIT_VIDEO) != 0 {
            panic!("SDL_Init failed: {}", super::get_error_message());
        }
        let _ = CallOnDrop::new(|| api::SDL_Quit());
        ::rust_main()
    }).report()
}
