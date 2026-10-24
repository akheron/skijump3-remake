use chrono::{Datelike, Timelike};
use common::files::{Overlay, OverlayFile, Store, WritableStore};
use common::{trace, Platform, TPalette};
use sdl2::keyboard::{Keycode, Mod, Scancode};
use sdl2::pixels::PixelFormatEnum::RGBA8888;
use sdl2::pixels::{Color, Palette, PixelFormatEnum};
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::surface::Surface;
use sdl2::video::{FullscreenType, Window, WindowContext};
use std::borrow::Cow;
use std::cell::{Cell, RefCell};
use std::future::Future;
use std::io::Cursor;
use std::path::PathBuf;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};
use std::{fs, thread};

pub const X_RES: u32 = 320;
pub const Y_RES: u32 = 200;
const TARGET_FRAMES: u32 = 70;
const ASPECT_RES: f64 = X_RES as f64 / Y_RES as f64;

pub struct SDLPortModule<'a> {
    files: Overlay<DirStore, DirStore>,
    window_multiplier: Cell<u32>,
    canvas: RefCell<&'a mut Canvas<Window>>,
    texture_creator: &'a TextureCreator<WindowContext>,
    original_surface: RefCell<Surface<'static>>,
    display_texture: RefCell<Texture<'a>>,
    palette: [Color; 256],
    traced_palette: Cell<TPalette>,
    render_dest_rect: Cell<Rect>,

    window_resized: Cell<bool>,
    full_screen: Cell<bool>,
    aspect: Cell<f64>,

    frame_count: Cell<u32>,
    sub_frame_duration: Cell<Duration>,
    last_frame_instant: Cell<Instant>,

    close_requested: Cell<bool>,

    // Originally in SJ3HELP.PAS
    pub ch: Cell<u8>,
    pub ch2: Cell<u8>,

    // Async
    pub async_state: Cell<AsyncState>,
    keypresses: RefCell<Vec<(Option<Keycode>, Option<Scancode>, Mod)>>,
}

pub fn get_arguments() -> bool {
    let mut sw_rendering = false;
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "--" => break,
            "-s" | "--sw-rendering" => sw_rendering = true,
            _ => {}
        }
    }
    sw_rendering
}

impl<'a> SDLPortModule<'a> {
    fn reset_window_size(&self) {
        let (w, h) = window_size(self.window_multiplier.get(), self.aspect.get());
        self.canvas.borrow_mut().window_mut().set_size(w, h).ok();
        self.window_resized.set(true);
    }

    pub fn init(
        canvas: &'a mut Canvas<Window>,
        texture_creator: &'a TextureCreator<WindowContext>,
        window_multiplier: u32,
        data_dir: PathBuf,
    ) -> Self {
        let aspect = ASPECT_RES;

        // 8-bit surface for original data
        let original_surface = Surface::new(X_RES, Y_RES, PixelFormatEnum::Index8).unwrap();

        // Finally a texture for displaying 32-bit display data
        let display_texture = texture_creator
            .create_texture_streaming(RGBA8888, X_RES, Y_RES)
            .unwrap();

        let render_dest_rect = get_render_rect(canvas.output_size().unwrap(), aspect);
        SDLPortModule {
            files: Overlay::new(
                DirStore { dir: data_dir },
                DirStore {
                    dir: PathBuf::from("assets"),
                },
            ),
            window_multiplier: Cell::new(window_multiplier),
            canvas: RefCell::new(canvas),
            texture_creator,
            original_surface: RefCell::new(original_surface),
            display_texture: RefCell::new(display_texture),
            palette: [Color::RGB(0, 0, 0); 256],
            traced_palette: Cell::new([[0; 3]; 256]),
            render_dest_rect: Cell::new(render_dest_rect),

            window_resized: Cell::new(false),
            full_screen: Cell::new(false),
            aspect: Cell::new(aspect),

            frame_count: Cell::new(0),
            sub_frame_duration: Cell::new(Duration::new(0, 0)),
            last_frame_instant: Cell::new(Instant::now()),

            close_requested: Cell::new(false),

            ch: Cell::new(1),
            ch2: Cell::new(1),

            async_state: Cell::new(AsyncState::None),
            keypresses: RefCell::new(Vec::new()),
        }
    }

    fn set_palette(&self, input: &TPalette) {
        self.traced_palette.set(*input);
        let colors = input
            .iter()
            .map(|color| Color::RGB(color[0] * 4, color[1] * 4, color[2] * 4))
            .collect::<Vec<_>>();
        let palette = Palette::with_colors(&colors).unwrap();
        self.original_surface
            .borrow_mut()
            .set_palette(&palette)
            .unwrap();
    }

    fn render_phase1(&self, buffer: &[u8]) {
        if trace::active() {
            trace::frame(buffer, &self.traced_palette.get());
        }
        let mut original_surface = self.original_surface.borrow_mut();
        original_surface.with_lock_mut(|pixels| {
            pixels.copy_from_slice(buffer);
        });
    }

