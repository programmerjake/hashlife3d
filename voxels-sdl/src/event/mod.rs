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
mod scancode;
use super::api;
use std::ffi::*;
use std::fmt;
use std::mem::*;
use std::ops;
use std::os::raw::*;
use std::ptr::*;

use self::scancode::make_keycode;
pub use self::scancode::Keycode;
pub use self::scancode::Scancode;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct KeyModifiers(pub u16);

#[allow(dead_code)]
impl KeyModifiers {
    pub const NONE: KeyModifiers = KeyModifiers(api::KMOD_NONE as u16);
    pub const LSHIFT: KeyModifiers = KeyModifiers(api::KMOD_LSHIFT as u16);
    pub const RSHIFT: KeyModifiers = KeyModifiers(api::KMOD_RSHIFT as u16);
    pub const LCTRL: KeyModifiers = KeyModifiers(api::KMOD_LCTRL as u16);
    pub const RCTRL: KeyModifiers = KeyModifiers(api::KMOD_RCTRL as u16);
    pub const LALT: KeyModifiers = KeyModifiers(api::KMOD_LALT as u16);
    pub const RALT: KeyModifiers = KeyModifiers(api::KMOD_RALT as u16);
    pub const LGUI: KeyModifiers = KeyModifiers(api::KMOD_LGUI as u16);
    pub const RGUI: KeyModifiers = KeyModifiers(api::KMOD_RGUI as u16);
    pub const NUM: KeyModifiers = KeyModifiers(api::KMOD_NUM as u16);
    pub const CAPS: KeyModifiers = KeyModifiers(api::KMOD_CAPS as u16);
    pub const MODE: KeyModifiers = KeyModifiers(api::KMOD_MODE as u16);
    pub const SHIFT: KeyModifiers = KeyModifiers(KeyModifiers::LSHIFT.0 | KeyModifiers::RSHIFT.0);
    pub const CTRL: KeyModifiers = KeyModifiers(KeyModifiers::LCTRL.0 | KeyModifiers::RCTRL.0);
    pub const ALT: KeyModifiers = KeyModifiers(KeyModifiers::LALT.0 | KeyModifiers::RALT.0);
    pub const GUI: KeyModifiers = KeyModifiers(KeyModifiers::LGUI.0 | KeyModifiers::RGUI.0);
    pub fn is_none(&self) -> bool {
        *self == KeyModifiers::NONE
    }
    pub fn includes_lshift(&self) -> bool {
        !(*self & KeyModifiers::LSHIFT).is_none()
    }
    pub fn includes_rshift(&self) -> bool {
        !(*self & KeyModifiers::RSHIFT).is_none()
    }
    pub fn includes_lctrl(&self) -> bool {
        !(*self & KeyModifiers::LCTRL).is_none()
    }
    pub fn includes_rctrl(&self) -> bool {
        !(*self & KeyModifiers::RCTRL).is_none()
    }
    pub fn includes_lalt(&self) -> bool {
        !(*self & KeyModifiers::LALT).is_none()
    }
    pub fn includes_ralt(&self) -> bool {
        !(*self & KeyModifiers::RALT).is_none()
    }
    pub fn includes_lgui(&self) -> bool {
        !(*self & KeyModifiers::LGUI).is_none()
    }
    pub fn includes_rgui(&self) -> bool {
        !(*self & KeyModifiers::RGUI).is_none()
    }
    pub fn includes_num(&self) -> bool {
        !(*self & KeyModifiers::NUM).is_none()
    }
    pub fn includes_caps(&self) -> bool {
        !(*self & KeyModifiers::CAPS).is_none()
    }
    pub fn includes_mode(&self) -> bool {
        !(*self & KeyModifiers::MODE).is_none()
    }
    pub fn includes_shift(&self) -> bool {
        !(*self & KeyModifiers::SHIFT).is_none()
    }
    pub fn includes_ctrl(&self) -> bool {
        !(*self & KeyModifiers::CTRL).is_none()
    }
    pub fn includes_alt(&self) -> bool {
        !(*self & KeyModifiers::ALT).is_none()
    }
    pub fn includes_gui(&self) -> bool {
        !(*self & KeyModifiers::GUI).is_none()
    }
}

impl ops::Not for KeyModifiers {
    type Output = KeyModifiers;
    fn not(self) -> Self {
        KeyModifiers(!self.0)
    }
}

impl ops::BitAnd for KeyModifiers {
    type Output = KeyModifiers;
    fn bitand(self, rhs: KeyModifiers) -> KeyModifiers {
        KeyModifiers(self.0 & rhs.0)
    }
}

impl ops::BitAndAssign for KeyModifiers {
    fn bitand_assign(&mut self, rhs: KeyModifiers) {
        self.0 &= rhs.0;
    }
}

impl ops::BitOr for KeyModifiers {
    type Output = KeyModifiers;
    fn bitor(self, rhs: KeyModifiers) -> KeyModifiers {
        KeyModifiers(self.0 | rhs.0)
    }
}

