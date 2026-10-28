use crate::storage::LocalStorageStore;
use crate::{AsyncState, Files};
use common::files::{Overlay, OverlayFile};
use common::{Platform, TPalette};
use std::borrow::Cow;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::future::Future;
use std::io::Cursor;
use std::pin::Pin;
use std::task::{Context, Poll};

pub struct WebPlatform {
    files: Overlay<LocalStorageStore, Files>,
    palette: RefCell<TPalette>,
    screen: RefCell<Vec<u32>>,
    ch: Cell<u8>,
    ch2: Cell<u8>,

    pub async_state: Cell<AsyncState>,
    keypresses: RefCell<Vec<(String, (u8, u8))>>,
    // The character of each key code without Shift in the current layout
    unshifted_keys: RefCell<HashMap<String, char>>,
    // A KeyboardEvent does not tell Left Alt from Right Alt, so follow the Right Alt key events
    right_alt: Cell<bool>,
    close_requested: Cell<bool>,
}

#[derive(Clone, Copy)]
pub struct Modifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub alt_graph: bool,
    pub meta: bool,
    pub caps_lock: bool,
}

impl WebPlatform {
    pub fn new(data: LocalStorageStore, files: Files) -> Self {
        Self {
            files: Overlay::new(data, files),
            palette: RefCell::new([[0; 3]; 256]),
            screen: RefCell::new(vec![0; 320 * 200]),
            ch: Cell::new(0),
            ch2: Cell::new(0),

            async_state: Cell::new(AsyncState::None),
            keypresses: RefCell::new(vec![]),
            unshifted_keys: RefCell::new(HashMap::new()),
            right_alt: Cell::new(false),
            close_requested: Cell::new(false),
        }
    }

    pub fn request_close(&self) {
        self.close_requested.set(true);
    }

    pub fn reset(&self) {
        self.close_requested.set(false);
        self.keypresses.borrow_mut().clear();
        self.ch.set(0);
        self.ch2.set(0);
    }

    pub fn pixels(&self) -> *const u32 {
        self.screen.borrow().as_ptr()
    }

    pub fn keydown(&self, key: &str, code: &str, modifiers: Modifiers) -> bool {
        // The keyup is lost if the page loses focus while Right Alt is down
        if !modifiers.alt {
            self.right_alt.set(false);
        }
        if code == "AltRight" {
            self.right_alt.set(true);
        }
        if let Some(c) = unshifted_key(key, code, modifiers) {
            self.unshifted_keys.borrow_mut().insert(code.to_string(), c);
        }
        let unshifted = self.unshifted_keys.borrow().get(code).copied();
        let keypress = Self::translate_key(key, code, unshifted, self.right_alt.get(), modifiers);
        self.keypresses
            .borrow_mut()
            .push((key.to_string(), keypress.unwrap_or((0, 0))));
        keypress.is_some()
    }

    pub fn keyup(&self, code: &str) {
        if code == "AltRight" {
            self.right_alt.set(false);
        }
    }

    pub fn set_unshifted_key(&self, code: &str, key: &str) {
        if let Some(c) = single_char(key) {
            self.unshifted_keys.borrow_mut().insert(code.to_string(), c);
        }
    }

    fn is_key_pressed(&self) -> bool {
        let mut keypresses = self.keypresses.borrow_mut();
        while keypresses.len() > 0 {
            let keypress = keypresses.remove(0);
            match keypress.0.as_str() {
                // Ignore modifier and status keys, since they did not trigger a keypress in DOS version.
                "Control" | "Shift" | "Alt" | "AltGraph" | "Meta" | "OS" | "CapsLock"
                | "ScrollLock" | "NumLock" | "PrintScreen" | "Pause" => {}
                _ => {
                    keypresses.insert(0, keypress);
                    return true;
                }
            }
        }
        false
    }

    fn process_keypress(&self) -> (u8, u8) {
        let mut keypresses = self.keypresses.borrow_mut();
        let (_, (ch, ch2)) = keypresses.remove(0);
        self.ch.set(ch);
        self.ch2.set(ch2);
        (ch, ch2)
    }

