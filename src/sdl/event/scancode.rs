use super::super::api;
use std::fmt;

#[repr(u32)]
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[allow(dead_code)]
pub enum Scancode {
    Unknown = api::SDL_SCANCODE_UNKNOWN,
    A = api::SDL_SCANCODE_A,
    B = api::SDL_SCANCODE_B,
    C = api::SDL_SCANCODE_C,
    D = api::SDL_SCANCODE_D,
    E = api::SDL_SCANCODE_E,
    F = api::SDL_SCANCODE_F,
    G = api::SDL_SCANCODE_G,
    H = api::SDL_SCANCODE_H,
    I = api::SDL_SCANCODE_I,
    J = api::SDL_SCANCODE_J,
    K = api::SDL_SCANCODE_K,
    L = api::SDL_SCANCODE_L,
    M = api::SDL_SCANCODE_M,
    N = api::SDL_SCANCODE_N,
    O = api::SDL_SCANCODE_O,
    P = api::SDL_SCANCODE_P,
    Q = api::SDL_SCANCODE_Q,
    R = api::SDL_SCANCODE_R,
    S = api::SDL_SCANCODE_S,
    T = api::SDL_SCANCODE_T,
    U = api::SDL_SCANCODE_U,
    V = api::SDL_SCANCODE_V,
    W = api::SDL_SCANCODE_W,
    X = api::SDL_SCANCODE_X,
    Y = api::SDL_SCANCODE_Y,
    Z = api::SDL_SCANCODE_Z,
    Num1 = api::SDL_SCANCODE_1,
    Num2 = api::SDL_SCANCODE_2,
    Num3 = api::SDL_SCANCODE_3,
    Num4 = api::SDL_SCANCODE_4,
    Num5 = api::SDL_SCANCODE_5,
    Num6 = api::SDL_SCANCODE_6,
    Num7 = api::SDL_SCANCODE_7,
    Num8 = api::SDL_SCANCODE_8,
    Num9 = api::SDL_SCANCODE_9,
    Num0 = api::SDL_SCANCODE_0,
    Return = api::SDL_SCANCODE_RETURN,
    Escape = api::SDL_SCANCODE_ESCAPE,
    Backspace = api::SDL_SCANCODE_BACKSPACE,
    Tab = api::SDL_SCANCODE_TAB,
    Space = api::SDL_SCANCODE_SPACE,
    Minus = api::SDL_SCANCODE_MINUS,
    Equals = api::SDL_SCANCODE_EQUALS,
    LeftBracket = api::SDL_SCANCODE_LEFTBRACKET,
    RightBracket = api::SDL_SCANCODE_RIGHTBRACKET,
    Backslash = api::SDL_SCANCODE_BACKSLASH,
    NonUSHash = api::SDL_SCANCODE_NONUSHASH,
    Semicolon = api::SDL_SCANCODE_SEMICOLON,
    Apostrophe = api::SDL_SCANCODE_APOSTROPHE,
    Grave = api::SDL_SCANCODE_GRAVE,
    Comma = api::SDL_SCANCODE_COMMA,
    Period = api::SDL_SCANCODE_PERIOD,
    Slash = api::SDL_SCANCODE_SLASH,
    CapsLock = api::SDL_SCANCODE_CAPSLOCK,
    F1 = api::SDL_SCANCODE_F1,
    F2 = api::SDL_SCANCODE_F2,
    F3 = api::SDL_SCANCODE_F3,
    F4 = api::SDL_SCANCODE_F4,
    F5 = api::SDL_SCANCODE_F5,
    F6 = api::SDL_SCANCODE_F6,
    F7 = api::SDL_SCANCODE_F7,
    F8 = api::SDL_SCANCODE_F8,
    F9 = api::SDL_SCANCODE_F9,
    F10 = api::SDL_SCANCODE_F10,
    F11 = api::SDL_SCANCODE_F11,
    F12 = api::SDL_SCANCODE_F12,
    PrintScreen = api::SDL_SCANCODE_PRINTSCREEN,
    ScrollLock = api::SDL_SCANCODE_SCROLLLOCK,
    Pause = api::SDL_SCANCODE_PAUSE,
    Insert = api::SDL_SCANCODE_INSERT,
    Home = api::SDL_SCANCODE_HOME,
    PageUp = api::SDL_SCANCODE_PAGEUP,
    Delete = api::SDL_SCANCODE_DELETE,
    End = api::SDL_SCANCODE_END,
    PageDown = api::SDL_SCANCODE_PAGEDOWN,
    Right = api::SDL_SCANCODE_RIGHT,
    Left = api::SDL_SCANCODE_LEFT,
    Down = api::SDL_SCANCODE_DOWN,
    Up = api::SDL_SCANCODE_UP,
    NumlockClear = api::SDL_SCANCODE_NUMLOCKCLEAR,
    KpDivide = api::SDL_SCANCODE_KP_DIVIDE,
    KpMultiply = api::SDL_SCANCODE_KP_MULTIPLY,
    KpMinus = api::SDL_SCANCODE_KP_MINUS,
    KpPlus = api::SDL_SCANCODE_KP_PLUS,
    KpEnter = api::SDL_SCANCODE_KP_ENTER,
    Kp1 = api::SDL_SCANCODE_KP_1,
    Kp2 = api::SDL_SCANCODE_KP_2,
    Kp3 = api::SDL_SCANCODE_KP_3,
    Kp4 = api::SDL_SCANCODE_KP_4,
    Kp5 = api::SDL_SCANCODE_KP_5,
    Kp6 = api::SDL_SCANCODE_KP_6,
    Kp7 = api::SDL_SCANCODE_KP_7,
    Kp8 = api::SDL_SCANCODE_KP_8,
    Kp9 = api::SDL_SCANCODE_KP_9,
    Kp0 = api::SDL_SCANCODE_KP_0,
    KpPeriod = api::SDL_SCANCODE_KP_PERIOD,
    NonUSBackslash = api::SDL_SCANCODE_NONUSBACKSLASH,
    Application = api::SDL_SCANCODE_APPLICATION,
    Power = api::SDL_SCANCODE_POWER,
    KpEquals = api::SDL_SCANCODE_KP_EQUALS,
    F13 = api::SDL_SCANCODE_F13,
    F14 = api::SDL_SCANCODE_F14,
    F15 = api::SDL_SCANCODE_F15,
    F16 = api::SDL_SCANCODE_F16,
    F17 = api::SDL_SCANCODE_F17,
    F18 = api::SDL_SCANCODE_F18,
    F19 = api::SDL_SCANCODE_F19,
    F20 = api::SDL_SCANCODE_F20,
    F21 = api::SDL_SCANCODE_F21,
    F22 = api::SDL_SCANCODE_F22,
    F23 = api::SDL_SCANCODE_F23,
    F24 = api::SDL_SCANCODE_F24,
    Execute = api::SDL_SCANCODE_EXECUTE,
    Help = api::SDL_SCANCODE_HELP,
    Menu = api::SDL_SCANCODE_MENU,
    Select = api::SDL_SCANCODE_SELECT,
    Stop = api::SDL_SCANCODE_STOP,
    Again = api::SDL_SCANCODE_AGAIN,
    Undo = api::SDL_SCANCODE_UNDO,
    Cut = api::SDL_SCANCODE_CUT,
    Copy = api::SDL_SCANCODE_COPY,
    Paste = api::SDL_SCANCODE_PASTE,
    Find = api::SDL_SCANCODE_FIND,
    Mute = api::SDL_SCANCODE_MUTE,
    VolumeUp = api::SDL_SCANCODE_VOLUMEUP,
    VolumeDown = api::SDL_SCANCODE_VOLUMEDOWN,
    KpComma = api::SDL_SCANCODE_KP_COMMA,
    KpEqualsAS400 = api::SDL_SCANCODE_KP_EQUALSAS400,
    International1 = api::SDL_SCANCODE_INTERNATIONAL1,
    International2 = api::SDL_SCANCODE_INTERNATIONAL2,
    International3 = api::SDL_SCANCODE_INTERNATIONAL3,
    International4 = api::SDL_SCANCODE_INTERNATIONAL4,
    International5 = api::SDL_SCANCODE_INTERNATIONAL5,
    International6 = api::SDL_SCANCODE_INTERNATIONAL6,
    International7 = api::SDL_SCANCODE_INTERNATIONAL7,
    International8 = api::SDL_SCANCODE_INTERNATIONAL8,
    International9 = api::SDL_SCANCODE_INTERNATIONAL9,
    Lang1 = api::SDL_SCANCODE_LANG1,
    Lang2 = api::SDL_SCANCODE_LANG2,
    Lang3 = api::SDL_SCANCODE_LANG3,
    Lang4 = api::SDL_SCANCODE_LANG4,
    Lang5 = api::SDL_SCANCODE_LANG5,
    Lang6 = api::SDL_SCANCODE_LANG6,
    Lang7 = api::SDL_SCANCODE_LANG7,
    Lang8 = api::SDL_SCANCODE_LANG8,
    Lang9 = api::SDL_SCANCODE_LANG9,
    AltErase = api::SDL_SCANCODE_ALTERASE,
    SysReq = api::SDL_SCANCODE_SYSREQ,
    Cancel = api::SDL_SCANCODE_CANCEL,
    Clear = api::SDL_SCANCODE_CLEAR,
    Prior = api::SDL_SCANCODE_PRIOR,
    Return2 = api::SDL_SCANCODE_RETURN2,
    Separator = api::SDL_SCANCODE_SEPARATOR,
    Out = api::SDL_SCANCODE_OUT,
    Oper = api::SDL_SCANCODE_OPER,
    ClearAgain = api::SDL_SCANCODE_CLEARAGAIN,
    CrSel = api::SDL_SCANCODE_CRSEL,
    ExSel = api::SDL_SCANCODE_EXSEL,
    Kp00 = api::SDL_SCANCODE_KP_00,
    Kp000 = api::SDL_SCANCODE_KP_000,
    ThousandsSeparator = api::SDL_SCANCODE_THOUSANDSSEPARATOR,
    DecimalSeparator = api::SDL_SCANCODE_DECIMALSEPARATOR,
    CurrencyUnit = api::SDL_SCANCODE_CURRENCYUNIT,
    CurrencySubunit = api::SDL_SCANCODE_CURRENCYSUBUNIT,
    KpLeftParen = api::SDL_SCANCODE_KP_LEFTPAREN,
    KpRightParen = api::SDL_SCANCODE_KP_RIGHTPAREN,
    KpLeftBrace = api::SDL_SCANCODE_KP_LEFTBRACE,
    KpRightBrace = api::SDL_SCANCODE_KP_RIGHTBRACE,
    KpTab = api::SDL_SCANCODE_KP_TAB,
    KpBackspace = api::SDL_SCANCODE_KP_BACKSPACE,
    KpA = api::SDL_SCANCODE_KP_A,
    KpB = api::SDL_SCANCODE_KP_B,
    KpC = api::SDL_SCANCODE_KP_C,
    KpD = api::SDL_SCANCODE_KP_D,
    KpE = api::SDL_SCANCODE_KP_E,
    KpF = api::SDL_SCANCODE_KP_F,
    KpXor = api::SDL_SCANCODE_KP_XOR,
    KpPower = api::SDL_SCANCODE_KP_POWER,
    KpPercent = api::SDL_SCANCODE_KP_PERCENT,
    KpLess = api::SDL_SCANCODE_KP_LESS,
    KpGreater = api::SDL_SCANCODE_KP_GREATER,
    KpAmpersand = api::SDL_SCANCODE_KP_AMPERSAND,
    KpDblAmpersand = api::SDL_SCANCODE_KP_DBLAMPERSAND,
    KpVerticalBar = api::SDL_SCANCODE_KP_VERTICALBAR,
    KpDblVerticalBar = api::SDL_SCANCODE_KP_DBLVERTICALBAR,
    KpColon = api::SDL_SCANCODE_KP_COLON,
    KpHash = api::SDL_SCANCODE_KP_HASH,
    KpSpace = api::SDL_SCANCODE_KP_SPACE,
    KpAt = api::SDL_SCANCODE_KP_AT,
    KpExclam = api::SDL_SCANCODE_KP_EXCLAM,
    KpMemStore = api::SDL_SCANCODE_KP_MEMSTORE,
    KpMemRecall = api::SDL_SCANCODE_KP_MEMRECALL,
    KpMemClear = api::SDL_SCANCODE_KP_MEMCLEAR,
    KpMemAdd = api::SDL_SCANCODE_KP_MEMADD,
    KpMemSubtract = api::SDL_SCANCODE_KP_MEMSUBTRACT,
    KpMemMultiply = api::SDL_SCANCODE_KP_MEMMULTIPLY,
    KpMemDivide = api::SDL_SCANCODE_KP_MEMDIVIDE,
    KpPlusMinus = api::SDL_SCANCODE_KP_PLUSMINUS,
    KpClear = api::SDL_SCANCODE_KP_CLEAR,
    KpClearEntry = api::SDL_SCANCODE_KP_CLEARENTRY,
    KpBinary = api::SDL_SCANCODE_KP_BINARY,
    KpOctal = api::SDL_SCANCODE_KP_OCTAL,
    KpDecimal = api::SDL_SCANCODE_KP_DECIMAL,
    KpHexadecimal = api::SDL_SCANCODE_KP_HEXADECIMAL,
    LCtrl = api::SDL_SCANCODE_LCTRL,
    LShift = api::SDL_SCANCODE_LSHIFT,
    LAlt = api::SDL_SCANCODE_LALT,
    LGui = api::SDL_SCANCODE_LGUI,
    RCtrl = api::SDL_SCANCODE_RCTRL,
    RShift = api::SDL_SCANCODE_RSHIFT,
    RAlt = api::SDL_SCANCODE_RALT,
    RGui = api::SDL_SCANCODE_RGUI,
    Mode = api::SDL_SCANCODE_MODE,
    AudioNext = api::SDL_SCANCODE_AUDIONEXT,
    AudioPrev = api::SDL_SCANCODE_AUDIOPREV,
    AudioStop = api::SDL_SCANCODE_AUDIOSTOP,
    AudioPlay = api::SDL_SCANCODE_AUDIOPLAY,
    AudioMute = api::SDL_SCANCODE_AUDIOMUTE,
    MediaSelect = api::SDL_SCANCODE_MEDIASELECT,
    WWW = api::SDL_SCANCODE_WWW,
    Mail = api::SDL_SCANCODE_MAIL,
    Calculator = api::SDL_SCANCODE_CALCULATOR,
    Computer = api::SDL_SCANCODE_COMPUTER,
    AcSearch = api::SDL_SCANCODE_AC_SEARCH,
    AcHome = api::SDL_SCANCODE_AC_HOME,
    AcBack = api::SDL_SCANCODE_AC_BACK,
    AcForward = api::SDL_SCANCODE_AC_FORWARD,
    AcStop = api::SDL_SCANCODE_AC_STOP,
    AcRefresh = api::SDL_SCANCODE_AC_REFRESH,
    AcBookmarks = api::SDL_SCANCODE_AC_BOOKMARKS,
    BrightnessDown = api::SDL_SCANCODE_BRIGHTNESSDOWN,
    BrightnessUp = api::SDL_SCANCODE_BRIGHTNESSUP,
    DisplaySwitch = api::SDL_SCANCODE_DISPLAYSWITCH,
    KbdIllumToggle = api::SDL_SCANCODE_KBDILLUMTOGGLE,
    KbdIllumDown = api::SDL_SCANCODE_KBDILLUMDOWN,
    KbdIllumUp = api::SDL_SCANCODE_KBDILLUMUP,
    Eject = api::SDL_SCANCODE_EJECT,
    Sleep = api::SDL_SCANCODE_SLEEP,
    App1 = api::SDL_SCANCODE_APP1,
    App2 = api::SDL_SCANCODE_APP2,
    AudioRewind = api::SDL_SCANCODE_AUDIOREWIND,
    AudioFastForward = api::SDL_SCANCODE_AUDIOFASTFORWARD,
}