impl ops::BitOrAssign for KeyModifiers {
    fn bitor_assign(&mut self, rhs: KeyModifiers) {
        self.0 |= rhs.0;
    }
}

impl ops::BitXor for KeyModifiers {
    type Output = KeyModifiers;
    fn bitxor(self, rhs: KeyModifiers) -> KeyModifiers {
        KeyModifiers(self.0 ^ rhs.0)
    }
}

impl ops::BitXorAssign for KeyModifiers {
    fn bitxor_assign(&mut self, rhs: KeyModifiers) {
        self.0 ^= rhs.0;
    }
}

impl fmt::Debug for KeyModifiers {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut debug_set = f.debug_set();
        if *self == KeyModifiers::NONE {
            debug_set.entry(&"NONE");
        }
        if self.includes_lshift() {
            debug_set.entry(&"LSHIFT");
        }
        if self.includes_rshift() {
            debug_set.entry(&"RSHIFT");
        }
        if self.includes_lctrl() {
            debug_set.entry(&"LCTRL");
        }
        if self.includes_rctrl() {
            debug_set.entry(&"RCTRL");
        }
        if self.includes_lalt() {
            debug_set.entry(&"LALT");
        }
        if self.includes_ralt() {
            debug_set.entry(&"RALT");
        }
        if self.includes_lgui() {
            debug_set.entry(&"LGUI");
        }
        if self.includes_rgui() {
            debug_set.entry(&"RGUI");
        }
        if self.includes_num() {
            debug_set.entry(&"NUM");
        }
        if self.includes_caps() {
            debug_set.entry(&"CAPS");
        }
        if self.includes_mode() {
            debug_set.entry(&"MODE");
        }
        debug_set.finish()
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct MouseButton(u8);

#[allow(dead_code)]
impl MouseButton {
    pub const NONE: MouseButton = MouseButton(0);
    pub const LEFT: MouseButton = MouseButton(api::SDL_BUTTON_LEFT as u8);
    pub const MIDDLE: MouseButton = MouseButton(api::SDL_BUTTON_MIDDLE as u8);
    pub const RIGHT: MouseButton = MouseButton(api::SDL_BUTTON_RIGHT as u8);
    pub const X1: MouseButton = MouseButton(api::SDL_BUTTON_X1 as u8);
    pub const X2: MouseButton = MouseButton(api::SDL_BUTTON_X2 as u8);
}

impl fmt::Debug for MouseButton {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match *self {
            MouseButton::NONE => "NONE",
            MouseButton::LEFT => "LEFT",
            MouseButton::MIDDLE => "MIDDLE",
            MouseButton::RIGHT => "RIGHT",
            MouseButton::X1 => "X1",
            MouseButton::X2 => "X2",
            _ => "Unknown",
        })
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct MouseButtons(pub u32);

impl MouseButtons {
    pub const NONE: MouseButtons = MouseButtons(0);
    pub const LEFT: MouseButtons = MouseButtons(1 << (MouseButton::LEFT.0 - 1));
    pub const MIDDLE: MouseButtons = MouseButtons(1 << (MouseButton::MIDDLE.0 - 1));
    pub const RIGHT: MouseButtons = MouseButtons(1 << (MouseButton::RIGHT.0 - 1));
    pub const X1: MouseButtons = MouseButtons(1 << (MouseButton::X1.0 - 1));
    pub const X2: MouseButtons = MouseButtons(1 << (MouseButton::X2.0 - 1));
    pub fn is_none(&self) -> bool {
        *self == MouseButtons::NONE
    }
    pub fn includes_left(&self) -> bool {
        !(*self & MouseButtons::LEFT).is_none()
    }
    pub fn includes_middle(&self) -> bool {
        !(*self & MouseButtons::MIDDLE).is_none()
    }
    pub fn includes_right(&self) -> bool {
        !(*self & MouseButtons::RIGHT).is_none()
    }
    pub fn includes_x1(&self) -> bool {
        !(*self & MouseButtons::X1).is_none()
    }
    pub fn includes_x2(&self) -> bool {
        !(*self & MouseButtons::X2).is_none()
    }
}

impl ops::Not for MouseButtons {
    type Output = MouseButtons;
    fn not(self) -> Self {
        MouseButtons(!self.0)
    }
}

impl ops::BitAnd for MouseButtons {
    type Output = MouseButtons;
    fn bitand(self, rhs: MouseButtons) -> MouseButtons {
        MouseButtons(self.0 & rhs.0)
    }
}

impl ops::BitAndAssign for MouseButtons {
    fn bitand_assign(&mut self, rhs: MouseButtons) {
        self.0 &= rhs.0;
    }
}

impl ops::BitOr for MouseButtons {
    type Output = MouseButtons;
    fn bitor(self, rhs: MouseButtons) -> MouseButtons {
        MouseButtons(self.0 | rhs.0)
    }
}

impl ops::BitOrAssign for MouseButtons {
    fn bitor_assign(&mut self, rhs: MouseButtons) {
        self.0 |= rhs.0;
    }
}

impl ops::BitXor for MouseButtons {
    type Output = MouseButtons;
    fn bitxor(self, rhs: MouseButtons) -> MouseButtons {
        MouseButtons(self.0 ^ rhs.0)
    }
}

impl ops::BitXorAssign for MouseButtons {
    fn bitxor_assign(&mut self, rhs: MouseButtons) {
        self.0 ^= rhs.0;
    }
}

impl fmt::Debug for MouseButtons {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut debug_set = f.debug_set();
        if *self == MouseButtons::NONE {
            debug_set.entry(&"NONE");
        }
        if self.includes_left() {
            debug_set.entry(&"LEFT");
        }
        if self.includes_middle() {
            debug_set.entry(&"MIDDLE");
        }
        if self.includes_right() {
            debug_set.entry(&"RIGHT");
        }
        if self.includes_x1() {
            debug_set.entry(&"X1");
        }
        if self.includes_x2() {
            debug_set.entry(&"X2");
        }
        debug_set.finish()
    }
}

