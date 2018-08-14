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
use super::api;
use libc::*;
use std::ffi::*;
use std::ptr::null_mut;

pub struct Window(*mut api::SDL_Window);

unsafe impl Sync for Window {}

impl Window {
    pub fn new<T: Into<String>>(
        title: T,
        position: Option<(i32, i32)>,
        size: (u32, u32),
        flags: u32,
    ) -> Result<Self, super::SDLError> {
        let title = title.into();
        let title = CString::new(title).unwrap();
        let position = match position {
            Some(position) => position,
            None => (
                api::SDL_WINDOWPOS_UNDEFINED as i32,
                api::SDL_WINDOWPOS_UNDEFINED as i32,
            ),
        };
        unsafe {
            let window = api::SDL_CreateWindow(
                title.as_ptr(),
                position.0,
                position.1,
                size.0 as c_int,
                size.1 as c_int,
                flags,
            );
            if window == null_mut() {
                Err(super::get_error())
            } else {
                Ok(Window(window))
            }
        }
    }
    pub fn get(&self) -> *mut api::SDL_Window {
        self.0
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        unsafe { api::SDL_DestroyWindow(self.0) }
    }
}