    fn render_phase3(&self) {
        self.wait_raster();

        if self.window_resized.replace(false) {
            self.render_dest_rect.set(get_render_rect(
                self.canvas.borrow().output_size().unwrap(),
                self.aspect.get(),
            ));
        }

        let mut canvas = self.canvas.borrow_mut();
        canvas.clear();

        // Filled by render_phase1()
        let original_surface = self.original_surface.borrow();

        // Surface to texture
        let mut display_texture = self.display_texture.borrow_mut();
        display_texture
            .with_lock(None, |dest, pitch| {
                original_surface
                    .convert_format(RGBA8888)
                    .unwrap()
                    .with_lock(|src| {
                        dest.copy_from_slice(src);
                    });
            })
            .unwrap();

        // Render texture to display
        canvas
            .copy(&display_texture, None, self.render_dest_rect.get())
            .unwrap();
        canvas.present();
    }

    fn wait_raster(&self) {
        if trace::active() {
            return;
        }
        let frame_duration = Duration::from_nanos(1_000_000_000 / TARGET_FRAMES as u64);
        let mut sub_frame_duration = self.sub_frame_duration.get();
        let mut frame_count = self.frame_count.get();
        let last_frame_instant = self.last_frame_instant.get();

        let elapsed = last_frame_instant.elapsed();
        if elapsed < frame_duration {
            thread::sleep(frame_duration - elapsed);
        }

        let now = Instant::now();
        sub_frame_duration += now - last_frame_instant;
        assert!(sub_frame_duration >= frame_duration);
        while sub_frame_duration >= frame_duration {
            sub_frame_duration -= frame_duration;
            frame_count += 1;
        }

        self.frame_count.set(frame_count);
        self.sub_frame_duration.set(sub_frame_duration);
        self.last_frame_instant.set(now);
    }

    fn key_pressed(&self) -> bool {
        if trace::active() {
            return trace::key_pressed();
        }
        let mut keypresses = self.keypresses.borrow_mut();
        while keypresses.len() > 0 {
            let (keycode, scancode, keymod) = keypresses.remove(0);
            match keycode {
                // Ignore modifier and status keys, since they did not trigger a keypress in DOS version.
                // F11, F12 and probably others were ignored too, but that would probably be counterintuitive.
                Some(
                    Keycode::LCTRL
                    | Keycode::RCTRL
                    | Keycode::LSHIFT
                    | Keycode::RSHIFT
                    | Keycode::LALT
                    | Keycode::RALT
                    | Keycode::LGUI
                    | Keycode::RGUI
                    | Keycode::CAPSLOCK
                    | Keycode::SCROLLLOCK
                    | Keycode::NUMLOCKCLEAR
                    | Keycode::PRINTSCREEN
                    | Keycode::SYSREQ
                    | Keycode::PAUSE,
                ) => {}
                _ => {
                    keypresses.insert(0, (keycode, scancode, keymod));
                    return true;
                }
            }
        }
        return false;
    }

    fn toggle_fullscreen(&self) {
        self.window_resized.set(true);

        self.full_screen.set(!self.full_screen.get());
        let flags = if self.full_screen.get() {
            FullscreenType::Desktop
        } else {
            FullscreenType::Off
        };

        self.canvas
            .borrow_mut()
            .window_mut()
            .set_fullscreen(flags)
            .ok();
    }

    fn process_keypress(mut keycode: Keycode, keymod: Mod) -> (u8, u8) {
        // Setting ch2 to a correct scancode value for the key pressed can be largely ignored.
        // It is only checked by the game for special keys, where ch1 is checked or assumed to be 0.
        // This also means that the game cannot differentiate between different keys producing the same character.
        let mut ch1 = 0u8;
        let mut ch2 = 0u8;

        // Special cases for key combinations used throughout the game.
        // Check for Right Alt/AltGr first, since pressing it can make Left Ctrl look as pressed too.
        if keymod.intersects(Mod::MODEMOD) {
            return (0, 0);
        }
        if keymod.intersects(Mod::RALTMOD) {
            return (0, 0);
        }
        if keymod.intersects(Mod::LALTMOD) {
            return match keycode {
                // ALT-X
                Keycode::X => (0, 45),
                _ => (0, 0),
            };
        }
        if keymod.intersects(Mod::LCTRLMOD | Mod::RCTRLMOD) {
            return match keycode {
                // CTRL-C
                Keycode::C => (3, 0),
                _ => (0, 0),
            };
        }
        if keymod.intersects(Mod::LGUIMOD | Mod::RGUIMOD) {
            return (0, 0);
        }

        // Handle conversion of letters to uppercase.
        // Convert only if Caps Lock is off and Shift is pressed or vice versa.
        if keymod.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD) ^ keymod.intersects(Mod::CAPSMOD) {
            if let 97..=122 | 224..=246 | 248..=254 = *keycode {
                keycode = Keycode::from_i32(*keycode - 32).unwrap();
            }
        }