    /// Translates a key in the same way as WaitForKeyPress of the SDL port. `unshifted` is the
    /// character of the key without Shift in the current layout, if known.
    ///
    /// Returns None for a key that gives neither a character nor a scan code.
    fn translate_key(
        key: &str,
        code: &str,
        unshifted: Option<char>,
        right_alt: bool,
        modifiers: Modifiers,
    ) -> Option<(u8, u8)> {
        let keycode =
            single_char(key).and_then(|c| sdl_keycode(c, code, unshifted, modifiers.shift));

        // Special cases for key combinations used throughout the game.
        // Check for Right Alt/AltGr first, since pressing it can make Ctrl and Alt look as pressed too.
        if modifiers.alt_graph {
            return None;
        }
        if modifiers.alt && right_alt {
            return None;
        }
        if modifiers.alt {
            // ALT-X
            return (keycode == Some('x')).then_some((0, 45));
        }
        if modifiers.ctrl {
            // CTRL-C
            return (keycode == Some('c')).then_some((3, 0));
        }
        if modifiers.meta {
            return None;
        }

        let Some(c) = single_char(key) else {
            // The SDL port ignores the keys that are not in its Shift table
            if modifiers.shift {
                return None;
            }
            // With Num Lock off, the key value of keypad 5 is "Clear", and the other keypad keys
            // have the key values of the navigation keys.
            if code == "Numpad5" {
                return Some((0, 76));
            }
            return named_key(key);
        };
        if code.starts_with("Numpad") {
            // The keypad keys are not in the Shift table of the SDL port
            if modifiers.shift {
                return None;
            }
            // In some locales the keypad decimal key gives a comma, but the SDL port gives a period
            return character(if code == "NumpadDecimal" { '.' } else { c });
        }

        let mut ch = keycode?;
        // Handle conversion of letters to uppercase.
        // Convert only if Caps Lock is off and Shift is pressed or vice versa.
        if modifiers.shift ^ modifiers.caps_lock {
            if let 'a'..='z' | 'à'..='ö' | 'ø'..='þ' = ch {
                ch = char::from_u32(ch as u32 - 32).unwrap();
            }
        }
        // If modifier is Shift, convert the key pressed accordingly.
        // For special keys, standard US QWERTY layout is assumed.
        if modifiers.shift
            && !matches!(ch, 'A'..='Z' | 'a'..='z' | '¾'..='Ö' | 'Ø'..='Þ' | 'à'..='ö' | 'ø'..='þ')
        {
            ch = us_qwerty_shift(ch)?;
        }
        character(ch)
    }
}

struct WebPortFuture<'a> {
    p: &'a WebPlatform,
}

impl<'a> Future for WebPortFuture<'a> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let AsyncState::None = self.p.async_state.get() {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

impl Platform for WebPlatform {
    fn set_palette(&self, input: &TPalette) {
        let mut palette = self.palette.borrow_mut();
        palette.copy_from_slice(input);
    }

    fn render_phase1(&self, buffer: &[u8]) {
        let palette = self.palette.borrow();
        let mut screen = self.screen.borrow_mut();
        for (i, pixel) in buffer.iter().enumerate() {
            let [r, g, b] = palette[*pixel as usize];
            screen[i] =
                (255u32 << 24) | ((b as u32) * 4 << 16) | ((g as u32) * 4 << 8) | (r as u32) * 4;
        }
    }

    async fn render_phase2(&self) {
        self.async_state.set(AsyncState::Render);
        WebPortFuture { p: self }.await
    }

    async fn key_pressed(&self) -> bool {
        self.async_state.set(AsyncState::KeyPressed);
        WebPortFuture { p: self }.await;
        self.is_key_pressed()
    }

    async fn wait_for_key_press(&self) -> (u8, u8) {
        // The close must save the files in one poll, so it cannot wait for a key
        if self.close_requested.get() {
            return (self.ch.get(), self.ch2.get());
        }
        if self.keypresses.borrow().is_empty() {
            self.async_state.set(AsyncState::WaitForKeyPress);
            WebPortFuture { p: self }.await;
        }
        self.process_keypress()
    }

    fn putsaa(&self) {
        while self.is_key_pressed() {
            self.process_keypress();
        }
    }

    fn clearchs(&self) {
        self.ch.set(1);
        self.ch2.set(1);
    }

    fn get_ch(&self) -> u8 {
        self.ch.get()
    }