#[derive(Eq, PartialEq, Copy, Clone, Hash)]
pub struct Keycode(i32);

const SCANCODE_MASK: i32 = 1 << 30;

pub fn make_keycode(v: i32) -> Keycode {
    Keycode(v)
}

#[allow(dead_code)]
impl Keycode {
    pub const UNKNOWN: Keycode = Keycode(0);
    pub const RETURN: Keycode = Keycode('\r' as i32);
    pub const ESCAPE: Keycode = Keycode(0x1B);
    pub const BACKSPACE: Keycode = Keycode(8);
    pub const TAB: Keycode = Keycode('\t' as i32);
    pub const SPACE: Keycode = Keycode(' ' as i32);
    pub const EXCLAIM: Keycode = Keycode('!' as i32);
    pub const QUOTEDBL: Keycode = Keycode('"' as i32);
    pub const HASH: Keycode = Keycode('#' as i32);
    pub const PERCENT: Keycode = Keycode('%' as i32);
    pub const DOLLAR: Keycode = Keycode('$' as i32);
    pub const AMPERSAND: Keycode = Keycode('&' as i32);
    pub const QUOTE: Keycode = Keycode('\'' as i32);
    pub const LEFTPAREN: Keycode = Keycode('(' as i32);
    pub const RIGHTPAREN: Keycode = Keycode(')' as i32);
    pub const ASTERISK: Keycode = Keycode('*' as i32);
    pub const PLUS: Keycode = Keycode('+' as i32);
    pub const COMMA: Keycode = Keycode(',' as i32);
    pub const MINUS: Keycode = Keycode('-' as i32);
    pub const PERIOD: Keycode = Keycode('.' as i32);
    pub const SLASH: Keycode = Keycode('/' as i32);
    pub const NUM_0: Keycode = Keycode('0' as i32);
    pub const NUM_1: Keycode = Keycode('1' as i32);
    pub const NUM_2: Keycode = Keycode('2' as i32);
    pub const NUM_3: Keycode = Keycode('3' as i32);
    pub const NUM_4: Keycode = Keycode('4' as i32);
    pub const NUM_5: Keycode = Keycode('5' as i32);
    pub const NUM_6: Keycode = Keycode('6' as i32);
    pub const NUM_7: Keycode = Keycode('7' as i32);
    pub const NUM_8: Keycode = Keycode('8' as i32);
    pub const NUM_9: Keycode = Keycode('9' as i32);
    pub const COLON: Keycode = Keycode(':' as i32);
    pub const SEMICOLON: Keycode = Keycode(';' as i32);
    pub const LESS: Keycode = Keycode('<' as i32);
    pub const EQUALS: Keycode = Keycode('=' as i32);
    pub const GREATER: Keycode = Keycode('>' as i32);
    pub const QUESTION: Keycode = Keycode('?' as i32);
    pub const AT: Keycode = Keycode('@' as i32);
    pub const LEFTBRACKET: Keycode = Keycode('[' as i32);
    pub const BACKSLASH: Keycode = Keycode('\\' as i32);
    pub const RIGHTBRACKET: Keycode = Keycode(']' as i32);
    pub const CARET: Keycode = Keycode('^' as i32);
    pub const UNDERSCORE: Keycode = Keycode('_' as i32);
    pub const BACKQUOTE: Keycode = Keycode('`' as i32);
    pub const A: Keycode = Keycode('a' as i32);
    pub const B: Keycode = Keycode('b' as i32);
    pub const C: Keycode = Keycode('c' as i32);
    pub const D: Keycode = Keycode('d' as i32);
    pub const E: Keycode = Keycode('e' as i32);
    pub const F: Keycode = Keycode('f' as i32);
    pub const G: Keycode = Keycode('g' as i32);
    pub const H: Keycode = Keycode('h' as i32);
    pub const I: Keycode = Keycode('i' as i32);
    pub const J: Keycode = Keycode('j' as i32);
    pub const K: Keycode = Keycode('k' as i32);
    pub const L: Keycode = Keycode('l' as i32);
    pub const M: Keycode = Keycode('m' as i32);
    pub const N: Keycode = Keycode('n' as i32);
    pub const O: Keycode = Keycode('o' as i32);
    pub const P: Keycode = Keycode('p' as i32);
    pub const Q: Keycode = Keycode('q' as i32);
    pub const R: Keycode = Keycode('r' as i32);
    pub const S: Keycode = Keycode('s' as i32);
    pub const T: Keycode = Keycode('t' as i32);
    pub const U: Keycode = Keycode('u' as i32);
    pub const V: Keycode = Keycode('v' as i32);
    pub const W: Keycode = Keycode('w' as i32);
    pub const X: Keycode = Keycode('x' as i32);
    pub const Y: Keycode = Keycode('y' as i32);
    pub const Z: Keycode = Keycode('z' as i32);
    pub const CAPSLOCK: Keycode = Keycode(api::SDL_SCANCODE_CAPSLOCK as i32 | SCANCODE_MASK);
    pub const F1: Keycode = Keycode(api::SDL_SCANCODE_F1 as i32 | SCANCODE_MASK);
    pub const F2: Keycode = Keycode(api::SDL_SCANCODE_F2 as i32 | SCANCODE_MASK);
    pub const F3: Keycode = Keycode(api::SDL_SCANCODE_F3 as i32 | SCANCODE_MASK);
    pub const F4: Keycode = Keycode(api::SDL_SCANCODE_F4 as i32 | SCANCODE_MASK);
    pub const F5: Keycode = Keycode(api::SDL_SCANCODE_F5 as i32 | SCANCODE_MASK);
    pub const F6: Keycode = Keycode(api::SDL_SCANCODE_F6 as i32 | SCANCODE_MASK);
    pub const F7: Keycode = Keycode(api::SDL_SCANCODE_F7 as i32 | SCANCODE_MASK);
    pub const F8: Keycode = Keycode(api::SDL_SCANCODE_F8 as i32 | SCANCODE_MASK);
    pub const F9: Keycode = Keycode(api::SDL_SCANCODE_F9 as i32 | SCANCODE_MASK);
    pub const F10: Keycode = Keycode(api::SDL_SCANCODE_F10 as i32 | SCANCODE_MASK);
    pub const F11: Keycode = Keycode(api::SDL_SCANCODE_F11 as i32 | SCANCODE_MASK);
    pub const F12: Keycode = Keycode(api::SDL_SCANCODE_F12 as i32 | SCANCODE_MASK);
    pub const PRINTSCREEN: Keycode = Keycode(api::SDL_SCANCODE_PRINTSCREEN as i32 | SCANCODE_MASK);
    pub const SCROLLLOCK: Keycode = Keycode(api::SDL_SCANCODE_SCROLLLOCK as i32 | SCANCODE_MASK);
    pub const PAUSE: Keycode = Keycode(api::SDL_SCANCODE_PAUSE as i32 | SCANCODE_MASK);
    pub const INSERT: Keycode = Keycode(api::SDL_SCANCODE_INSERT as i32 | SCANCODE_MASK);
    pub const HOME: Keycode = Keycode(api::SDL_SCANCODE_HOME as i32 | SCANCODE_MASK);
    pub const PAGEUP: Keycode = Keycode(api::SDL_SCANCODE_PAGEUP as i32 | SCANCODE_MASK);
    pub const DELETE: Keycode = Keycode(0x7F);
    pub const END: Keycode = Keycode(api::SDL_SCANCODE_END as i32 | SCANCODE_MASK);
    pub const PAGEDOWN: Keycode = Keycode(api::SDL_SCANCODE_PAGEDOWN as i32 | SCANCODE_MASK);
    pub const RIGHT: Keycode = Keycode(api::SDL_SCANCODE_RIGHT as i32 | SCANCODE_MASK);
    pub const LEFT: Keycode = Keycode(api::SDL_SCANCODE_LEFT as i32 | SCANCODE_MASK);
    pub const DOWN: Keycode = Keycode(api::SDL_SCANCODE_DOWN as i32 | SCANCODE_MASK);
    pub const UP: Keycode = Keycode(api::SDL_SCANCODE_UP as i32 | SCANCODE_MASK);
    pub const NUMLOCKCLEAR: Keycode =
        Keycode(api::SDL_SCANCODE_NUMLOCKCLEAR as i32 | SCANCODE_MASK);
    pub const KP_DIVIDE: Keycode = Keycode(api::SDL_SCANCODE_KP_DIVIDE as i32 | SCANCODE_MASK);
    pub const KP_MULTIPLY: Keycode = Keycode(api::SDL_SCANCODE_KP_MULTIPLY as i32 | SCANCODE_MASK);
    pub const KP_MINUS: Keycode = Keycode(api::SDL_SCANCODE_KP_MINUS as i32 | SCANCODE_MASK);
    pub const KP_PLUS: Keycode = Keycode(api::SDL_SCANCODE_KP_PLUS as i32 | SCANCODE_MASK);
    pub const KP_ENTER: Keycode = Keycode(api::SDL_SCANCODE_KP_ENTER as i32 | SCANCODE_MASK);
    pub const KP_1: Keycode = Keycode(api::SDL_SCANCODE_KP_1 as i32 | SCANCODE_MASK);
    pub const KP_2: Keycode = Keycode(api::SDL_SCANCODE_KP_2 as i32 | SCANCODE_MASK);
    pub const KP_3: Keycode = Keycode(api::SDL_SCANCODE_KP_3 as i32 | SCANCODE_MASK);
    pub const KP_4: Keycode = Keycode(api::SDL_SCANCODE_KP_4 as i32 | SCANCODE_MASK);
    pub const KP_5: Keycode = Keycode(api::SDL_SCANCODE_KP_5 as i32 | SCANCODE_MASK);
    pub const KP_6: Keycode = Keycode(api::SDL_SCANCODE_KP_6 as i32 | SCANCODE_MASK);
    pub const KP_7: Keycode = Keycode(api::SDL_SCANCODE_KP_7 as i32 | SCANCODE_MASK);
    pub const KP_8: Keycode = Keycode(api::SDL_SCANCODE_KP_8 as i32 | SCANCODE_MASK);
    pub const KP_9: Keycode = Keycode(api::SDL_SCANCODE_KP_9 as i32 | SCANCODE_MASK);
    pub const KP_0: Keycode = Keycode(api::SDL_SCANCODE_KP_0 as i32 | SCANCODE_MASK);
    pub const KP_PERIOD: Keycode = Keycode(api::SDL_SCANCODE_KP_PERIOD as i32 | SCANCODE_MASK);
    pub const APPLICATION: Keycode = Keycode(api::SDL_SCANCODE_APPLICATION as i32 | SCANCODE_MASK);
    pub const POWER: Keycode = Keycode(api::SDL_SCANCODE_POWER as i32 | SCANCODE_MASK);
    pub const KP_EQUALS: Keycode = Keycode(api::SDL_SCANCODE_KP_EQUALS as i32 | SCANCODE_MASK);
    pub const F13: Keycode = Keycode(api::SDL_SCANCODE_F13 as i32 | SCANCODE_MASK);
    pub const F14: Keycode = Keycode(api::SDL_SCANCODE_F14 as i32 | SCANCODE_MASK);
    pub const F15: Keycode = Keycode(api::SDL_SCANCODE_F15 as i32 | SCANCODE_MASK);
    pub const F16: Keycode = Keycode(api::SDL_SCANCODE_F16 as i32 | SCANCODE_MASK);
    pub const F17: Keycode = Keycode(api::SDL_SCANCODE_F17 as i32 | SCANCODE_MASK);
    pub const F18: Keycode = Keycode(api::SDL_SCANCODE_F18 as i32 | SCANCODE_MASK);
    pub const F19: Keycode = Keycode(api::SDL_SCANCODE_F19 as i32 | SCANCODE_MASK);
    pub const F20: Keycode = Keycode(api::SDL_SCANCODE_F20 as i32 | SCANCODE_MASK);
    pub const F21: Keycode = Keycode(api::SDL_SCANCODE_F21 as i32 | SCANCODE_MASK);
    pub const F22: Keycode = Keycode(api::SDL_SCANCODE_F22 as i32 | SCANCODE_MASK);
    pub const F23: Keycode = Keycode(api::SDL_SCANCODE_F23 as i32 | SCANCODE_MASK);
    pub const F24: Keycode = Keycode(api::SDL_SCANCODE_F24 as i32 | SCANCODE_MASK);
    pub const EXECUTE: Keycode = Keycode(api::SDL_SCANCODE_EXECUTE as i32 | SCANCODE_MASK);
    pub const HELP: Keycode = Keycode(api::SDL_SCANCODE_HELP as i32 | SCANCODE_MASK);
    pub const MENU: Keycode = Keycode(api::SDL_SCANCODE_MENU as i32 | SCANCODE_MASK);
    pub const SELECT: Keycode = Keycode(api::SDL_SCANCODE_SELECT as i32 | SCANCODE_MASK);
    pub const STOP: Keycode = Keycode(api::SDL_SCANCODE_STOP as i32 | SCANCODE_MASK);
    pub const AGAIN: Keycode = Keycode(api::SDL_SCANCODE_AGAIN as i32 | SCANCODE_MASK);
    pub const UNDO: Keycode = Keycode(api::SDL_SCANCODE_UNDO as i32 | SCANCODE_MASK);
    pub const CUT: Keycode = Keycode(api::SDL_SCANCODE_CUT as i32 | SCANCODE_MASK);
    pub const COPY: Keycode = Keycode(api::SDL_SCANCODE_COPY as i32 | SCANCODE_MASK);
    pub const PASTE: Keycode = Keycode(api::SDL_SCANCODE_PASTE as i32 | SCANCODE_MASK);
    pub const FIND: Keycode = Keycode(api::SDL_SCANCODE_FIND as i32 | SCANCODE_MASK);
    pub const MUTE: Keycode = Keycode(api::SDL_SCANCODE_MUTE as i32 | SCANCODE_MASK);
    pub const VOLUMEUP: Keycode = Keycode(api::SDL_SCANCODE_VOLUMEUP as i32 | SCANCODE_MASK);
    pub const VOLUMEDOWN: Keycode = Keycode(api::SDL_SCANCODE_VOLUMEDOWN as i32 | SCANCODE_MASK);
    pub const KP_COMMA: Keycode = Keycode(api::SDL_SCANCODE_KP_COMMA as i32 | SCANCODE_MASK);
    pub const KP_EQUALSAS400: Keycode =
        Keycode(api::SDL_SCANCODE_KP_EQUALSAS400 as i32 | SCANCODE_MASK);
    pub const ALTERASE: Keycode = Keycode(api::SDL_SCANCODE_ALTERASE as i32 | SCANCODE_MASK);
    pub const SYSREQ: Keycode = Keycode(api::SDL_SCANCODE_SYSREQ as i32 | SCANCODE_MASK);
    pub const CANCEL: Keycode = Keycode(api::SDL_SCANCODE_CANCEL as i32 | SCANCODE_MASK);
    pub const CLEAR: Keycode = Keycode(api::SDL_SCANCODE_CLEAR as i32 | SCANCODE_MASK);
    pub const PRIOR: Keycode = Keycode(api::SDL_SCANCODE_PRIOR as i32 | SCANCODE_MASK);
    pub const RETURN2: Keycode = Keycode(api::SDL_SCANCODE_RETURN2 as i32 | SCANCODE_MASK);
    pub const SEPARATOR: Keycode = Keycode(api::SDL_SCANCODE_SEPARATOR as i32 | SCANCODE_MASK);
    pub const OUT: Keycode = Keycode(api::SDL_SCANCODE_OUT as i32 | SCANCODE_MASK);
    pub const OPER: Keycode = Keycode(api::SDL_SCANCODE_OPER as i32 | SCANCODE_MASK);
    pub const CLEARAGAIN: Keycode = Keycode(api::SDL_SCANCODE_CLEARAGAIN as i32 | SCANCODE_MASK);
    pub const CRSEL: Keycode = Keycode(api::SDL_SCANCODE_CRSEL as i32 | SCANCODE_MASK);
    pub const EXSEL: Keycode = Keycode(api::SDL_SCANCODE_EXSEL as i32 | SCANCODE_MASK);
    pub const KP_00: Keycode = Keycode(api::SDL_SCANCODE_KP_00 as i32 | SCANCODE_MASK);
    pub const KP_000: Keycode = Keycode(api::SDL_SCANCODE_KP_000 as i32 | SCANCODE_MASK);
    pub const THOUSANDSSEPARATOR: Keycode =
        Keycode(api::SDL_SCANCODE_THOUSANDSSEPARATOR as i32 | SCANCODE_MASK);
    pub const DECIMALSEPARATOR: Keycode =
        Keycode(api::SDL_SCANCODE_DECIMALSEPARATOR as i32 | SCANCODE_MASK);
    pub const CURRENCYUNIT: Keycode =
        Keycode(api::SDL_SCANCODE_CURRENCYUNIT as i32 | SCANCODE_MASK);
    pub const CURRENCYSUBUNIT: Keycode =
        Keycode(api::SDL_SCANCODE_CURRENCYSUBUNIT as i32 | SCANCODE_MASK);
    pub const KP_LEFTPAREN: Keycode =
        Keycode(api::SDL_SCANCODE_KP_LEFTPAREN as i32 | SCANCODE_MASK);
    pub const KP_RIGHTPAREN: Keycode =
        Keycode(api::SDL_SCANCODE_KP_RIGHTPAREN as i32 | SCANCODE_MASK);
    pub const KP_LEFTBRACE: Keycode =
        Keycode(api::SDL_SCANCODE_KP_LEFTBRACE as i32 | SCANCODE_MASK);
    pub const KP_RIGHTBRACE: Keycode =
        Keycode(api::SDL_SCANCODE_KP_RIGHTBRACE as i32 | SCANCODE_MASK);
    pub const KP_TAB: Keycode = Keycode(api::SDL_SCANCODE_KP_TAB as i32 | SCANCODE_MASK);
    pub const KP_BACKSPACE: Keycode =
        Keycode(api::SDL_SCANCODE_KP_BACKSPACE as i32 | SCANCODE_MASK);
    pub const KP_A: Keycode = Keycode(api::SDL_SCANCODE_KP_A as i32 | SCANCODE_MASK);
    pub const KP_B: Keycode = Keycode(api::SDL_SCANCODE_KP_B as i32 | SCANCODE_MASK);
    pub const KP_C: Keycode = Keycode(api::SDL_SCANCODE_KP_C as i32 | SCANCODE_MASK);
    pub const KP_D: Keycode = Keycode(api::SDL_SCANCODE_KP_D as i32 | SCANCODE_MASK);
    pub const KP_E: Keycode = Keycode(api::SDL_SCANCODE_KP_E as i32 | SCANCODE_MASK);
    pub const KP_F: Keycode = Keycode(api::SDL_SCANCODE_KP_F as i32 | SCANCODE_MASK);
    pub const KP_XOR: Keycode = Keycode(api::SDL_SCANCODE_KP_XOR as i32 | SCANCODE_MASK);
    pub const KP_POWER: Keycode = Keycode(api::SDL_SCANCODE_KP_POWER as i32 | SCANCODE_MASK);
    pub const KP_PERCENT: Keycode = Keycode(api::SDL_SCANCODE_KP_PERCENT as i32 | SCANCODE_MASK);
    pub const KP_LESS: Keycode = Keycode(api::SDL_SCANCODE_KP_LESS as i32 | SCANCODE_MASK);
    pub const KP_GREATER: Keycode = Keycode(api::SDL_SCANCODE_KP_GREATER as i32 | SCANCODE_MASK);
    pub const KP_AMPERSAND: Keycode =
        Keycode(api::SDL_SCANCODE_KP_AMPERSAND as i32 | SCANCODE_MASK);
    pub const KP_DBLAMPERSAND: Keycode =
        Keycode(api::SDL_SCANCODE_KP_DBLAMPERSAND as i32 | SCANCODE_MASK);
    pub const KP_VERTICALBAR: Keycode =
        Keycode(api::SDL_SCANCODE_KP_VERTICALBAR as i32 | SCANCODE_MASK);
    pub const KP_DBLVERTICALBAR: Keycode =
        Keycode(api::SDL_SCANCODE_KP_DBLVERTICALBAR as i32 | SCANCODE_MASK);
    pub const KP_COLON: Keycode = Keycode(api::SDL_SCANCODE_KP_COLON as i32 | SCANCODE_MASK);
    pub const KP_HASH: Keycode = Keycode(api::SDL_SCANCODE_KP_HASH as i32 | SCANCODE_MASK);
    pub const KP_SPACE: Keycode = Keycode(api::SDL_SCANCODE_KP_SPACE as i32 | SCANCODE_MASK);
    pub const KP_AT: Keycode = Keycode(api::SDL_SCANCODE_KP_AT as i32 | SCANCODE_MASK);
    pub const KP_EXCLAM: Keycode = Keycode(api::SDL_SCANCODE_KP_EXCLAM as i32 | SCANCODE_MASK);
    pub const KP_MEMSTORE: Keycode = Keycode(api::SDL_SCANCODE_KP_MEMSTORE as i32 | SCANCODE_MASK);
    pub const KP_MEMRECALL: Keycode =
        Keycode(api::SDL_SCANCODE_KP_MEMRECALL as i32 | SCANCODE_MASK);
    pub const KP_MEMCLEAR: Keycode = Keycode(api::SDL_SCANCODE_KP_MEMCLEAR as i32 | SCANCODE_MASK);
    pub const KP_MEMADD: Keycode = Keycode(api::SDL_SCANCODE_KP_MEMADD as i32 | SCANCODE_MASK);
    pub const KP_MEMSUBTRACT: Keycode =
        Keycode(api::SDL_SCANCODE_KP_MEMSUBTRACT as i32 | SCANCODE_MASK);
    pub const KP_MEMMULTIPLY: Keycode =
        Keycode(api::SDL_SCANCODE_KP_MEMMULTIPLY as i32 | SCANCODE_MASK);
    pub const KP_MEMDIVIDE: Keycode =
        Keycode(api::SDL_SCANCODE_KP_MEMDIVIDE as i32 | SCANCODE_MASK);
    pub const KP_PLUSMINUS: Keycode =
        Keycode(api::SDL_SCANCODE_KP_PLUSMINUS as i32 | SCANCODE_MASK);
    pub const KP_CLEAR: Keycode = Keycode(api::SDL_SCANCODE_KP_CLEAR as i32 | SCANCODE_MASK);
    pub const KP_CLEARENTRY: Keycode =
        Keycode(api::SDL_SCANCODE_KP_CLEARENTRY as i32 | SCANCODE_MASK);
    pub const KP_BINARY: Keycode = Keycode(api::SDL_SCANCODE_KP_BINARY as i32 | SCANCODE_MASK);
    pub const KP_OCTAL: Keycode = Keycode(api::SDL_SCANCODE_KP_OCTAL as i32 | SCANCODE_MASK);
    pub const KP_DECIMAL: Keycode = Keycode(api::SDL_SCANCODE_KP_DECIMAL as i32 | SCANCODE_MASK);
    pub const KP_HEXADECIMAL: Keycode =
        Keycode(api::SDL_SCANCODE_KP_HEXADECIMAL as i32 | SCANCODE_MASK);
    pub const LCTRL: Keycode = Keycode(api::SDL_SCANCODE_LCTRL as i32 | SCANCODE_MASK);
    pub const LSHIFT: Keycode = Keycode(api::SDL_SCANCODE_LSHIFT as i32 | SCANCODE_MASK);
    pub const LALT: Keycode = Keycode(api::SDL_SCANCODE_LALT as i32 | SCANCODE_MASK);
    pub const LGUI: Keycode = Keycode(api::SDL_SCANCODE_LGUI as i32 | SCANCODE_MASK);
    pub const RCTRL: Keycode = Keycode(api::SDL_SCANCODE_RCTRL as i32 | SCANCODE_MASK);
    pub const RSHIFT: Keycode = Keycode(api::SDL_SCANCODE_RSHIFT as i32 | SCANCODE_MASK);
    pub const RALT: Keycode = Keycode(api::SDL_SCANCODE_RALT as i32 | SCANCODE_MASK);
    pub const RGUI: Keycode = Keycode(api::SDL_SCANCODE_RGUI as i32 | SCANCODE_MASK);
    pub const MODE: Keycode = Keycode(api::SDL_SCANCODE_MODE as i32 | SCANCODE_MASK);
    pub const AUDIONEXT: Keycode = Keycode(api::SDL_SCANCODE_AUDIONEXT as i32 | SCANCODE_MASK);
    pub const AUDIOPREV: Keycode = Keycode(api::SDL_SCANCODE_AUDIOPREV as i32 | SCANCODE_MASK);
    pub const AUDIOSTOP: Keycode = Keycode(api::SDL_SCANCODE_AUDIOSTOP as i32 | SCANCODE_MASK);
    pub const AUDIOPLAY: Keycode = Keycode(api::SDL_SCANCODE_AUDIOPLAY as i32 | SCANCODE_MASK);
    pub const AUDIOMUTE: Keycode = Keycode(api::SDL_SCANCODE_AUDIOMUTE as i32 | SCANCODE_MASK);
    pub const MEDIASELECT: Keycode = Keycode(api::SDL_SCANCODE_MEDIASELECT as i32 | SCANCODE_MASK);
    pub const WWW: Keycode = Keycode(api::SDL_SCANCODE_WWW as i32 | SCANCODE_MASK);
    pub const MAIL: Keycode = Keycode(api::SDL_SCANCODE_MAIL as i32 | SCANCODE_MASK);
    pub const CALCULATOR: Keycode = Keycode(api::SDL_SCANCODE_CALCULATOR as i32 | SCANCODE_MASK);
    pub const COMPUTER: Keycode = Keycode(api::SDL_SCANCODE_COMPUTER as i32 | SCANCODE_MASK);
    pub const AC_SEARCH: Keycode = Keycode(api::SDL_SCANCODE_AC_SEARCH as i32 | SCANCODE_MASK);
    pub const AC_HOME: Keycode = Keycode(api::SDL_SCANCODE_AC_HOME as i32 | SCANCODE_MASK);
    pub const AC_BACK: Keycode = Keycode(api::SDL_SCANCODE_AC_BACK as i32 | SCANCODE_MASK);
    pub const AC_FORWARD: Keycode = Keycode(api::SDL_SCANCODE_AC_FORWARD as i32 | SCANCODE_MASK);
    pub const AC_STOP: Keycode = Keycode(api::SDL_SCANCODE_AC_STOP as i32 | SCANCODE_MASK);
    pub const AC_REFRESH: Keycode = Keycode(api::SDL_SCANCODE_AC_REFRESH as i32 | SCANCODE_MASK);
    pub const AC_BOOKMARKS: Keycode =
        Keycode(api::SDL_SCANCODE_AC_BOOKMARKS as i32 | SCANCODE_MASK);
    pub const BRIGHTNESSDOWN: Keycode =
        Keycode(api::SDL_SCANCODE_BRIGHTNESSDOWN as i32 | SCANCODE_MASK);
    pub const BRIGHTNESSUP: Keycode =
        Keycode(api::SDL_SCANCODE_BRIGHTNESSUP as i32 | SCANCODE_MASK);
    pub const DISPLAYSWITCH: Keycode =
        Keycode(api::SDL_SCANCODE_DISPLAYSWITCH as i32 | SCANCODE_MASK);
    pub const KBDILLUMTOGGLE: Keycode =
        Keycode(api::SDL_SCANCODE_KBDILLUMTOGGLE as i32 | SCANCODE_MASK);
    pub const KBDILLUMDOWN: Keycode =
        Keycode(api::SDL_SCANCODE_KBDILLUMDOWN as i32 | SCANCODE_MASK);
    pub const KBDILLUMUP: Keycode = Keycode(api::SDL_SCANCODE_KBDILLUMUP as i32 | SCANCODE_MASK);
    pub const EJECT: Keycode = Keycode(api::SDL_SCANCODE_EJECT as i32 | SCANCODE_MASK);
    pub const SLEEP: Keycode = Keycode(api::SDL_SCANCODE_SLEEP as i32 | SCANCODE_MASK);
    pub const APP1: Keycode = Keycode(api::SDL_SCANCODE_APP1 as i32 | SCANCODE_MASK);
    pub const APP2: Keycode = Keycode(api::SDL_SCANCODE_APP2 as i32 | SCANCODE_MASK);
    pub const AUDIOREWIND: Keycode = Keycode(api::SDL_SCANCODE_AUDIOREWIND as i32 | SCANCODE_MASK);
    pub const AUDIOFASTFORWARD: Keycode =
        Keycode(api::SDL_SCANCODE_AUDIOFASTFORWARD as i32 | SCANCODE_MASK);
}