        // If modifier is Shift, convert the key pressed accordingly.
        // For special keys, standard US QWERTY layout is assumed.
        if keymod.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD) {
            keycode = match keycode {
                // Already handled above
                k if matches!(
                    *k,
                    65..=90 | 97..=122 | 190..=214 | 216..=222 | 224..=246 | 248..=254
                ) =>
                {
                    k
                }
                Keycode::NUM_1 => Keycode::EXCLAIM,
                Keycode::NUM_2 => Keycode::AT,
                Keycode::NUM_3 => Keycode::HASH,
                Keycode::NUM_4 => Keycode::DOLLAR,
                Keycode::NUM_5 => Keycode::PERCENT,
                Keycode::NUM_6 => Keycode::CARET,
                Keycode::NUM_7 => Keycode::AMPERSAND,
                Keycode::NUM_8 => Keycode::ASTERISK,
                Keycode::NUM_9 => Keycode::LEFTPAREN,
                Keycode::NUM_0 => Keycode::RIGHTPAREN,
                Keycode::MINUS => Keycode::UNDERSCORE,
                Keycode::EQUALS => Keycode::PLUS,
                Keycode::LEFTBRACKET => Keycode::from_i32('{' as i32).unwrap(),
                Keycode::RIGHTBRACKET => Keycode::from_i32('}' as i32).unwrap(),
                Keycode::SEMICOLON => Keycode::COLON,
                Keycode::QUOTE => Keycode::QUOTEDBL,
                Keycode::BACKQUOTE => Keycode::from_i32('~' as i32).unwrap(),
                Keycode::BACKSLASH => Keycode::from_i32('|' as i32).unwrap(),
                Keycode::COMMA => Keycode::LESS,
                Keycode::PERIOD => Keycode::GREATER,
                Keycode::SLASH => Keycode::QUESTION,
                _ => return (0, 0),
            };
        }

        // If NumLock modifier is not set, convert the charaters accordingly
        if !keymod.intersects(Mod::NUMMOD) {
            keycode = match keycode {
                Keycode::KP_1 => Keycode::END,
                Keycode::KP_2 => Keycode::DOWN,
                Keycode::KP_3 => Keycode::PAGEDOWN,
                Keycode::KP_4 => Keycode::LEFT,
                Keycode::KP_5 => return (0, 76),
                Keycode::KP_6 => Keycode::RIGHT,
                Keycode::KP_7 => Keycode::HOME,
                Keycode::KP_8 => Keycode::UP,
                Keycode::KP_9 => Keycode::PAGEUP,
                Keycode::KP_0 => Keycode::INSERT,
                Keycode::KP_PERIOD => Keycode::DELETE,
                k => k,
            };
        }

        // Merge keypad characters with their regular counterparts.
        // It is not needed to differentiate between them, since the scancode isn't checked for normal characters.
        // Checks for Shift and Num Lock were already done, so they won't interfere with the merge.
        keycode = match keycode {
            Keycode::KP_DIVIDE => Keycode::SLASH,
            Keycode::KP_MULTIPLY => Keycode::ASTERISK,
            Keycode::KP_MINUS => Keycode::MINUS,
            Keycode::KP_PLUS => Keycode::PLUS,
            Keycode::KP_ENTER => Keycode::RETURN,
            Keycode::KP_1 => Keycode::NUM_1,
            Keycode::KP_2 => Keycode::NUM_2,
            Keycode::KP_3 => Keycode::NUM_3,
            Keycode::KP_4 => Keycode::NUM_4,
            Keycode::KP_5 => Keycode::NUM_5,
            Keycode::KP_6 => Keycode::NUM_6,
            Keycode::KP_7 => Keycode::NUM_7,
            Keycode::KP_8 => Keycode::NUM_8,
            Keycode::KP_9 => Keycode::NUM_9,
            Keycode::KP_0 => Keycode::NUM_0,
            Keycode::KP_PERIOD => Keycode::PERIOD,
            Keycode::KP_EQUALS => Keycode::EQUALS,
            k => k,
        };

        match keycode {
            Keycode::RETURN => ch1 = 13,
            Keycode::ESCAPE => ch1 = 27,
            Keycode::BACKSPACE => ch1 = 8,
            Keycode::TAB => ch1 = 9,
            _ => {}
        }
        match *keycode {
            32..=126 => ch1 = *keycode as u8,

            // Special characters supported by the game, OEM 865 nordic encoding is used for ch1 values
            196 => return (142, 0), // A with diaeresis
            197 => return (143, 0), // A with ring above
            198 => return (146, 0), // AE
            214 => return (153, 0), // O with diaeresis
            216 => return (157, 0), // O with stroke
            220 => return (154, 0), // U with diaeresis
            223 => return (225, 0), // sharp s

            228 => return (132, 36), // a with diaeresis
            229 => return (134, 26), // a with ring above
            230 => return (145, 0),  // ae
            246 => return (148, 39), // o with diaeresis
            248 => return (158, 0),  // o with stroke
            252 => return (129, 0),  // u with diaeresis
            _ => {}
        }

        match keycode {
            Keycode::F1 => ch2 = 59,
            Keycode::F2 => ch2 = 60,
            Keycode::F3 => ch2 = 61,
            Keycode::F4 => ch2 = 62,
            Keycode::F5 => ch2 = 63,
            Keycode::F6 => ch2 = 64,
            Keycode::F7 => ch2 = 65,
            Keycode::F8 => ch2 = 66,
            Keycode::F9 => ch2 = 67,
            Keycode::F10 => ch2 = 68,

            Keycode::LEFT => ch2 = 75,
            Keycode::RIGHT => ch2 = 77,
            Keycode::UP => ch2 = 72,
            Keycode::DOWN => ch2 = 80,

            Keycode::INSERT => ch2 = 82,
            Keycode::DELETE => ch2 = 83,
            Keycode::HOME => ch2 = 71,
            Keycode::END => ch2 = 79,
            Keycode::PAGEUP => ch2 = 73,
            Keycode::PAGEDOWN => ch2 = 81,

            Keycode::A => ch2 = 30,
            Keycode::B => ch2 = 48,
            Keycode::C => ch2 = 46,
            Keycode::D => ch2 = 32,
            Keycode::E => ch2 = 18,
            Keycode::F => ch2 = 33,
            Keycode::G => ch2 = 34,
            Keycode::H => ch2 = 35,
            Keycode::I => ch2 = 23,
            Keycode::J => ch2 = 36,
            Keycode::K => ch2 = 37,
            Keycode::L => ch2 = 38,
            Keycode::M => ch2 = 50,
            Keycode::N => ch2 = 49,
            Keycode::O => ch2 = 24,
            Keycode::P => ch2 = 25,
            Keycode::Q => ch2 = 16,
            Keycode::R => ch2 = 19,
            Keycode::S => ch2 = 31,
            Keycode::T => ch2 = 20,
            Keycode::U => ch2 = 22,
            Keycode::V => ch2 = 47,
            Keycode::W => ch2 = 17,
            Keycode::X => ch2 = 45,
            Keycode::Y => ch2 = 21,
            Keycode::Z => ch2 = 44,
            _ => {}
        }
        (ch1, ch2)
    }

    // Originally in SJ3UNIT.PAS
    fn kword(&self) -> u16 {
        ((self.ch.get() as u16) << 8) + self.ch2.get() as u16
    }

    // Everything down from here originally in SJ3HELP.PAS
    fn putsaa(&self) {
        while self.key_pressed() {
            let (ch, ch2) = self.next_keypress();
            self.ch.set(ch);
            self.ch2.set(ch2);
        }
    }

    fn clearchs(&self) {
        self.ch.set(1);
        self.ch2.set(1);
    }

    // Async state management

    pub fn handle_resized(&self) {
        self.window_resized.set(true);
    }

    pub fn handle_quit(&self) {
        self.close_requested.set(true);
    }

    pub fn handle_keydown(
        &self,
        keycode: Option<Keycode>,
        scancode: Option<Scancode>,
        keymod: Mod,
    ) {
        self.keypresses
            .borrow_mut()
            .push((keycode, scancode, keymod));
    }

    pub fn main_loop(&self) {
        match self.async_state.get() {
            AsyncState::Render => {
                self.render_phase3();
                self.async_state.set(AsyncState::None);
            }
            AsyncState::WaitForKeyPress => {
                if trace::active() || !self.keypresses.borrow().is_empty() {
                    let (ch, ch2) = self.next_keypress();
                    self.ch.set(ch);
                    self.ch2.set(ch2);
                    self.async_state.set(AsyncState::None);
                } else {
                    thread::sleep(Duration::from_millis(10));
                }
            }
            AsyncState::KeyPressed => {
                // Just visit the main loop once to process events
                self.async_state.set(AsyncState::None);
            }
            AsyncState::None => {
                panic!("Unexpected AsyncState::None");
            }
        }
    }

    fn next_keypress(&self) -> (u8, u8) {
        if trace::active() {
            return trace::keypress();
        }
        let (keycode, scancode, keymod) = self.keypresses.borrow_mut().remove(0);

        // SDL version specific shortcuts
        if keymod.intersects(Mod::LALTMOD) {
            match scancode {
                Some(Scancode::Return) => {
                    self.toggle_fullscreen();
                    return (0, 0);
                }
                Some(Scancode::R) => {
                    self.reset_window_size();
                    return (0, 0);
                }
                Some(Scancode::KpPlus | Scancode::Equals) => {
                    self.window_multiplier.set(self.window_multiplier.get() + 1);
                    self.reset_window_size();
                    return (0, 0);
                }
                Some(Scancode::KpMinus | Scancode::Minus) => {
                    if self.window_multiplier.get() > 1 {
                        self.window_multiplier.set(self.window_multiplier.get() - 1);
                        self.reset_window_size();
                        return (0, 0);
                    }
                }
                Some(Scancode::A) => {
                    if self.aspect.get() != ASPECT_RES {
                        self.aspect.set(ASPECT_RES);
                    } else {
                        self.aspect.set(4.0 / 3.0);
                    }
                    self.reset_window_size();
                    return (0, 0);
                }
                _ => {}
            }
        }

        keycode.map_or((0, 0), |keycode| Self::process_keypress(keycode, keymod))
    }
}