    fn get_ch2(&self) -> u8 {
        self.ch2.get()
    }

    fn set_ch(&self, ch: u8) {
        self.ch.set(ch);
    }

    fn set_ch2(&self, ch: u8) {
        self.ch2.set(ch);
    }

    type WritableFile<'a> = OverlayFile<'a, LocalStorageStore, Files>;

    fn create_file<'a, P: AsRef<str>>(&'a self, path: P) -> Self::WritableFile<'a> {
        self.files.create(path.as_ref())
    }

    type ReadableFile<'a> = Cursor<Cow<'a, [u8]>>;

    fn open_file<'a, P: AsRef<str>>(&'a self, path: P) -> Self::ReadableFile<'a> {
        let path = path.as_ref();
        Cursor::new(
            self.files
                .read(path)
                .unwrap_or_else(|| panic!("Unknown file {}", path)),
        )
    }

    fn file_exists<P: AsRef<str>>(&self, path: P) -> bool {
        self.files.exists(path.as_ref())
    }

    fn remove_file<P: AsRef<str>>(&self, path: P) {
        self.files.remove(path.as_ref());
    }

    fn find_files(&self, pattern: &str) -> Vec<String> {
        self.files.find(pattern)
    }

    fn get_date(&self) -> (u16, u16, u16, u16) {
        let now = js_sys::Date::new_0();
        (
            now.get_full_year() as u16,
            now.get_month() as u16 + 1,
            now.get_date() as u16,
            now.get_day() as u16,
        )
    }

    fn get_time(&self) -> (u16, u16, u16, u16) {
        let now = js_sys::Date::new_0();
        (
            now.get_hours() as u16,
            now.get_minutes() as u16,
            now.get_seconds() as u16,
            (now.get_milliseconds() / 10) as u16,
        )
    }

    fn param_str(&self, _index: usize) -> String {
        String::new()
    }

    fn wait(&self, ms: u32) {
        let end = js_sys::Date::now() + ms as f64;
        while js_sys::Date::now() < end {}
    }

    fn close_requested(&self) -> bool {
        self.close_requested.get()
    }
}

/// Returns the key value if it is the character of the key without Shift.
fn unshifted_key(key: &str, code: &str, modifiers: Modifiers) -> Option<char> {
    let c = single_char(key)?;
    let modified = modifiers.shift
        || modifiers.ctrl
        || modifiers.alt
        || modifiers.alt_graph
        || modifiers.meta
        || modifiers.caps_lock;
    (!modified && !code.starts_with("Numpad") && !is_composed(c, code)).then_some(c)
}

/// Returns the key as SDL gives it: the character of the key without Shift in the current layout,
/// lowercase for a letter.
fn sdl_keycode(c: char, code: &str, unshifted: Option<char>, shift: bool) -> Option<char> {
    // SDL gives the number row as digits in every layout
    if let Some(digit) = code.strip_prefix("Digit").and_then(single_char) {
        return Some(digit);
    }
    if unshifted.is_some() {
        return unshifted;
    }
    // A composed character does not show the key, so use the key position on a US QWERTY keyboard
    if is_composed(c, code) {
        return code
            .strip_prefix("Key")
            .and_then(single_char)
            .map(|letter| letter.to_ascii_lowercase());
    }
    match c {
        'A'..='Z' | 'À'..='Ö' | 'Ø'..='Þ' => char::from_u32(c as u32 + 32),
        'a'..='z' | 'à'..='ö' | 'ø'..='þ' => Some(c),
        _ if !shift => Some(c),
        // The key value has the Shift of the layout, so use the key position on a US QWERTY keyboard
        _ => us_qwerty_key(code),
    }
}

/// After a dead key, a letter key gives a composed character such as "é". On macOS, Option also
/// changes the character of a letter key.
fn is_composed(c: char, code: &str) -> bool {
    code.starts_with("Key") && !c.is_ascii_alphabetic()
}

fn us_qwerty_key(code: &str) -> Option<char> {
    let c = match code {
        "Minus" => '-',
        "Equal" => '=',
        "BracketLeft" => '[',
        "BracketRight" => ']',
        "Semicolon" => ';',
        "Quote" => '\'',
        "Backquote" => '`',
        "Backslash" => '\\',
        "Comma" => ',',
        "Period" => '.',
        "Slash" => '/',
        _ => return None,
    };
    Some(c)
}

