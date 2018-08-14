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
pub mod event;
mod main;
pub mod window;
#[cfg(not(test))]
pub use self::main::SDL_main;
use std::error::Error;
use std::ffi::*;
use std::fmt;

#[allow(dead_code)]
#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
pub mod api {
    include!(concat!(env!("OUT_DIR"), "/sdl-bindings.rs"));
    pub const SDL_WINDOWPOS_UNDEFINED: u32 = SDL_WINDOWPOS_UNDEFINED_MASK;
    pub const SDL_WINDOWPOS_CENTERED: u32 = SDL_WINDOWPOS_CENTERED_MASK;
}

pub fn get_error() -> SDLError {
    SDLError(
        unsafe { CStr::from_ptr(api::SDL_GetError()) }
            .to_str()
            .unwrap()
            .into(),
    )
}

#[derive(Debug)]
pub struct SDLError(String);

impl fmt::Display for SDLError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Error for SDLError {}