impl fmt::Debug for Keycode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        const MATCHES: &'static [(Keycode, &'static str)] = &[
            (Keycode::UNKNOWN, "UNKNOWN"),
            (Keycode::RETURN, "RETURN"),
            (Keycode::ESCAPE, "ESCAPE"),
            (Keycode::BACKSPACE, "BACKSPACE"),
            (Keycode::TAB, "TAB"),
            (Keycode::SPACE, "SPACE"),
            (Keycode::EXCLAIM, "EXCLAIM"),
            (Keycode::QUOTEDBL, "QUOTEDBL"),
            (Keycode::HASH, "HASH"),
            (Keycode::PERCENT, "PERCENT"),
            (Keycode::DOLLAR, "DOLLAR"),
            (Keycode::AMPERSAND, "AMPERSAND"),
            (Keycode::QUOTE, "QUOTE"),
            (Keycode::LEFTPAREN, "LEFTPAREN"),
            (Keycode::RIGHTPAREN, "RIGHTPAREN"),
            (Keycode::ASTERISK, "ASTERISK"),
            (Keycode::PLUS, "PLUS"),
            (Keycode::COMMA, "COMMA"),
            (Keycode::MINUS, "MINUS"),
            (Keycode::PERIOD, "PERIOD"),
            (Keycode::SLASH, "SLASH"),
            (Keycode::NUM_0, "NUM_0"),
            (Keycode::NUM_1, "NUM_1"),
            (Keycode::NUM_2, "NUM_2"),
            (Keycode::NUM_3, "NUM_3"),
            (Keycode::NUM_4, "NUM_4"),
            (Keycode::NUM_5, "NUM_5"),
            (Keycode::NUM_6, "NUM_6"),
            (Keycode::NUM_7, "NUM_7"),
            (Keycode::NUM_8, "NUM_8"),
            (Keycode::NUM_9, "NUM_9"),
            (Keycode::COLON, "COLON"),
            (Keycode::SEMICOLON, "SEMICOLON"),
            (Keycode::LESS, "LESS"),
            (Keycode::EQUALS, "EQUALS"),
            (Keycode::GREATER, "GREATER"),
            (Keycode::QUESTION, "QUESTION"),
            (Keycode::AT, "AT"),
            (Keycode::LEFTBRACKET, "LEFTBRACKET"),
            (Keycode::BACKSLASH, "BACKSLASH"),
            (Keycode::RIGHTBRACKET, "RIGHTBRACKET"),
            (Keycode::CARET, "CARET"),
            (Keycode::UNDERSCORE, "UNDERSCORE"),
            (Keycode::BACKQUOTE, "BACKQUOTE"),
            (Keycode::A, "A"),
            (Keycode::B, "B"),
            (Keycode::C, "C"),
            (Keycode::D, "D"),
            (Keycode::E, "E"),
            (Keycode::F, "F"),
            (Keycode::G, "G"),
            (Keycode::H, "H"),
            (Keycode::I, "I"),
            (Keycode::J, "J"),
            (Keycode::K, "K"),
            (Keycode::L, "L"),
            (Keycode::M, "M"),
            (Keycode::N, "N"),
            (Keycode::O, "O"),
            (Keycode::P, "P"),
            (Keycode::Q, "Q"),
            (Keycode::R, "R"),
            (Keycode::S, "S"),
            (Keycode::T, "T"),
            (Keycode::U, "U"),
            (Keycode::V, "V"),
            (Keycode::W, "W"),
            (Keycode::X, "X"),
            (Keycode::Y, "Y"),
            (Keycode::Z, "Z"),
            (Keycode::CAPSLOCK, "CAPSLOCK"),
            (Keycode::F1, "F1"),
            (Keycode::F2, "F2"),
            (Keycode::F3, "F3"),
            (Keycode::F4, "F4"),
            (Keycode::F5, "F5"),
            (Keycode::F6, "F6"),
            (Keycode::F7, "F7"),
            (Keycode::F8, "F8"),
            (Keycode::F9, "F9"),
            (Keycode::F10, "F10"),
            (Keycode::F11, "F11"),
            (Keycode::F12, "F12"),
            (Keycode::PRINTSCREEN, "PRINTSCREEN"),
            (Keycode::SCROLLLOCK, "SCROLLLOCK"),
            (Keycode::PAUSE, "PAUSE"),
            (Keycode::INSERT, "INSERT"),
            (Keycode::HOME, "HOME"),
            (Keycode::PAGEUP, "PAGEUP"),
            (Keycode::DELETE, "DELETE"),
            (Keycode::END, "END"),
            (Keycode::PAGEDOWN, "PAGEDOWN"),
            (Keycode::RIGHT, "RIGHT"),
            (Keycode::LEFT, "LEFT"),
            (Keycode::DOWN, "DOWN"),
            (Keycode::UP, "UP"),
            (Keycode::NUMLOCKCLEAR, "NUMLOCKCLEAR"),
            (Keycode::KP_DIVIDE, "KP_DIVIDE"),
            (Keycode::KP_MULTIPLY, "KP_MULTIPLY"),
            (Keycode::KP_MINUS, "KP_MINUS"),
            (Keycode::KP_PLUS, "KP_PLUS"),
            (Keycode::KP_ENTER, "KP_ENTER"),
            (Keycode::KP_1, "KP_1"),
            (Keycode::KP_2, "KP_2"),
            (Keycode::KP_3, "KP_3"),
            (Keycode::KP_4, "KP_4"),
            (Keycode::KP_5, "KP_5"),
            (Keycode::KP_6, "KP_6"),
            (Keycode::KP_7, "KP_7"),
            (Keycode::KP_8, "KP_8"),
            (Keycode::KP_9, "KP_9"),
            (Keycode::KP_0, "KP_0"),
            (Keycode::KP_PERIOD, "KP_PERIOD"),
            (Keycode::APPLICATION, "APPLICATION"),
            (Keycode::POWER, "POWER"),
            (Keycode::KP_EQUALS, "KP_EQUALS"),
            (Keycode::F13, "F13"),
            (Keycode::F14, "F14"),
            (Keycode::F15, "F15"),
            (Keycode::F16, "F16"),
            (Keycode::F17, "F17"),
            (Keycode::F18, "F18"),
            (Keycode::F19, "F19"),
            (Keycode::F20, "F20"),
            (Keycode::F21, "F21"),
            (Keycode::F22, "F22"),
            (Keycode::F23, "F23"),
            (Keycode::F24, "F24"),
            (Keycode::EXECUTE, "EXECUTE"),
            (Keycode::HELP, "HELP"),
            (Keycode::MENU, "MENU"),
            (Keycode::SELECT, "SELECT"),
            (Keycode::STOP, "STOP"),
            (Keycode::AGAIN, "AGAIN"),
            (Keycode::UNDO, "UNDO"),
            (Keycode::CUT, "CUT"),
            (Keycode::COPY, "COPY"),
            (Keycode::PASTE, "PASTE"),
            (Keycode::FIND, "FIND"),
            (Keycode::MUTE, "MUTE"),
            (Keycode::VOLUMEUP, "VOLUMEUP"),
            (Keycode::VOLUMEDOWN, "VOLUMEDOWN"),
            (Keycode::KP_COMMA, "KP_COMMA"),
            (Keycode::KP_EQUALSAS400, "KP_EQUALSAS400"),
            (Keycode::ALTERASE, "ALTERASE"),
            (Keycode::SYSREQ, "SYSREQ"),
            (Keycode::CANCEL, "CANCEL"),
            (Keycode::CLEAR, "CLEAR"),
            (Keycode::PRIOR, "PRIOR"),
            (Keycode::RETURN2, "RETURN2"),
            (Keycode::SEPARATOR, "SEPARATOR"),
            (Keycode::OUT, "OUT"),
            (Keycode::OPER, "OPER"),
            (Keycode::CLEARAGAIN, "CLEARAGAIN"),
            (Keycode::CRSEL, "CRSEL"),
            (Keycode::EXSEL, "EXSEL"),
            (Keycode::KP_00, "KP_00"),
            (Keycode::KP_000, "KP_000"),
            (Keycode::THOUSANDSSEPARATOR, "THOUSANDSSEPARATOR"),
            (Keycode::DECIMALSEPARATOR, "DECIMALSEPARATOR"),
            (Keycode::CURRENCYUNIT, "CURRENCYUNIT"),
            (Keycode::CURRENCYSUBUNIT, "CURRENCYSUBUNIT"),
            (Keycode::KP_LEFTPAREN, "KP_LEFTPAREN"),
            (Keycode::KP_RIGHTPAREN, "KP_RIGHTPAREN"),
            (Keycode::KP_LEFTBRACE, "KP_LEFTBRACE"),
            (Keycode::KP_RIGHTBRACE, "KP_RIGHTBRACE"),
            (Keycode::KP_TAB, "KP_TAB"),
            (Keycode::KP_BACKSPACE, "KP_BACKSPACE"),
            (Keycode::KP_A, "KP_A"),
            (Keycode::KP_B, "KP_B"),
            (Keycode::KP_C, "KP_C"),
            (Keycode::KP_D, "KP_D"),
            (Keycode::KP_E, "KP_E"),
            (Keycode::KP_F, "KP_F"),
            (Keycode::KP_XOR, "KP_XOR"),
            (Keycode::KP_POWER, "KP_POWER"),
            (Keycode::KP_PERCENT, "KP_PERCENT"),
            (Keycode::KP_LESS, "KP_LESS"),
            (Keycode::KP_GREATER, "KP_GREATER"),
            (Keycode::KP_AMPERSAND, "KP_AMPERSAND"),
            (Keycode::KP_DBLAMPERSAND, "KP_DBLAMPERSAND"),
            (Keycode::KP_VERTICALBAR, "KP_VERTICALBAR"),
            (Keycode::KP_DBLVERTICALBAR, "KP_DBLVERTICALBAR"),
            (Keycode::KP_COLON, "KP_COLON"),
            (Keycode::KP_HASH, "KP_HASH"),
            (Keycode::KP_SPACE, "KP_SPACE"),
            (Keycode::KP_AT, "KP_AT"),
            (Keycode::KP_EXCLAM, "KP_EXCLAM"),
            (Keycode::KP_MEMSTORE, "KP_MEMSTORE"),
            (Keycode::KP_MEMRECALL, "KP_MEMRECALL"),
            (Keycode::KP_MEMCLEAR, "KP_MEMCLEAR"),
            (Keycode::KP_MEMADD, "KP_MEMADD"),
            (Keycode::KP_MEMSUBTRACT, "KP_MEMSUBTRACT"),
            (Keycode::KP_MEMMULTIPLY, "KP_MEMMULTIPLY"),
            (Keycode::KP_MEMDIVIDE, "KP_MEMDIVIDE"),
            (Keycode::KP_PLUSMINUS, "KP_PLUSMINUS"),
            (Keycode::KP_CLEAR, "KP_CLEAR"),
            (Keycode::KP_CLEARENTRY, "KP_CLEARENTRY"),
            (Keycode::KP_BINARY, "KP_BINARY"),
            (Keycode::KP_OCTAL, "KP_OCTAL"),
            (Keycode::KP_DECIMAL, "KP_DECIMAL"),
            (Keycode::KP_HEXADECIMAL, "KP_HEXADECIMAL"),
            (Keycode::LCTRL, "LCTRL"),
            (Keycode::LSHIFT, "LSHIFT"),
            (Keycode::LALT, "LALT"),
            (Keycode::LGUI, "LGUI"),
            (Keycode::RCTRL, "RCTRL"),
            (Keycode::RSHIFT, "RSHIFT"),
            (Keycode::RALT, "RALT"),
            (Keycode::RGUI, "RGUI"),
            (Keycode::MODE, "MODE"),
            (Keycode::AUDIONEXT, "AUDIONEXT"),
            (Keycode::AUDIOPREV, "AUDIOPREV"),
            (Keycode::AUDIOSTOP, "AUDIOSTOP"),
            (Keycode::AUDIOPLAY, "AUDIOPLAY"),
            (Keycode::AUDIOMUTE, "AUDIOMUTE"),
            (Keycode::MEDIASELECT, "MEDIASELECT"),
            (Keycode::WWW, "WWW"),
            (Keycode::MAIL, "MAIL"),
            (Keycode::CALCULATOR, "CALCULATOR"),
            (Keycode::COMPUTER, "COMPUTER"),
            (Keycode::AC_SEARCH, "AC_SEARCH"),
            (Keycode::AC_HOME, "AC_HOME"),
            (Keycode::AC_BACK, "AC_BACK"),
            (Keycode::AC_FORWARD, "AC_FORWARD"),
            (Keycode::AC_STOP, "AC_STOP"),
            (Keycode::AC_REFRESH, "AC_REFRESH"),
            (Keycode::AC_BOOKMARKS, "AC_BOOKMARKS"),
            (Keycode::BRIGHTNESSDOWN, "BRIGHTNESSDOWN"),
            (Keycode::BRIGHTNESSUP, "BRIGHTNESSUP"),
            (Keycode::DISPLAYSWITCH, "DISPLAYSWITCH"),
            (Keycode::KBDILLUMTOGGLE, "KBDILLUMTOGGLE"),
            (Keycode::KBDILLUMDOWN, "KBDILLUMDOWN"),
            (Keycode::KBDILLUMUP, "KBDILLUMUP"),
            (Keycode::EJECT, "EJECT"),
            (Keycode::SLEEP, "SLEEP"),
            (Keycode::APP1, "APP1"),
            (Keycode::APP2, "APP2"),
            (Keycode::AUDIOREWIND, "AUDIOREWIND"),
            (Keycode::AUDIOFASTFORWARD, "AUDIOFASTFORWARD"),
        ];
        if let Some(&(_, name)) = MATCHES.iter().find(|&&(value, _)| value == *self) {
            write!(f, "Keycode::{}", name)
        } else {
            f.debug_tuple("Keycode").field(&self.0).finish()
        }
    }
}