fn named_key(key: &str) -> Option<(u8, u8)> {
    let keypress = match key {
        "Enter" => (13, 0),
        "Escape" => (27, 0),
        "Backspace" => (8, 0),
        "Tab" => (9, 0),

        "F1" => (0, 59),
        "F2" => (0, 60),
        "F3" => (0, 61),
        "F4" => (0, 62),
        "F5" => (0, 63),
        "F6" => (0, 64),
        "F7" => (0, 65),
        "F8" => (0, 66),
        "F9" => (0, 67),
        "F10" => (0, 68),

        "ArrowLeft" => (0, 75),
        "ArrowRight" => (0, 77),
        "ArrowUp" => (0, 72),
        "ArrowDown" => (0, 80),

        "Insert" => (0, 82),
        "Delete" => (0, 83),
        "Home" => (0, 71),
        "End" => (0, 79),
        "PageUp" => (0, 73),
        "PageDown" => (0, 81),
        _ => return None,
    };
    Some(keypress)
}

fn us_qwerty_shift(c: char) -> Option<char> {
    let shifted = match c {
        '1' => '!',
        '2' => '@',
        '3' => '#',
        '4' => '$',
        '5' => '%',
        '6' => '^',
        '7' => '&',
        '8' => '*',
        '9' => '(',
        '0' => ')',
        '-' => '_',
        '=' => '+',
        '[' => '{',
        ']' => '}',
        ';' => ':',
        '\'' => '"',
        '`' => '~',
        '\\' => '|',
        ',' => '<',
        '.' => '>',
        '/' => '?',
        _ => return None,
    };
    Some(shifted)
}

fn character(ch: char) -> Option<(u8, u8)> {
    let ch1 = match ch as u32 {
        32..=126 => ch as u8,

        // Special characters supported by the game, OEM 865 nordic encoding is used for ch1 values
        196 => return Some((142, 0)), // A with diaeresis
        197 => return Some((143, 0)), // A with ring above
        198 => return Some((146, 0)), // AE
        214 => return Some((153, 0)), // O with diaeresis
        216 => return Some((157, 0)), // O with stroke
        220 => return Some((154, 0)), // U with diaeresis
        223 => return Some((225, 0)), // sharp s

        228 => return Some((132, 36)), // a with diaeresis
        229 => return Some((134, 26)), // a with ring above
        230 => return Some((145, 0)),  // ae
        246 => return Some((148, 39)), // o with diaeresis
        248 => return Some((158, 0)),  // o with stroke
        252 => return Some((129, 0)),  // u with diaeresis
        _ => return None,
    };
    let ch2 = match ch1 {
        b'a' => 30,
        b'b' => 48,
        b'c' => 46,
        b'd' => 32,
        b'e' => 18,
        b'f' => 33,
        b'g' => 34,
        b'h' => 35,
        b'i' => 23,
        b'j' => 36,
        b'k' => 37,
        b'l' => 38,
        b'm' => 50,
        b'n' => 49,
        b'o' => 24,
        b'p' => 25,
        b'q' => 16,
        b'r' => 19,
        b's' => 31,
        b't' => 20,
        b'u' => 22,
        b'v' => 47,
        b'w' => 17,
        b'x' => 45,
        b'y' => 21,
        b'z' => 44,
        _ => 0,
    };
    Some((ch1, ch2))
}