#[derive(Eq, PartialEq, Debug, Clone, Copy, Hash)]
pub struct MouseID(pub u32);

impl MouseID {
    #[allow(dead_code)]
    pub const TOUCH: MouseID = MouseID(!0);
}

#[derive(Eq, PartialEq, Debug, Clone, Copy, Hash)]
pub enum MouseWheelDirection {
    Normal,
    Flipped,
}

#[allow(dead_code)]
impl MouseWheelDirection {
    pub fn is_normal(&self) -> bool {
        match self {
            MouseWheelDirection::Normal => true,
            _ => false,
        }
    }
    pub fn is_flipped(&self) -> bool {
        match self {
            MouseWheelDirection::Flipped => true,
            _ => false,
        }
    }
    pub fn get_normal_scroll_amount(&self, x: i32, y: i32) -> (i32, i32) {
        match self {
            MouseWheelDirection::Normal => (x, y),
            MouseWheelDirection::Flipped => (-x, -y),
        }
    }
}

#[derive(Eq, PartialEq, Debug, Clone, Copy, Hash)]
pub struct JoystickID(pub u32);

impl JoystickID {
    fn from_sdl(v: i32) -> Option<JoystickID> {
        if v < 0 {
            None
        } else {
            Some(JoystickID(v as u32))
        }
    }
}

#[derive(Eq, PartialEq, Debug, Clone, Copy, Hash)]
pub enum JoystickHatDirection {
    LeftUp,
    Up,
    RightUp,
    Left,
    Centered,
    Right,
    LeftDown,
    Down,
    RightDown,
}

#[allow(dead_code)]
impl JoystickHatDirection {
    fn from_sdl(v: u8) -> JoystickHatDirection {
        use self::JoystickHatDirection::*;
        match v as u32 {
            api::SDL_HAT_LEFTUP => LeftUp,
            api::SDL_HAT_UP => Up,
            api::SDL_HAT_RIGHTUP => RightUp,
            api::SDL_HAT_LEFT => Left,
            api::SDL_HAT_CENTERED => Centered,
            api::SDL_HAT_RIGHT => Right,
            api::SDL_HAT_LEFTDOWN => LeftDown,
            api::SDL_HAT_DOWN => Down,
            api::SDL_HAT_RIGHTDOWN => RightDown,
            _ => panic!("unknown joystick hat direction"),
        }
    }
    pub fn get_cartesian_coordinates(&self) -> (i32, i32) {
        use self::JoystickHatDirection::*;
        match self {
            LeftUp => (-1, 1),
            Up => (0, 1),
            RightUp => (1, 1),
            Left => (-1, 0),
            Centered => (0, 0),
            Right => (1, 0),
            LeftDown => (-1, -1),
            Down => (0, -1),
            RightDown => (1, -1),
        }
    }
}

pub type UserEvent = api::SDL_UserEvent;

