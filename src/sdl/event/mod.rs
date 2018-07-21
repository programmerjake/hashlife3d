mod scancode;
use super::api;
use std::ffi::*;
use std::fmt;
use std::mem::*;
use std::ops;

use self::scancode::make_keycode;
pub use self::scancode::Keycode;
pub use self::scancode::Scancode;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct KeyModifiers(pub u16);

impl KeyModifiers {
    pub const NONE: KeyModifiers = KeyModifiers(api::SDL_Keymod_KMOD_NONE as u16);
    pub const LSHIFT: KeyModifiers = KeyModifiers(api::SDL_Keymod_KMOD_LSHIFT as u16);
    pub const RSHIFT: KeyModifiers = KeyModifiers(api::SDL_Keymod_KMOD_RSHIFT as u16);
    pub const LCTRL: KeyModifiers = KeyModifiers(api::SDL_Keymod_KMOD_LCTRL as u16);
    pub const RCTRL: KeyModifiers = KeyModifiers(api::SDL_Keymod_KMOD_RCTRL as u16);
    pub const LALT: KeyModifiers = KeyModifiers(api::SDL_Keymod_KMOD_LALT as u16);
    pub const RALT: KeyModifiers = KeyModifiers(api::SDL_Keymod_KMOD_RALT as u16);
    pub const LGUI: KeyModifiers = KeyModifiers(api::SDL_Keymod_KMOD_LGUI as u16);
    pub const RGUI: KeyModifiers = KeyModifiers(api::SDL_Keymod_KMOD_RGUI as u16);
    pub const NUM: KeyModifiers = KeyModifiers(api::SDL_Keymod_KMOD_NUM as u16);
    pub const CAPS: KeyModifiers = KeyModifiers(api::SDL_Keymod_KMOD_CAPS as u16);
    pub const MODE: KeyModifiers = KeyModifiers(api::SDL_Keymod_KMOD_MODE as u16);
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
        let debug_set = f.debug_set();
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
pub struct MouseButton(u32);

impl MouseButton {
    pub const NONE: MouseButton = MouseButton(0);
    pub const LEFT: MouseButton = MouseButton(api::SDL_BUTTON_LEFT);
    pub const MIDDLE: MouseButton = MouseButton(api::SDL_BUTTON_MIDDLE);
    pub const RIGHT: MouseButton = MouseButton(api::SDL_BUTTON_RIGHT);
    pub const X1: MouseButton = MouseButton(api::SDL_BUTTON_X1);
    pub const X2: MouseButton = MouseButton(api::SDL_BUTTON_X2);
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
        let debug_set = f.debug_set();
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
    pub const TOUCH: MouseID = MouseID(!0);
}

pub enum Event {
    Quit {
        timestamp: u32,
    },
    AppTerminating,
    AppLowMemory,
    AppWillEnterBackground,
    AppDidEnterBackground,
    AppWillEnterForeground,
    AppDidEnterForeground,
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
    KeyMapChanged,
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
}

impl Event {
    pub fn timestamp(&self) -> Option<u32> {
        use self::Event::*;
        match self {
            Quit { timestamp } => Some(*timestamp),
            AppTerminating => None,
            AppLowMemory => None,
            AppWillEnterBackground => None,
            AppDidEnterBackground => None,
            AppWillEnterForeground => None,
            AppDidEnterForeground => None,
        }
    }
}

impl From<api::SDL_Event> for Event {
    fn from(event: api::SDL_Event) -> Self {
        use self::Event::*;
        unsafe {
            match event.type_ {
                api::SDL_EventType_SDL_QUIT => Quit {
                    timestamp: event.quit.timestamp,
                },
                api::SDL_EventType_SDL_APP_TERMINATING => AppTerminating,
                api::SDL_EventType_SDL_APP_LOWMEMORY => AppLowMemory,
                api::SDL_EventType_SDL_APP_WILLENTERBACKGROUND => AppWillEnterBackground,
                api::SDL_EventType_SDL_APP_DIDENTERBACKGROUND => AppDidEnterBackground,
                api::SDL_EventType_SDL_APP_WILLENTERFOREGROUND => AppWillEnterForeground,
                api::SDL_EventType_SDL_APP_DIDENTERFOREGROUND => AppDidEnterForeground,
                api::SDL_EventType_SDL_WINDOWEVENT => match event.window.event as u32 {
                    api::SDL_WindowEventID_SDL_WINDOWEVENT_SHOWN => WindowShown {
                        timestamp: event.window.timestamp,
                        window_id: event.window.windowID,
                    },
                    api::SDL_WindowEventID_SDL_WINDOWEVENT_HIDDEN => WindowHidden {
                        timestamp: event.window.timestamp,
                        window_id: event.window.windowID,
                    },
                    api::SDL_WindowEventID_SDL_WINDOWEVENT_EXPOSED => WindowExposed {
                        timestamp: event.window.timestamp,
                        window_id: event.window.windowID,
                    },
                    api::SDL_WindowEventID_SDL_WINDOWEVENT_MOVED => WindowMoved {
                        timestamp: event.window.timestamp,
                        window_id: event.window.windowID,
                        x: event.window.data1,
                        y: event.window.data2,
                    },
                    api::SDL_WindowEventID_SDL_WINDOWEVENT_RESIZED => WindowResized {
                        timestamp: event.window.timestamp,
                        window_id: event.window.windowID,
                        w: event.window.data1 as u32,
                        h: event.window.data2 as u32,
                    },
                    api::SDL_WindowEventID_SDL_WINDOWEVENT_SIZE_CHANGED => WindowSizeChanged {
                        timestamp: event.window.timestamp,
                        window_id: event.window.windowID,
                        w: event.window.data1 as u32,
                        h: event.window.data2 as u32,
                    },
                    api::SDL_WindowEventID_SDL_WINDOWEVENT_MINIMIZED => WindowMinimized {
                        timestamp: event.window.timestamp,
                        window_id: event.window.windowID,
                    },
                    api::SDL_WindowEventID_SDL_WINDOWEVENT_MAXIMIZED => WindowMaximized {
                        timestamp: event.window.timestamp,
                        window_id: event.window.windowID,
                    },
                    api::SDL_WindowEventID_SDL_WINDOWEVENT_RESTORED => WindowRestored {
                        timestamp: event.window.timestamp,
                        window_id: event.window.windowID,
                    },
                    api::SDL_WindowEventID_SDL_WINDOWEVENT_ENTER => MouseEntered {
                        timestamp: event.window.timestamp,
                        window_id: event.window.windowID,
                    },
                    api::SDL_WindowEventID_SDL_WINDOWEVENT_LEAVE => MouseLeft {
                        timestamp: event.window.timestamp,
                        window_id: event.window.windowID,
                    },
                    api::SDL_WindowEventID_SDL_WINDOWEVENT_FOCUS_GAINED => KeyboardFocusGained {
                        timestamp: event.window.timestamp,
                        window_id: event.window.windowID,
                    },
                    api::SDL_WindowEventID_SDL_WINDOWEVENT_FOCUS_LOST => KeyboardFocusLost {
                        timestamp: event.window.timestamp,
                        window_id: event.window.windowID,
                    },
                    api::SDL_WindowEventID_SDL_WINDOWEVENT_CLOSE => WindowClose {
                        timestamp: event.window.timestamp,
                        window_id: event.window.windowID,
                    },
                    api::SDL_WindowEventID_SDL_WINDOWEVENT_TAKE_FOCUS => WindowTakeFocus {
                        timestamp: event.window.timestamp,
                        window_id: event.window.windowID,
                    },
                    api::SDL_WindowEventID_SDL_WINDOWEVENT_HIT_TEST => WindowHitTest {
                        timestamp: event.window.timestamp,
                        window_id: event.window.windowID,
                    },
                    _ => panic!("unknown SDL window event type"),
                },
                api::SDL_EventType_SDL_SYSWMEVENT => SysWMEvent {
                    timestamp: event.syswm.timestamp,
                    msg: event.syswm.msg,
                },
                api::SDL_EventType_SDL_KEYDOWN => KeyDown {
                    timestamp: event.key.timestamp,
                    window_id: event.key.windowID,
                    repeat: event.key.repeat != 0,
                    scancode: unsafe { transmute(event.key.keysym.scancode as u32) },
                    keycode: make_keycode(event.key.keysym.sym),
                    modifiers: KeyModifiers(event.key.keysym.mod_),
                },
                api::SDL_EventType_SDL_KEYUP => KeyUp {
                    timestamp: event.key.timestamp,
                    window_id: event.key.windowID,
                    scancode: unsafe { transmute(event.key.keysym.scancode as u32) },
                    keycode: make_keycode(event.key.keysym.sym),
                    modifiers: KeyModifiers(event.key.keysym.mod_),
                },
                api::SDL_EventType_SDL_TEXTEDITING => TextEditing {
                    timestamp: event.edit.timestamp,
                    window_id: event.edit.windowID,
                    text: CStr::from_ptr(&event.edit.text as *const i8)
                        .to_string_lossy()
                        .into(),
                    start: event.edit.start as usize,
                },
                api::SDL_EventType_SDL_TEXTINPUT => TextInput {
                    timestamp: event.edit.timestamp,
                    window_id: event.edit.windowID,
                    text: CStr::from_ptr(&event.edit.text as *const i8)
                        .to_string_lossy()
                        .into(),
                },
                api::SDL_EventType_SDL_KEYMAPCHANGED => KeyMapChanged,
                api::SDL_EventType_SDL_MOUSEMOTION => MouseMotion {
                    timestamp: event.motion.timestamp,
                    window_id: event.motion.windowID,
                    which: event.motion.which,
                    state: event.motion.state,
                    x: event.motion.x,
                    y: event.motion.y,
                    x_relative: event.motion.xrel,
                    y_relative: event.motion.yrel,
                },
                api::SDL_EventType_SDL_MOUSEBUTTONDOWN => MOUSEBUTTONDOWN,
                api::SDL_EventType_SDL_MOUSEBUTTONUP => MOUSEBUTTONUP,
                api::SDL_EventType_SDL_MOUSEWHEEL => MOUSEWHEEL,
                api::SDL_EventType_SDL_JOYAXISMOTION => JOYAXISMOTION,
                api::SDL_EventType_SDL_JOYBALLMOTION => JOYBALLMOTION,
                api::SDL_EventType_SDL_JOYHATMOTION => JOYHATMOTION,
                api::SDL_EventType_SDL_JOYBUTTONDOWN => JOYBUTTONDOWN,
                api::SDL_EventType_SDL_JOYBUTTONUP => JOYBUTTONUP,
                api::SDL_EventType_SDL_JOYDEVICEADDED => JOYDEVICEADDED,
                api::SDL_EventType_SDL_JOYDEVICEREMOVED => JOYDEVICEREMOVED,
                api::SDL_EventType_SDL_CONTROLLERAXISMOTION => CONTROLLERAXISMOTION,
                api::SDL_EventType_SDL_CONTROLLERBUTTONDOWN => CONTROLLERBUTTONDOWN,
                api::SDL_EventType_SDL_CONTROLLERBUTTONUP => CONTROLLERBUTTONUP,
                api::SDL_EventType_SDL_CONTROLLERDEVICEADDED => CONTROLLERDEVICEADDED,
                api::SDL_EventType_SDL_CONTROLLERDEVICEREMOVED => CONTROLLERDEVICEREMOVED,
                api::SDL_EventType_SDL_CONTROLLERDEVICEREMAPPED => CONTROLLERDEVICEREMAPPED,
                api::SDL_EventType_SDL_FINGERDOWN => FINGERDOWN,
                api::SDL_EventType_SDL_FINGERUP => FINGERUP,
                api::SDL_EventType_SDL_FINGERMOTION => FINGERMOTION,
                api::SDL_EventType_SDL_DOLLARGESTURE => DOLLARGESTURE,
                api::SDL_EventType_SDL_DOLLARRECORD => DOLLARRECORD,
                api::SDL_EventType_SDL_MULTIGESTURE => MULTIGESTURE,
                api::SDL_EventType_SDL_CLIPBOARDUPDATE => CLIPBOARDUPDATE,
                api::SDL_EventType_SDL_DROPFILE => DROPFILE,
                api::SDL_EventType_SDL_DROPTEXT => DROPTEXT,
                api::SDL_EventType_SDL_DROPBEGIN => DROPBEGIN,
                api::SDL_EventType_SDL_DROPCOMPLETE => DROPCOMPLETE,
                api::SDL_EventType_SDL_AUDIODEVICEADDED => AUDIODEVICEADDED,
                api::SDL_EventType_SDL_AUDIODEVICEREMOVED => AUDIODEVICEREMOVED,
                api::SDL_EventType_SDL_RENDER_TARGETS_RESET => RENDER_TARGETS_RESET,
                api::SDL_EventType_SDL_RENDER_DEVICE_RESET => RENDER_DEVICE_RESET,
                api::SDL_EventType_SDL_USEREVENT => USEREVENT,
                _ => panic!("unknown SDL event type"),
            }
        }
    }
}