fn single_char(key: &str) -> Option<char> {
    let mut chars = key.chars();
    match (chars.next(), chars.next()) {
        (Some(c), None) => Some(c),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{Modifiers, WebPlatform};
    use crate::storage::LocalStorageStore;
    use crate::Files;

    const NONE: Modifiers = Modifiers {
        shift: false,
        ctrl: false,
        alt: false,
        alt_graph: false,
        meta: false,
        caps_lock: false,
    };
    const SHIFT: Modifiers = Modifiers {
        shift: true,
        ..NONE
    };
    const CAPS: Modifiers = Modifiers {
        caps_lock: true,
        ..NONE
    };
    const SHIFT_CAPS: Modifiers = Modifiers {
        shift: true,
        caps_lock: true,
        ..NONE
    };
    const CTRL: Modifiers = Modifiers { ctrl: true, ..NONE };
    const CTRL_SHIFT: Modifiers = Modifiers {
        ctrl: true,
        shift: true,
        ..NONE
    };
    const ALT: Modifiers = Modifiers { alt: true, ..NONE };
    const ALT_SHIFT: Modifiers = Modifiers {
        alt: true,
        shift: true,
        ..NONE
    };
    const ALT_GRAPH: Modifiers = Modifiers {
        alt_graph: true,
        ..NONE
    };
    // Windows gives AltGr also as Ctrl and Alt
    const CTRL_ALT_GRAPH: Modifiers = Modifiers {
        ctrl: true,
        alt: true,
        alt_graph: true,
        ..NONE
    };
    const META: Modifiers = Modifiers { meta: true, ..NONE };

    // The expected values come from the Pascal WaitForKeyPress, for the SDL key event of the same
    // key with a US layout, or with a Nordic layout for the Nordic keys.
    #[test]
    fn translate_key() {
        let table = [
            // AltGr blocks every key, also Ctrl+C
            ("c", "KeyC", ALT_GRAPH, None),
            ("c", "KeyC", CTRL_ALT_GRAPH, None),
            // Alt gives only Alt+X
            ("x", "KeyX", ALT, Some((0, 45))),
            ("X", "KeyX", ALT_SHIFT, Some((0, 45))),
            ("≈", "KeyX", ALT, Some((0, 45))),
            ("a", "KeyA", ALT, None),
            ("Enter", "Enter", ALT, None),
            // Ctrl gives only Ctrl+C
            ("c", "KeyC", CTRL, Some((3, 0))),
            ("C", "KeyC", CTRL_SHIFT, Some((3, 0))),
            ("a", "KeyA", CTRL, None),
            // Meta blocks every key
            ("a", "KeyA", META, None),
            // A lowercase letter has a scan code, an uppercase letter does not
            ("a", "KeyA", NONE, Some((97, 30))),
            ("b", "KeyB", NONE, Some((98, 48))),
            ("c", "KeyC", NONE, Some((99, 46))),
            ("d", "KeyD", NONE, Some((100, 32))),
            ("e", "KeyE", NONE, Some((101, 18))),
            ("f", "KeyF", NONE, Some((102, 33))),
            ("g", "KeyG", NONE, Some((103, 34))),
            ("h", "KeyH", NONE, Some((104, 35))),
            ("i", "KeyI", NONE, Some((105, 23))),
            ("j", "KeyJ", NONE, Some((106, 36))),
            ("k", "KeyK", NONE, Some((107, 37))),
            ("l", "KeyL", NONE, Some((108, 38))),
            ("m", "KeyM", NONE, Some((109, 50))),
            ("n", "KeyN", NONE, Some((110, 49))),
            ("o", "KeyO", NONE, Some((111, 24))),
            ("p", "KeyP", NONE, Some((112, 25))),
            ("q", "KeyQ", NONE, Some((113, 16))),
            ("r", "KeyR", NONE, Some((114, 19))),
            ("s", "KeyS", NONE, Some((115, 31))),
            ("t", "KeyT", NONE, Some((116, 20))),
            ("u", "KeyU", NONE, Some((117, 22))),
            ("v", "KeyV", NONE, Some((118, 47))),
            ("w", "KeyW", NONE, Some((119, 17))),
            ("x", "KeyX", NONE, Some((120, 45))),
            ("y", "KeyY", NONE, Some((121, 21))),
            ("z", "KeyZ", NONE, Some((122, 44))),
            ("A", "KeyA", SHIFT, Some((65, 0))),
            ("A", "KeyA", CAPS, Some((65, 0))),
            ("a", "KeyA", SHIFT_CAPS, Some((97, 30))),
            ("A", "KeyA", SHIFT_CAPS, Some((97, 30))),
            // Shift uses the US QWERTY layout
            ("!", "Digit1", SHIFT, Some((33, 0))),
            ("@", "Digit2", SHIFT, Some((64, 0))),
            ("#", "Digit3", SHIFT, Some((35, 0))),
            ("$", "Digit4", SHIFT, Some((36, 0))),
            ("%", "Digit5", SHIFT, Some((37, 0))),
            ("^", "Digit6", SHIFT, Some((94, 0))),
            ("&", "Digit7", SHIFT, Some((38, 0))),
            ("*", "Digit8", SHIFT, Some((42, 0))),
            ("(", "Digit9", SHIFT, Some((40, 0))),
            (")", "Digit0", SHIFT, Some((41, 0))),
            ("_", "Minus", SHIFT, Some((95, 0))),
            ("+", "Equal", SHIFT, Some((43, 0))),
            ("{", "BracketLeft", SHIFT, Some((123, 0))),
            ("}", "BracketRight", SHIFT, Some((125, 0))),
            (":", "Semicolon", SHIFT, Some((58, 0))),
            ("\"", "Quote", SHIFT, Some((34, 0))),
            ("~", "Backquote", SHIFT, Some((126, 0))),
            ("|", "Backslash", SHIFT, Some((124, 0))),
            ("<", "Comma", SHIFT, Some((60, 0))),
            (">", "Period", SHIFT, Some((62, 0))),
            ("?", "Slash", SHIFT, Some((63, 0))),
            // Shift+2 on a Finnish keyboard
            ("\"", "Digit2", SHIFT, Some((64, 0))),
            // Shift blocks the keys that are not in its table
            ("Enter", "Enter", SHIFT, None),
            ("Escape", "Escape", SHIFT, None),
            ("Backspace", "Backspace", SHIFT, None),
            ("Tab", "Tab", SHIFT, None),
            (" ", "Space", SHIFT, None),
            ("F1", "F1", SHIFT, None),
            ("ArrowLeft", "ArrowLeft", SHIFT, None),
            ("End", "Numpad1", SHIFT, None),
            ("1", "Numpad1", SHIFT, None),
            (".", "NumpadDecimal", SHIFT, None),
            ("+", "NumpadAdd", SHIFT, None),
            ("Enter", "NumpadEnter", SHIFT, None),
            // Characters without modifiers
            (" ", "Space", NONE, Some((32, 0))),
            ("0", "Digit0", NONE, Some((48, 0))),
            ("9", "Digit9", NONE, Some((57, 0))),
            (";", "Semicolon", NONE, Some((59, 0))),
            ("'", "Quote", NONE, Some((39, 0))),
            ("`", "Backquote", NONE, Some((96, 0))),
            (",", "Comma", NONE, Some((44, 0))),
            (".", "Period", NONE, Some((46, 0))),
            ("/", "Slash", NONE, Some((47, 0))),
            ("Enter", "Enter", NONE, Some((13, 0))),
            ("Escape", "Escape", NONE, Some((27, 0))),
            ("Backspace", "Backspace", NONE, Some((8, 0))),
            ("Tab", "Tab", NONE, Some((9, 0))),
            // Keys of a Finnish keyboard
            ("+", "Minus", NONE, Some((43, 0))),
            ("<", "IntlBackslash", NONE, Some((60, 0))),
            ("Dead", "Equal", NONE, None),
            // Special keys have only a scan code
            ("F1", "F1", NONE, Some((0, 59))),
            ("F2", "F2", NONE, Some((0, 60))),
            ("F3", "F3", NONE, Some((0, 61))),
            ("F4", "F4", NONE, Some((0, 62))),
            ("F5", "F5", NONE, Some((0, 63))),
            ("F6", "F6", NONE, Some((0, 64))),
            ("F7", "F7", NONE, Some((0, 65))),
            ("F8", "F8", NONE, Some((0, 66))),
            ("F9", "F9", NONE, Some((0, 67))),
            ("F10", "F10", NONE, Some((0, 68))),
            ("F11", "F11", NONE, None),
            ("ArrowLeft", "ArrowLeft", NONE, Some((0, 75))),
            ("ArrowRight", "ArrowRight", NONE, Some((0, 77))),
            ("ArrowUp", "ArrowUp", NONE, Some((0, 72))),
            ("ArrowDown", "ArrowDown", NONE, Some((0, 80))),
            ("Insert", "Insert", NONE, Some((0, 82))),
            ("Delete", "Delete", NONE, Some((0, 83))),
            ("Home", "Home", NONE, Some((0, 71))),
            ("End", "End", NONE, Some((0, 79))),
            ("PageUp", "PageUp", NONE, Some((0, 73))),
            ("PageDown", "PageDown", NONE, Some((0, 81))),
            // Keypad with Num Lock off
            ("End", "Numpad1", NONE, Some((0, 79))),
            ("ArrowDown", "Numpad2", NONE, Some((0, 80))),
            ("PageDown", "Numpad3", NONE, Some((0, 81))),
            ("ArrowLeft", "Numpad4", NONE, Some((0, 75))),
            ("Clear", "Numpad5", NONE, Some((0, 76))),
            ("ArrowRight", "Numpad6", NONE, Some((0, 77))),
            ("Home", "Numpad7", NONE, Some((0, 71))),
            ("ArrowUp", "Numpad8", NONE, Some((0, 72))),
            ("PageUp", "Numpad9", NONE, Some((0, 73))),
            ("Insert", "Numpad0", NONE, Some((0, 82))),
            ("Delete", "NumpadDecimal", NONE, Some((0, 83))),
            ("/", "NumpadDivide", NONE, Some((47, 0))),
            ("*", "NumpadMultiply", NONE, Some((42, 0))),
            ("-", "NumpadSubtract", NONE, Some((45, 0))),
            ("+", "NumpadAdd", NONE, Some((43, 0))),
            ("Enter", "NumpadEnter", NONE, Some((13, 0))),
            ("=", "NumpadEqual", NONE, Some((61, 0))),
            // Keypad with Num Lock on
            ("1", "Numpad1", NONE, Some((49, 0))),
            ("2", "Numpad2", NONE, Some((50, 0))),
            ("3", "Numpad3", NONE, Some((51, 0))),
            ("4", "Numpad4", NONE, Some((52, 0))),
            ("5", "Numpad5", NONE, Some((53, 0))),
            ("6", "Numpad6", NONE, Some((54, 0))),
            ("7", "Numpad7", NONE, Some((55, 0))),
            ("8", "Numpad8", NONE, Some((56, 0))),
            ("9", "Numpad9", NONE, Some((57, 0))),
            ("0", "Numpad0", NONE, Some((48, 0))),
            (".", "NumpadDecimal", NONE, Some((46, 0))),
            // The keypad decimal key of a Finnish keyboard
            (",", "NumpadDecimal", NONE, Some((46, 0))),
            // Nordic letters
            ("ä", "Quote", NONE, Some((132, 36))),
            ("å", "BracketLeft", NONE, Some((134, 26))),
            ("æ", "Quote", NONE, Some((145, 0))),
            ("ö", "Semicolon", NONE, Some((148, 39))),
            ("ø", "Semicolon", NONE, Some((158, 0))),
            ("ü", "BracketLeft", NONE, Some((129, 0))),
            ("ß", "Minus", NONE, Some((225, 0))),
            ("Ä", "Quote", SHIFT, Some((142, 0))),
            ("Å", "BracketLeft", SHIFT, Some((143, 0))),
            ("Æ", "Quote", SHIFT, Some((146, 0))),
            ("Ö", "Semicolon", SHIFT, Some((153, 0))),
            ("Ø", "Semicolon", SHIFT, Some((157, 0))),
            ("Ü", "BracketLeft", SHIFT, Some((154, 0))),
            ("Ä", "Quote", CAPS, Some((142, 0))),
            ("ä", "Quote", SHIFT_CAPS, Some((132, 36))),
            ("ß", "Minus", CAPS, Some((225, 0))),
            // Other Latin-1 letters give nothing
            ("ù", "Quote", NONE, None),
            ("è", "BracketLeft", NONE, None),
            // Modifier and status keys give nothing
            ("Shift", "ShiftLeft", SHIFT, None),
            ("Control", "ControlLeft", CTRL, None),
            ("CapsLock", "CapsLock", CAPS, None),
        ];
        for (key, code, modifiers, expected) in table {
            assert_eq!(
                WebPlatform::translate_key(key, code, None, false, modifiers),
                expected,
                "{key} {code}"
            );
        }
    }

    // The expected values come from the Pascal WaitForKeyPress, for the SDL key event of the same
    // key with the same layout.
    #[test]
    fn translate_key_with_layout() {
        let table = [
            // Finnish layout
            ("_", "Slash", Some('-'), SHIFT, Some((95, 0))),
            ("?", "Minus", Some('+'), SHIFT, None),
            ("*", "Backslash", Some('\''), SHIFT, Some((34, 0))),
            ("½", "Backquote", Some('§'), SHIFT, None),
            (">", "IntlBackslash", Some('<'), SHIFT, None),
            (";", "Comma", Some(','), SHIFT, Some((60, 0))),
            ("\"", "Digit2", Some('2'), SHIFT, Some((64, 0))),
            ("Ö", "Semicolon", Some('ö'), SHIFT, Some((153, 0))),
            ("-", "Slash", Some('-'), NONE, Some((45, 0))),
            // A letter after a dead key
            ("é", "KeyE", Some('e'), NONE, Some((101, 18))),
            ("É", "KeyE", Some('e'), SHIFT, Some((69, 0))),
            ("é", "KeyE", None, NONE, Some((101, 18))),
            // macOS Option+X
            ("≈", "KeyX", Some('x'), ALT, Some((0, 45))),
            // AZERTY layout
            ("&", "Digit1", Some('&'), NONE, Some((49, 0))),
            ("é", "Digit2", Some('é'), NONE, Some((50, 0))),
            ("1", "Digit1", Some('&'), SHIFT, Some((33, 0))),
            ("a", "KeyQ", Some('a'), NONE, Some((97, 30))),
            ("A", "KeyQ", Some('a'), SHIFT, Some((65, 0))),
            ("ù", "Quote", Some('ù'), NONE, None),
            ("%", "Quote", Some('ù'), SHIFT, None),
            ("m", "Semicolon", Some('m'), NONE, Some((109, 50))),
            ("x", "KeyX", Some('x'), ALT, Some((0, 45))),
        ];
        for (key, code, unshifted, modifiers, expected) in table {
            assert_eq!(
                WebPlatform::translate_key(key, code, unshifted, false, modifiers),
                expected,
                "{key} {code}"
            );
        }
    }

    #[test]
    fn keydown_learns_the_layout() {
        let platform = WebPlatform::new(LocalStorageStore::in_memory(), Files::new(&[0]));
        // Shift+'-' on a Finnish keyboard, before the layout is known
        platform.keydown("_", "Slash", SHIFT);
        assert_eq!(platform.process_keypress(), (63, 0));

        platform.keydown("-", "Slash", NONE);
        assert_eq!(platform.process_keypress(), (45, 0));
        platform.keydown("_", "Slash", SHIFT);
        assert_eq!(platform.process_keypress(), (95, 0));

        platform.set_unshifted_key("Minus", "+");
        platform.keydown("?", "Minus", SHIFT);
        assert_eq!(platform.process_keypress(), (0, 0));
    }

    #[test]
    fn right_alt_blocks_alt_x() {
        let platform = WebPlatform::new(LocalStorageStore::in_memory(), Files::new(&[0]));
        platform.keydown("Alt", "AltRight", ALT);
        assert!(!platform.keydown("x", "KeyX", ALT));
        platform.keyup("AltRight");
        platform.keydown("Alt", "AltLeft", ALT);
        assert!(platform.keydown("x", "KeyX", ALT));

        // A key press without Alt clears a Right Alt whose keyup was lost
        platform.keydown("Alt", "AltRight", ALT);
        platform.keydown("a", "KeyA", NONE);
        platform.keydown("Alt", "AltLeft", ALT);
        assert!(platform.keydown("x", "KeyX", ALT));
    }

    #[test]
    fn key_pressed_skips_modifier_keys() {
        let platform = WebPlatform::new(LocalStorageStore::in_memory(), Files::new(&[0]));
        platform.keydown("Shift", "ShiftLeft", SHIFT);
        platform.keydown("A", "KeyA", SHIFT);
        assert!(platform.is_key_pressed());
        assert_eq!(platform.process_keypress(), (65, 0));

        platform.keydown("Shift", "ShiftLeft", SHIFT);
        assert!(!platform.is_key_pressed());

        // WaitForKeyPress returns also for a modifier key
        platform.keydown("Shift", "ShiftLeft", SHIFT);
        assert_eq!(platform.process_keypress(), (0, 0));
    }
}