#[derive(Debug)]
pub enum Event {
    Quit {
        timestamp: u32,
    },
    AppTerminating {
        timestamp: u32,
    },
    AppLowMemory {
        timestamp: u32,
    },
    AppWillEnterBackground {
        timestamp: u32,
    },
    AppDidEnterBackground {
        timestamp: u32,
    },
    AppWillEnterForeground {
        timestamp: u32,
    },
    AppDidEnterForeground {
        timestamp: u32,
    },
    WindowShown {
        timestamp: u32,
        window_id: u32,
    },
    WindowHidden {
        timestamp: u32,
        window_id: u32,
    },
    WindowExposed {
        timestamp: u32,
        window_id: u32,
    },
    WindowMoved {
        timestamp: u32,
        window_id: u32,
        x: i32,
        y: i32,
    },
    WindowResized {
        timestamp: u32,
        window_id: u32,
        w: u32,
        h: u32,
    },
    WindowSizeChanged {
        timestamp: u32,
        window_id: u32,
        w: u32,
        h: u32,
    },
    WindowMinimized {
        timestamp: u32,
        window_id: u32,
    },
    WindowMaximized {
        timestamp: u32,
        window_id: u32,
    },
    WindowRestored {
        timestamp: u32,
        window_id: u32,
    },
    MouseEntered {
        timestamp: u32,
        window_id: u32,
    },
    MouseLeft {
        timestamp: u32,
        window_id: u32,
    },
    KeyboardFocusGained {
        timestamp: u32,
        window_id: u32,
    },
    KeyboardFocusLost {
        timestamp: u32,
        window_id: u32,
    },
    WindowClose {
        timestamp: u32,
        window_id: u32,
    },
    WindowTakeFocus {
        timestamp: u32,
        window_id: u32,
    },
    WindowHitTest {
        timestamp: u32,
        window_id: u32,
    },
    SysWMEvent {
        timestamp: u32,
        msg: *mut api::SDL_SysWMmsg,
    },
    KeyDown {
        timestamp: u32,
        window_id: u32,
        repeat: bool,
        scancode: Scancode,
        keycode: Keycode,
        modifiers: KeyModifiers,
    },
    KeyUp {
        timestamp: u32,
        window_id: u32,
        scancode: Scancode,
        keycode: Keycode,
        modifiers: KeyModifiers,
    },
    TextEditing {
        timestamp: u32,
        window_id: u32,
        text: String,
        start: usize,
    },
    TextInput {
        timestamp: u32,
        window_id: u32,
        text: String,
    },
    KeyMapChanged {
        timestamp: u32,
    },
    MouseMotion {
        timestamp: u32,
        window_id: u32,
        which: MouseID,
        state: MouseButtons,
        x: i32,
        y: i32,
        x_relative: i32,
        y_relative: i32,
    },
    MouseButtonDown {
        timestamp: u32,
        window_id: u32,
        which: MouseID,
        button: MouseButton,
        x: i32,
        y: i32,
    },
    MouseButtonUp {
        timestamp: u32,
        window_id: u32,
        which: MouseID,
        button: MouseButton,
        x: i32,
        y: i32,
    },
    MouseWheel {
        timestamp: u32,
        window_id: u32,
        which: MouseID,
        x: i32,
        y: i32,
        direction: MouseWheelDirection,
    },
    JoystickAxisMotion {
        timestamp: u32,
        which: JoystickID,
        axis: u8,
        value: i16,
    },
    JoystickBallMotion {
        timestamp: u32,
        which: JoystickID,
        ball: u8,
        x_relative: i16,
        y_relative: i16,
    },
    JoystickHatMotion {
        timestamp: u32,
        which: JoystickID,
        hat: u8,
        value: JoystickHatDirection,
    },
    JoystickButtonDown {
        timestamp: u32,
        which: JoystickID,
        button: u8,
    },
    JoystickButtonUp {
        timestamp: u32,
        which: JoystickID,
        button: u8,
    },
    JoystickDeviceAdded {
        timestamp: u32,
        which: i32,
    },
    JoystickDeviceRemoved {
        timestamp: u32,
        which: JoystickID,
    },
    ControllerAxisMotion {
        timestamp: u32,
        which: JoystickID,
        axis: u8,
        value: i16,
    },
    ControllerButtonDown {
        timestamp: u32,
        which: JoystickID,
        button: u8,
    },
    ControllerButtonUp {
        timestamp: u32,
        which: JoystickID,
        button: u8,
    },
    ControllerDeviceAdded {
        timestamp: u32,
        which: i32,
    },
    ControllerDeviceRemoved {
        timestamp: u32,
        which: JoystickID,
    },
    ControllerDeviceRemapped {
        timestamp: u32,
        which: JoystickID,
    },
    FingerDown {
        timestamp: u32,
        touch_id: i64,
        finger_id: i64,
        x: f32,
        y: f32,
        pressure: f32,
    },
    FingerUp {
        timestamp: u32,
        touch_id: i64,
        finger_id: i64,
        x: f32,
        y: f32,
        pressure: f32,
    },
    FingerMotion {
        timestamp: u32,
        touch_id: i64,
        finger_id: i64,
        x: f32,
        y: f32,
        dx: f32,
        dy: f32,
        pressure: f32,
    },
    DollarGesture {
        timestamp: u32,
        touch_id: i64,
        gesture_id: i64,
        finger_count: u32,
        error: f32,
        x: f32,
        y: f32,
    },
    DollarRecord {
        timestamp: u32,
        touch_id: i64,
        gesture_id: i64,
    },
    MultiGesture {
        timestamp: u32,
        touch_id: i64,
        delta_theta: f32,
        delta_distance: f32,
        x: f32,
        y: f32,
        finger_count: u32,
    },
    ClipboardUpdate {
        timestamp: u32,
    },
    DropFile {
        timestamp: u32,
        window_id: u32,
        file: CString,
    },
    DropText {
        timestamp: u32,
        window_id: u32,
        text: CString,
    },
    DropBegin {
        timestamp: u32,
        window_id: u32,
    },
    DropComplete {
        timestamp: u32,
        window_id: u32,
    },
    AudioDeviceAdded {
        timestamp: u32,
        device_index: u32,
        is_capture: bool,
    },
    AudioDeviceRemoved {
        timestamp: u32,
        device_id: u32,
        is_capture: bool,
    },
    RenderTargetsReset {
        timestamp: u32,
    },
    RenderDeviceReset {
        timestamp: u32,
    },
    User(UserEvent),
}