// Pascal Round rounds halves to even.
fn get_render_rect((window_w, window_h): (u32, u32), aspect: f64) -> Rect {
    if window_w as f64 / window_h as f64 <= aspect {
        let h = (window_w as f64 / aspect).round_ties_even() as u32;
        Rect::new(0, ((window_h - h) / 2) as i32, window_w, h)
    } else {
        let w = (window_h as f64 * aspect).round_ties_even() as u32;
        Rect::new(((window_w - w) / 2) as i32, 0, w, window_h)
    }
}

fn window_size(window_multiplier: u32, aspect: f64) -> (u32, u32) {
    (
        X_RES * window_multiplier,
        ((Y_RES * window_multiplier) as f64 * ASPECT_RES / aspect).round_ties_even() as u32,
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AsyncState {
    None,
    Render,
    WaitForKeyPress,
    KeyPressed,
}

pub enum RenderResult {
    Rendered,
    Skipped,
    Wait(Duration),
}

struct SDLPortFuture<'s, 'si> {
    s: &'s SDLPortModule<'si>,
}

impl<'s, 'si> Future for SDLPortFuture<'s, 'si> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let AsyncState::None = self.s.async_state.get() {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

impl<'si> Platform for SDLPortModule<'si> {
    fn set_palette(&self, input: &TPalette) {
        self.set_palette(input);
    }
    fn render_phase1(&self, buffer: &[u8]) {
        self.render_phase1(buffer);
    }
    async fn render_phase2(&self) {
        self.async_state.set(AsyncState::Render);
        SDLPortFuture { s: self }.await
    }
    async fn key_pressed(&self) -> bool {
        // Yield to the main loop to process events
        self.async_state.set(AsyncState::KeyPressed);
        SDLPortFuture { s: self }.await;
        self.key_pressed()
    }
    async fn wait_for_key_press(&self) -> (u8, u8) {
        // After a close, the window is hidden and cannot get a key
        if !self.close_requested.get() {
            self.async_state.set(AsyncState::WaitForKeyPress);
            SDLPortFuture { s: self }.await;
        }
        (self.ch.get(), self.ch2.get())
    }
    fn putsaa(&self) {
        self.putsaa();
    }
    fn clearchs(&self) {
        self.clearchs();
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

    type WritableFile<'a>
        = OverlayFile<'a, DirStore, DirStore>
    where
        Self: 'a;
    fn create_file<'a, P: AsRef<str>>(&'a self, path: P) -> Self::WritableFile<'a> {
        self.files.create(path.as_ref())
    }

    type ReadableFile<'a>
        = Cursor<Cow<'a, [u8]>>
    where
        Self: 'a;
    fn open_file<'a, P: AsRef<str>>(&'a self, path: P) -> Self::ReadableFile<'a> {
        let path = path.as_ref();
        Cursor::new(
            self.files
                .read(path)
                .unwrap_or_else(|| panic!("Unknown file {path}")),
        )
    }

    fn file_exists<P: AsRef<str>>(&self, path: P) -> bool {
        self.files.exists(path.as_ref())
    }
    fn remove_file<P: AsRef<str>>(&self, path: P) {
        self.files.remove(path.as_ref());
    }
    fn find_files(&self, pattern: &str) -> Vec<String> {
        if trace::active() {
            return trace::find_files(pattern);
        }
        self.files.find(pattern)
    }

    fn get_date(&self) -> (u16, u16, u16, u16) {
        if trace::active() {
            return trace::date();
        }
        let now = chrono::Local::now();
        (
            now.year() as u16,
            now.month() as u16,
            now.day() as u16,
            now.weekday().num_days_from_sunday() as u16,
        )
    }
    fn get_time(&self) -> (u16, u16, u16, u16) {
        if trace::active() {
            return trace::time();
        }
        let now = chrono::Local::now();
        (
            now.hour() as u16,
            now.minute() as u16,
            now.second() as u16,
            // A leap second has more than 10^9 nanoseconds
            (now.nanosecond() % 1_000_000_000 / 10_000_000) as u16,
        )
    }

    fn param_str(&self, index: usize) -> String {
        std::env::args_os()
            .nth(index)
            .map(|arg| arg.to_string_lossy().into_owned())
            .unwrap_or_default()
    }
    fn wait(&self, ms: u32) {
        if trace::active() {
            return;
        }
        thread::sleep(Duration::from_millis(ms as u64));
    }

    fn deinit_graphics(&self) {
        self.canvas.borrow_mut().window_mut().hide();
    }
    fn close_requested(&self) -> bool {
        self.close_requested.get()
    }
}