#[allow(dead_code)]
impl Event {
    pub fn timestamp(&self) -> u32 {
        use self::Event::*;
        match self {
            Quit { timestamp } => *timestamp,
            AppTerminating { timestamp } => *timestamp,
            AppLowMemory { timestamp } => *timestamp,
            AppWillEnterBackground { timestamp } => *timestamp,
            AppDidEnterBackground { timestamp } => *timestamp,
            AppWillEnterForeground { timestamp } => *timestamp,
            AppDidEnterForeground { timestamp } => *timestamp,
            WindowShown { timestamp, .. } => *timestamp,
            WindowHidden { timestamp, .. } => *timestamp,
            WindowExposed { timestamp, .. } => *timestamp,
            WindowMoved { timestamp, .. } => *timestamp,
            WindowResized { timestamp, .. } => *timestamp,
            WindowSizeChanged { timestamp, .. } => *timestamp,
            WindowMinimized { timestamp, .. } => *timestamp,
            WindowMaximized { timestamp, .. } => *timestamp,
            WindowRestored { timestamp, .. } => *timestamp,
            MouseEntered { timestamp, .. } => *timestamp,
            MouseLeft { timestamp, .. } => *timestamp,
            KeyboardFocusGained { timestamp, .. } => *timestamp,
            KeyboardFocusLost { timestamp, .. } => *timestamp,
            WindowClose { timestamp, .. } => *timestamp,
            WindowTakeFocus { timestamp, .. } => *timestamp,
            WindowHitTest { timestamp, .. } => *timestamp,
            SysWMEvent { timestamp, .. } => *timestamp,
            KeyDown { timestamp, .. } => *timestamp,
            KeyUp { timestamp, .. } => *timestamp,
            TextEditing { timestamp, .. } => *timestamp,
            TextInput { timestamp, .. } => *timestamp,
            KeyMapChanged { timestamp } => *timestamp,
            MouseMotion { timestamp, .. } => *timestamp,
            MouseButtonDown { timestamp, .. } => *timestamp,
            MouseButtonUp { timestamp, .. } => *timestamp,
            MouseWheel { timestamp, .. } => *timestamp,
            JoystickAxisMotion { timestamp, .. } => *timestamp,
            JoystickBallMotion { timestamp, .. } => *timestamp,
            JoystickHatMotion { timestamp, .. } => *timestamp,
            JoystickButtonDown { timestamp, .. } => *timestamp,
            JoystickButtonUp { timestamp, .. } => *timestamp,
            JoystickDeviceAdded { timestamp, .. } => *timestamp,
            JoystickDeviceRemoved { timestamp, .. } => *timestamp,
            ControllerAxisMotion { timestamp, .. } => *timestamp,
            ControllerButtonDown { timestamp, .. } => *timestamp,
            ControllerButtonUp { timestamp, .. } => *timestamp,
            ControllerDeviceAdded { timestamp, .. } => *timestamp,
            ControllerDeviceRemoved { timestamp, .. } => *timestamp,
            ControllerDeviceRemapped { timestamp, .. } => *timestamp,
            FingerDown { timestamp, .. } => *timestamp,
            FingerUp { timestamp, .. } => *timestamp,
            FingerMotion { timestamp, .. } => *timestamp,
            DollarGesture { timestamp, .. } => *timestamp,
            DollarRecord { timestamp, .. } => *timestamp,
            MultiGesture { timestamp, .. } => *timestamp,
            ClipboardUpdate { timestamp } => *timestamp,
            DropFile { timestamp, .. } => *timestamp,
            DropText { timestamp, .. } => *timestamp,
            DropBegin { timestamp, .. } => *timestamp,
            DropComplete { timestamp, .. } => *timestamp,
            AudioDeviceAdded { timestamp, .. } => *timestamp,
            AudioDeviceRemoved { timestamp, .. } => *timestamp,
            RenderTargetsReset { timestamp } => *timestamp,
            RenderDeviceReset { timestamp } => *timestamp,
            User(event) => event.timestamp,
        }
    }
}

#[repr(C)]
pub struct EventSource(
    *const u8, // to prevent Send or Sync
);

pub unsafe fn make_event_source() -> EventSource {
    EventSource(null())
}

impl EventSource {
    pub fn poll(&self) -> Option<Event> {
        unsafe {
            let mut event = zeroed();
            if api::SDL_PollEvent(&mut event) != 0 {
                Some(event.into())
            } else {
                None
            }
        }
    }
    pub fn next(&self) -> Event {
        unsafe {
            let mut event = zeroed();
            if api::SDL_WaitEvent(&mut event) != 0 {
                event.into()
            } else {
                panic!("SDL_WaitEvent failed: {}", super::get_error())
            }
        }
    }
}

unsafe fn owned_c_string_to_string(c_string: *mut c_char) -> CString {
    struct FreeFile(*mut c_char);
    impl Drop for FreeFile {
        fn drop(&mut self) {
            unsafe {
                api::SDL_free(self.0 as *mut c_void);
            }
        }
    }
    let _ = FreeFile(c_string);
    CStr::from_ptr(c_string).into()
}

impl From<api::SDL_Event> for Event {
    fn from(event: api::SDL_Event) -> Self {
        use self::Event::*;
        unsafe {
            match event.type_ {
                api::SDL_QUIT => Quit {
                    timestamp: event.quit.timestamp,
                },
                api::SDL_APP_TERMINATING => AppTerminating {
                    timestamp: event.common.timestamp,
                },
                api::SDL_APP_LOWMEMORY => AppLowMemory {
                    timestamp: event.common.timestamp,
                },
                api::SDL_APP_WILLENTERBACKGROUND => AppWillEnterBackground {
                    timestamp: event.common.timestamp,
                },
                api::SDL_APP_DIDENTERBACKGROUND => AppDidEnterBackground {
                    timestamp: event.common.timestamp,
                },
                api::SDL_APP_WILLENTERFOREGROUND => AppWillEnterForeground {
                    timestamp: event.common.timestamp,
                },
                api::SDL_APP_DIDENTERFOREGROUND => AppDidEnterForeground {
                    timestamp: event.common.timestamp,
                },
                api::SDL_WINDOWEVENT => match event.window.event as u32 {
                    api::SDL_WINDOWEVENT_SHOWN => WindowShown {
                        timestamp: event.window.timestamp,
                        window_id: event.window.windowID,
                    },
                    api::SDL_WINDOWEVENT_HIDDEN => WindowHidden {
                        timestamp: event.window.timestamp,
                        window_id: event.window.windowID,
                    },
                    api::SDL_WINDOWEVENT_EXPOSED => WindowExposed {
                        timestamp: event.window.timestamp,
                        window_id: event.window.windowID,
                    },
                    api::SDL_WINDOWEVENT_MOVED => WindowMoved {
                        timestamp: event.window.timestamp,
                        window_id: event.window.windowID,
                        x: event.window.data1,
                        y: event.window.data2,
                    },
                    api::SDL_WINDOWEVENT_RESIZED => WindowResized {
                        timestamp: event.window.timestamp,
                        window_id: event.window.windowID,
                        w: event.window.data1 as u32,
                        h: event.window.data2 as u32,
                    },
                    api::SDL_WINDOWEVENT_SIZE_CHANGED => WindowSizeChanged {
                        timestamp: event.window.timestamp,
                        window_id: event.window.windowID,
                        w: event.window.data1 as u32,
                        h: event.window.data2 as u32,
                    },
                    api::SDL_WINDOWEVENT_MINIMIZED => WindowMinimized {
                        timestamp: event.window.timestamp,
                        window_id: event.window.windowID,
                    },
                    api::SDL_WINDOWEVENT_MAXIMIZED => WindowMaximized {
                        timestamp: event.window.timestamp,
                        window_id: event.window.windowID,
                    },
                    api::SDL_WINDOWEVENT_RESTORED => WindowRestored {
                        timestamp: event.window.timestamp,
                        window_id: event.window.windowID,
                    },
                    api::SDL_WINDOWEVENT_ENTER => MouseEntered {
                        timestamp: event.window.timestamp,
                        window_id: event.window.windowID,
                    },
                    api::SDL_WINDOWEVENT_LEAVE => MouseLeft {
                        timestamp: event.window.timestamp,
                        window_id: event.window.windowID,
                    },
                    api::SDL_WINDOWEVENT_FOCUS_GAINED => KeyboardFocusGained {
                        timestamp: event.window.timestamp,
                        window_id: event.window.windowID,
                    },
                    api::SDL_WINDOWEVENT_FOCUS_LOST => KeyboardFocusLost {
                        timestamp: event.window.timestamp,
                        window_id: event.window.windowID,
                    },
                    api::SDL_WINDOWEVENT_CLOSE => WindowClose {
                        timestamp: event.window.timestamp,
                        window_id: event.window.windowID,
                    },
                    api::SDL_WINDOWEVENT_TAKE_FOCUS => WindowTakeFocus {
                        timestamp: event.window.timestamp,
                        window_id: event.window.windowID,
                    },
                    api::SDL_WINDOWEVENT_HIT_TEST => WindowHitTest {
                        timestamp: event.window.timestamp,
                        window_id: event.window.windowID,
                    },
                    _ => panic!("unknown SDL window event type"),
                },
                api::SDL_SYSWMEVENT => SysWMEvent {
                    timestamp: event.syswm.timestamp,
                    msg: event.syswm.msg,
                },
                api::SDL_KEYDOWN => KeyDown {
                    timestamp: event.key.timestamp,
                    window_id: event.key.windowID,
                    repeat: event.key.repeat != 0,
                    scancode: transmute(event.key.keysym.scancode as u32),
                    keycode: make_keycode(event.key.keysym.sym),
                    modifiers: KeyModifiers(event.key.keysym.mod_),
                },
                api::SDL_KEYUP => KeyUp {
                    timestamp: event.key.timestamp,
                    window_id: event.key.windowID,
                    scancode: transmute(event.key.keysym.scancode as u32),
                    keycode: make_keycode(event.key.keysym.sym),
                    modifiers: KeyModifiers(event.key.keysym.mod_),
                },
                api::SDL_TEXTEDITING => TextEditing {
                    timestamp: event.edit.timestamp,
                    window_id: event.edit.windowID,
                    text: CStr::from_ptr(&event.edit.text as *const i8)
                        .to_string_lossy()
                        .into(),
                    start: event.edit.start as usize,
                },
                api::SDL_TEXTINPUT => TextInput {
                    timestamp: event.edit.timestamp,
                    window_id: event.edit.windowID,
                    text: CStr::from_ptr(&event.edit.text as *const i8)
                        .to_string_lossy()
                        .into(),
                },
                api::SDL_KEYMAPCHANGED => KeyMapChanged {
                    timestamp: event.common.timestamp,
                },
                api::SDL_MOUSEMOTION => MouseMotion {
                    timestamp: event.motion.timestamp,
                    window_id: event.motion.windowID,
                    which: MouseID(event.motion.which),
                    state: MouseButtons(event.motion.state),
                    x: event.motion.x,
                    y: event.motion.y,
                    x_relative: event.motion.xrel,
                    y_relative: event.motion.yrel,
                },
                api::SDL_MOUSEBUTTONDOWN => MouseButtonDown {
                    timestamp: event.button.timestamp,
                    window_id: event.button.windowID,
                    which: MouseID(event.button.which),
                    button: MouseButton(event.button.button),
                    x: event.button.x,
                    y: event.button.y,
                },
                api::SDL_MOUSEBUTTONUP => MouseButtonUp {
                    timestamp: event.button.timestamp,
                    window_id: event.button.windowID,
                    which: MouseID(event.button.which),
                    button: MouseButton(event.button.button),
                    x: event.button.x,
                    y: event.button.y,
                },
                api::SDL_MOUSEWHEEL => MouseWheel {
                    timestamp: event.wheel.timestamp,
                    window_id: event.wheel.windowID,
                    which: MouseID(event.wheel.which),
                    x: event.wheel.x,
                    y: event.wheel.y,
                    direction: match event.wheel.direction {
                        api::SDL_MOUSEWHEEL_FLIPPED => MouseWheelDirection::Flipped,
                        api::SDL_MOUSEWHEEL_NORMAL => MouseWheelDirection::Normal,
                        _ => unreachable!(),
                    },
                },
                api::SDL_JOYAXISMOTION => JoystickAxisMotion {
                    timestamp: event.jaxis.timestamp,
                    which: JoystickID::from_sdl(event.jaxis.which).unwrap(),
                    axis: event.jaxis.axis,
                    value: event.jaxis.value,
                },
                api::SDL_JOYBALLMOTION => JoystickBallMotion {
                    timestamp: event.jball.timestamp,
                    which: JoystickID::from_sdl(event.jball.which).unwrap(),
                    ball: event.jball.ball,
                    x_relative: event.jball.xrel,
                    y_relative: event.jball.yrel,
                },
                api::SDL_JOYHATMOTION => JoystickHatMotion {
                    timestamp: event.jhat.timestamp,
                    which: JoystickID::from_sdl(event.jhat.which).unwrap(),
                    hat: event.jhat.hat,
                    value: JoystickHatDirection::from_sdl(event.jhat.value),
                },
                api::SDL_JOYBUTTONDOWN => JoystickButtonDown {
                    timestamp: event.jbutton.timestamp,
                    which: JoystickID::from_sdl(event.jbutton.which).unwrap(),
                    button: event.jbutton.button,
                },
                api::SDL_JOYBUTTONUP => JoystickButtonUp {
                    timestamp: event.jbutton.timestamp,
                    which: JoystickID::from_sdl(event.jbutton.which).unwrap(),
                    button: event.jbutton.button,
                },
                api::SDL_JOYDEVICEADDED => JoystickDeviceAdded {
                    timestamp: event.jdevice.timestamp,
                    which: event.jdevice.which,
                },
                api::SDL_JOYDEVICEREMOVED => JoystickDeviceRemoved {
                    timestamp: event.jdevice.timestamp,
                    which: JoystickID::from_sdl(event.jdevice.which).unwrap(),
                },
                api::SDL_CONTROLLERAXISMOTION => ControllerAxisMotion {
                    timestamp: event.caxis.timestamp,
                    which: JoystickID::from_sdl(event.caxis.which).unwrap(),
                    axis: event.caxis.axis,
                    value: event.caxis.value,
                },
                api::SDL_CONTROLLERBUTTONDOWN => ControllerButtonDown {
                    timestamp: event.cbutton.timestamp,
                    which: JoystickID::from_sdl(event.cbutton.which).unwrap(),
                    button: event.cbutton.button,
                },
                api::SDL_CONTROLLERBUTTONUP => ControllerButtonUp {
                    timestamp: event.cbutton.timestamp,
                    which: JoystickID::from_sdl(event.cbutton.which).unwrap(),
                    button: event.cbutton.button,
                },
                api::SDL_CONTROLLERDEVICEADDED => ControllerDeviceAdded {
                    timestamp: event.cdevice.timestamp,
                    which: event.cdevice.which,
                },
                api::SDL_CONTROLLERDEVICEREMOVED => ControllerDeviceRemoved {
                    timestamp: event.cdevice.timestamp,
                    which: JoystickID::from_sdl(event.cdevice.which).unwrap(),
                },
                api::SDL_CONTROLLERDEVICEREMAPPED => ControllerDeviceRemapped {
                    timestamp: event.cdevice.timestamp,
                    which: JoystickID::from_sdl(event.cdevice.which).unwrap(),
                },
                api::SDL_FINGERDOWN => FingerDown {
                    timestamp: event.tfinger.timestamp,
                    touch_id: event.tfinger.touchId,
                    finger_id: event.tfinger.fingerId,
                    x: event.tfinger.x,
                    y: event.tfinger.y,
                    pressure: event.tfinger.pressure,
                },
                api::SDL_FINGERUP => FingerUp {
                    timestamp: event.tfinger.timestamp,
                    touch_id: event.tfinger.touchId,
                    finger_id: event.tfinger.fingerId,
                    x: event.tfinger.x,
                    y: event.tfinger.y,
                    pressure: event.tfinger.pressure,
                },
                api::SDL_FINGERMOTION => FingerMotion {
                    timestamp: event.tfinger.timestamp,
                    touch_id: event.tfinger.touchId,
                    finger_id: event.tfinger.fingerId,
                    x: event.tfinger.x,
                    y: event.tfinger.y,
                    dx: event.tfinger.dx,
                    dy: event.tfinger.dy,
                    pressure: event.tfinger.pressure,
                },
                api::SDL_DOLLARGESTURE => DollarGesture {
                    timestamp: event.dgesture.timestamp,
                    touch_id: event.dgesture.touchId,
                    gesture_id: event.dgesture.gestureId,
                    error: event.dgesture.error,
                    x: event.dgesture.x,
                    y: event.dgesture.y,
                    finger_count: event.dgesture.numFingers,
                },
                api::SDL_DOLLARRECORD => DollarRecord {
                    timestamp: event.dgesture.timestamp,
                    touch_id: event.dgesture.touchId,
                    gesture_id: event.dgesture.gestureId,
                },
                api::SDL_MULTIGESTURE => MultiGesture {
                    timestamp: event.mgesture.timestamp,
                    touch_id: event.mgesture.touchId,
                    delta_distance: event.mgesture.dDist,
                    delta_theta: event.mgesture.dTheta,
                    x: event.mgesture.x,
                    y: event.mgesture.y,
                    finger_count: event.mgesture.numFingers as u32,
                },
                api::SDL_CLIPBOARDUPDATE => ClipboardUpdate {
                    timestamp: event.common.timestamp,
                },
                api::SDL_DROPFILE => DropFile {
                    timestamp: event.drop.timestamp,
                    window_id: event.drop.windowID,
                    file: owned_c_string_to_string(event.drop.file),
                },
                api::SDL_DROPTEXT => DropText {
                    timestamp: event.drop.timestamp,
                    window_id: event.drop.windowID,
                    text: owned_c_string_to_string(event.drop.file),
                },
                api::SDL_DROPBEGIN => DropBegin {
                    timestamp: event.drop.timestamp,
                    window_id: event.drop.windowID,
                },
                api::SDL_DROPCOMPLETE => DropComplete {
                    timestamp: event.drop.timestamp,
                    window_id: event.drop.windowID,
                },
                api::SDL_AUDIODEVICEADDED => AudioDeviceAdded {
                    timestamp: event.adevice.timestamp,
                    device_index: event.adevice.which,
                    is_capture: event.adevice.iscapture != 0,
                },
                api::SDL_AUDIODEVICEREMOVED => AudioDeviceRemoved {
                    timestamp: event.adevice.timestamp,
                    device_id: event.adevice.which,
                    is_capture: event.adevice.iscapture != 0,
                },
                api::SDL_RENDER_TARGETS_RESET => RenderTargetsReset {
                    timestamp: event.common.timestamp,
                },
                api::SDL_RENDER_DEVICE_RESET => RenderDeviceReset {
                    timestamp: event.common.timestamp,
                },
                api::SDL_USEREVENT..=api::SDL_LASTEVENT => User(event.user),
                _ => panic!("unknown SDL event type"),
            }
        }
    }
}