pub struct DirStore {
    dir: PathBuf,
}

impl Store for DirStore {
    fn read(&self, name: &str) -> Option<Cow<'_, [u8]>> {
        fs::read(self.dir.join(name)).ok().map(Cow::Owned)
    }
    fn contains(&self, name: &str) -> bool {
        self.dir.join(name).is_file()
    }
    fn names(&self) -> Vec<String> {
        let Ok(entries) = fs::read_dir(&self.dir) else {
            return Vec::new();
        };
        entries
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_ok_and(|t| t.is_file()))
            .filter_map(|entry| entry.file_name().into_string().ok())
            .collect()
    }
}

impl WritableStore for DirStore {
    fn write(&self, name: &str, content: &[u8]) {
        fs::write(self.dir.join(name), content).unwrap();
    }
    fn remove(&self, name: &str) {
        fs::remove_file(self.dir.join(name)).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::{get_render_rect, window_size, SDLPortModule, ASPECT_RES};
    use sdl2::keyboard::{Keycode, Mod};
    use sdl2::rect::Rect;

    // The expected values come from the Pascal WaitForKeyPress.
    #[test]
    fn process_keypress() {
        let table = [
            // Right Alt and AltGr block every key, also Ctrl+C
            (key('c'), Mod::RALTMOD, (0, 0)),
            (key('c'), Mod::MODEMOD, (0, 0)),
            (key('c'), Mod::LCTRLMOD | Mod::RALTMOD, (0, 0)),
            // Left Alt gives only Alt+X
            (key('x'), Mod::LALTMOD, (0, 45)),
            (key('x'), Mod::LALTMOD | Mod::LSHIFTMOD, (0, 45)),
            (key('a'), Mod::LALTMOD, (0, 0)),
            (Keycode::RETURN, Mod::LALTMOD, (0, 0)),
            // Either Ctrl gives only Ctrl+C
            (key('c'), Mod::LCTRLMOD, (3, 0)),
            (key('c'), Mod::RCTRLMOD, (3, 0)),
            (key('c'), Mod::LCTRLMOD | Mod::LSHIFTMOD, (3, 0)),
            (key('a'), Mod::LCTRLMOD, (0, 0)),
            // GUI blocks every key
            (key('a'), Mod::LGUIMOD, (0, 0)),
            (key('a'), Mod::RGUIMOD, (0, 0)),
            // A lowercase letter has a scan code, an uppercase letter does not
            (key('a'), Mod::NOMOD, (97, 30)),
            (key('b'), Mod::NOMOD, (98, 48)),
            (key('c'), Mod::NOMOD, (99, 46)),
            (key('d'), Mod::NOMOD, (100, 32)),
            (key('e'), Mod::NOMOD, (101, 18)),
            (key('f'), Mod::NOMOD, (102, 33)),
            (key('g'), Mod::NOMOD, (103, 34)),
            (key('h'), Mod::NOMOD, (104, 35)),
            (key('i'), Mod::NOMOD, (105, 23)),
            (key('j'), Mod::NOMOD, (106, 36)),
            (key('k'), Mod::NOMOD, (107, 37)),
            (key('l'), Mod::NOMOD, (108, 38)),
            (key('m'), Mod::NOMOD, (109, 50)),
            (key('n'), Mod::NOMOD, (110, 49)),
            (key('o'), Mod::NOMOD, (111, 24)),
            (key('p'), Mod::NOMOD, (112, 25)),
            (key('q'), Mod::NOMOD, (113, 16)),
            (key('r'), Mod::NOMOD, (114, 19)),
            (key('s'), Mod::NOMOD, (115, 31)),
            (key('t'), Mod::NOMOD, (116, 20)),
            (key('u'), Mod::NOMOD, (117, 22)),
            (key('v'), Mod::NOMOD, (118, 47)),
            (key('w'), Mod::NOMOD, (119, 17)),
            (key('x'), Mod::NOMOD, (120, 45)),
            (key('y'), Mod::NOMOD, (121, 21)),
            (key('z'), Mod::NOMOD, (122, 44)),
            (key('a'), Mod::LSHIFTMOD, (65, 0)),
            (key('z'), Mod::RSHIFTMOD, (90, 0)),
            (key('a'), Mod::LSHIFTMOD | Mod::RSHIFTMOD, (65, 0)),
            (key('a'), Mod::CAPSMOD, (65, 0)),
            (key('a'), Mod::LSHIFTMOD | Mod::CAPSMOD, (97, 30)),
            (key('a'), Mod::NUMMOD | Mod::CAPSMOD, (65, 0)),
            // Shift uses the US QWERTY layout
            (key('1'), Mod::LSHIFTMOD, (33, 0)),
            (key('2'), Mod::LSHIFTMOD, (64, 0)),
            (key('3'), Mod::LSHIFTMOD, (35, 0)),
            (key('4'), Mod::LSHIFTMOD, (36, 0)),
            (key('5'), Mod::LSHIFTMOD, (37, 0)),
            (key('6'), Mod::LSHIFTMOD, (94, 0)),
            (key('7'), Mod::LSHIFTMOD, (38, 0)),
            (key('8'), Mod::LSHIFTMOD, (42, 0)),
            (key('9'), Mod::LSHIFTMOD, (40, 0)),
            (key('0'), Mod::LSHIFTMOD, (41, 0)),
            (key('-'), Mod::LSHIFTMOD, (95, 0)),
            (key('='), Mod::LSHIFTMOD, (43, 0)),
            (key('['), Mod::LSHIFTMOD, (123, 0)),
            (key(']'), Mod::LSHIFTMOD, (125, 0)),
            (key(';'), Mod::LSHIFTMOD, (58, 0)),
            (key('\''), Mod::LSHIFTMOD, (34, 0)),
            (key('`'), Mod::LSHIFTMOD, (126, 0)),
            (key('\\'), Mod::LSHIFTMOD, (124, 0)),
            (key(','), Mod::LSHIFTMOD, (60, 0)),
            (key('.'), Mod::LSHIFTMOD, (62, 0)),
            (key('/'), Mod::LSHIFTMOD, (63, 0)),
            // Shift blocks the keys that are not in its table
            (Keycode::RETURN, Mod::LSHIFTMOD, (0, 0)),
            (Keycode::ESCAPE, Mod::LSHIFTMOD, (0, 0)),
            (Keycode::BACKSPACE, Mod::LSHIFTMOD, (0, 0)),
            (Keycode::TAB, Mod::LSHIFTMOD, (0, 0)),
            (key(' '), Mod::LSHIFTMOD, (0, 0)),
            (Keycode::F1, Mod::LSHIFTMOD, (0, 0)),
            (Keycode::LEFT, Mod::LSHIFTMOD, (0, 0)),
            (Keycode::HOME, Mod::LSHIFTMOD, (0, 0)),
            (Keycode::KP_1, Mod::LSHIFTMOD, (0, 0)),
            (Keycode::KP_ENTER, Mod::LSHIFTMOD, (0, 0)),
            (key('ß'), Mod::LSHIFTMOD, (0, 0)),
            (Keycode::LSHIFT, Mod::LSHIFTMOD, (0, 0)),
            (Keycode::KP_1, Mod::NUMMOD | Mod::LSHIFTMOD, (0, 0)),
            // Characters without modifiers
            (key(' '), Mod::NOMOD, (32, 0)),
            (key('!'), Mod::NOMOD, (33, 0)),
            (key('0'), Mod::NOMOD, (48, 0)),
            (key('9'), Mod::NOMOD, (57, 0)),
            (key(';'), Mod::NOMOD, (59, 0)),
            (key('\''), Mod::NOMOD, (39, 0)),
            (key('`'), Mod::NOMOD, (96, 0)),
            (key(','), Mod::NOMOD, (44, 0)),
            (key('.'), Mod::NOMOD, (46, 0)),
            (key('/'), Mod::NOMOD, (47, 0)),
            (key('<'), Mod::NOMOD, (60, 0)),
            (Keycode::RETURN, Mod::NOMOD, (13, 0)),
            (Keycode::ESCAPE, Mod::NOMOD, (27, 0)),
            (Keycode::BACKSPACE, Mod::NOMOD, (8, 0)),
            (Keycode::TAB, Mod::NOMOD, (9, 0)),
            // Special keys have only a scan code
            (Keycode::F1, Mod::NOMOD, (0, 59)),
            (Keycode::F2, Mod::NOMOD, (0, 60)),
            (Keycode::F3, Mod::NOMOD, (0, 61)),
            (Keycode::F4, Mod::NOMOD, (0, 62)),
            (Keycode::F5, Mod::NOMOD, (0, 63)),
            (Keycode::F6, Mod::NOMOD, (0, 64)),
            (Keycode::F7, Mod::NOMOD, (0, 65)),
            (Keycode::F8, Mod::NOMOD, (0, 66)),
            (Keycode::F9, Mod::NOMOD, (0, 67)),
            (Keycode::F10, Mod::NOMOD, (0, 68)),
            (Keycode::F11, Mod::NOMOD, (0, 0)),
            (Keycode::LEFT, Mod::NOMOD, (0, 75)),
            (Keycode::RIGHT, Mod::NOMOD, (0, 77)),
            (Keycode::UP, Mod::NOMOD, (0, 72)),
            (Keycode::DOWN, Mod::NOMOD, (0, 80)),
            (Keycode::INSERT, Mod::NOMOD, (0, 82)),
            (Keycode::DELETE, Mod::NOMOD, (0, 83)),
            (Keycode::HOME, Mod::NOMOD, (0, 71)),
            (Keycode::END, Mod::NOMOD, (0, 79)),
            (Keycode::PAGEUP, Mod::NOMOD, (0, 73)),
            (Keycode::PAGEDOWN, Mod::NOMOD, (0, 81)),
            // Keypad with Num Lock off
            (Keycode::KP_1, Mod::NOMOD, (0, 79)),
            (Keycode::KP_2, Mod::NOMOD, (0, 80)),
            (Keycode::KP_3, Mod::NOMOD, (0, 81)),
            (Keycode::KP_4, Mod::NOMOD, (0, 75)),
            (Keycode::KP_5, Mod::NOMOD, (0, 76)),
            (Keycode::KP_6, Mod::NOMOD, (0, 77)),
            (Keycode::KP_7, Mod::NOMOD, (0, 71)),
            (Keycode::KP_8, Mod::NOMOD, (0, 72)),
            (Keycode::KP_9, Mod::NOMOD, (0, 73)),
            (Keycode::KP_0, Mod::NOMOD, (0, 82)),
            (Keycode::KP_PERIOD, Mod::NOMOD, (0, 83)),
            (Keycode::KP_DIVIDE, Mod::NOMOD, (47, 0)),
            (Keycode::KP_MULTIPLY, Mod::NOMOD, (42, 0)),
            (Keycode::KP_MINUS, Mod::NOMOD, (45, 0)),
            (Keycode::KP_PLUS, Mod::NOMOD, (43, 0)),
            (Keycode::KP_ENTER, Mod::NOMOD, (13, 0)),
            (Keycode::KP_EQUALS, Mod::NOMOD, (61, 0)),
            // Keypad with Num Lock on
            (Keycode::KP_1, Mod::NUMMOD, (49, 0)),
            (Keycode::KP_2, Mod::NUMMOD, (50, 0)),
            (Keycode::KP_3, Mod::NUMMOD, (51, 0)),
            (Keycode::KP_4, Mod::NUMMOD, (52, 0)),
            (Keycode::KP_5, Mod::NUMMOD, (53, 0)),
            (Keycode::KP_6, Mod::NUMMOD, (54, 0)),
            (Keycode::KP_7, Mod::NUMMOD, (55, 0)),
            (Keycode::KP_8, Mod::NUMMOD, (56, 0)),
            (Keycode::KP_9, Mod::NUMMOD, (57, 0)),
            (Keycode::KP_0, Mod::NUMMOD, (48, 0)),
            (Keycode::KP_PERIOD, Mod::NUMMOD, (46, 0)),
            (Keycode::KP_DIVIDE, Mod::NUMMOD, (47, 0)),
            (Keycode::KP_MULTIPLY, Mod::NUMMOD, (42, 0)),
            (Keycode::KP_MINUS, Mod::NUMMOD, (45, 0)),
            (Keycode::KP_PLUS, Mod::NUMMOD, (43, 0)),
            (Keycode::KP_ENTER, Mod::NUMMOD, (13, 0)),
            (Keycode::KP_EQUALS, Mod::NUMMOD, (61, 0)),
            // Nordic letters
            (key('ä'), Mod::NOMOD, (132, 36)),
            (key('å'), Mod::NOMOD, (134, 26)),
            (key('æ'), Mod::NOMOD, (145, 0)),
            (key('ö'), Mod::NOMOD, (148, 39)),
            (key('ø'), Mod::NOMOD, (158, 0)),
            (key('ü'), Mod::NOMOD, (129, 0)),
            (key('ß'), Mod::NOMOD, (225, 0)),
            (key('ä'), Mod::LSHIFTMOD, (142, 0)),
            (key('å'), Mod::LSHIFTMOD, (143, 0)),
            (key('æ'), Mod::LSHIFTMOD, (146, 0)),
            (key('ö'), Mod::LSHIFTMOD, (153, 0)),
            (key('ø'), Mod::LSHIFTMOD, (157, 0)),
            (key('ü'), Mod::LSHIFTMOD, (154, 0)),
            (key('ä'), Mod::CAPSMOD, (142, 0)),
            (key('ä'), Mod::LSHIFTMOD | Mod::CAPSMOD, (132, 36)),
            (key('ß'), Mod::CAPSMOD, (225, 0)),
            // Other Latin-1 letters give nothing
            (key('é'), Mod::NOMOD, (0, 0)),
            (key('é'), Mod::LSHIFTMOD, (0, 0)),
            // Modifier and status keys give nothing
            (Keycode::LSHIFT, Mod::LSHIFTMOD, (0, 0)),
            (Keycode::LCTRL, Mod::LCTRLMOD, (0, 0)),
            (Keycode::CAPSLOCK, Mod::CAPSMOD, (0, 0)),
        ];
        for (keycode, keymod, expected) in table {
            assert_eq!(
                SDLPortModule::process_keypress(keycode, keymod),
                expected,
                "{keycode:?} {keymod:?}"
            );
        }
    }

    #[test]
    fn window_size_steps() {
        assert_eq!(window_size(1, ASPECT_RES), (320, 200));
        assert_eq!(window_size(2, ASPECT_RES), (640, 400));
        assert_eq!(window_size(3, ASPECT_RES), (960, 600));
        // 4:3 aspect makes the window taller, not narrower
        assert_eq!(window_size(1, 4.0 / 3.0), (320, 240));
        assert_eq!(window_size(2, 4.0 / 3.0), (640, 480));
        assert_eq!(window_size(3, 4.0 / 3.0), (960, 720));
    }

    #[test]
    fn render_rect() {
        let table = [
            ((640, 400), ASPECT_RES, Rect::new(0, 0, 640, 400)),
            // Taller window: bars at top and bottom
            ((640, 480), ASPECT_RES, Rect::new(0, 40, 640, 400)),
            // Wider window: bars at left and right
            ((800, 400), ASPECT_RES, Rect::new(80, 0, 640, 400)),
            ((640, 480), 4.0 / 3.0, Rect::new(0, 0, 640, 480)),
            ((640, 400), 4.0 / 3.0, Rect::new(53, 0, 533, 400)),
            // 646 / (4 / 3) = 484.5 rounds to even, as Pascal Round does
            ((646, 600), 4.0 / 3.0, Rect::new(0, 58, 646, 484)),
            ((650, 600), 4.0 / 3.0, Rect::new(0, 56, 650, 488)),
        ];
        for (size, aspect, expected) in table {
            assert_eq!(get_render_rect(size, aspect), expected, "{size:?} {aspect}");
        }
    }

    fn key(c: char) -> Keycode {
        Keycode::from_i32(c as i32).unwrap()
    }
}
