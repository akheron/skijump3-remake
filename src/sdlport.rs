use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::{Keycode, Mod};
use sdl2::pixels::PixelFormatEnum::RGBA8888;
use sdl2::pixels::{Color, Palette, PixelFormatEnum};
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::surface::Surface;
use sdl2::video::{Window, WindowContext};
use sdl2::{EventPump, EventSubsystem};
use std::cell::{Cell, RefCell};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};
use std::{process, thread};

pub type TPalette = [[u8; 3]; 256];

pub const X_RES: u32 = 320;
pub const Y_RES: u32 = 200;
const TARGET_FRAMES: u32 = 70;
const ASPECT_RES: f64 = X_RES as f64 / Y_RES as f64;

pub struct SDLPortModule<'a> {
    window_multiplier: u32,
    canvas: RefCell<&'a mut Canvas<Window>>,
    texture_creator: &'a TextureCreator<WindowContext>,
    original_surface: RefCell<Surface<'static>>,
    display_texture: RefCell<Texture<'a>>,
    palette: [Color; 256],
    render_dest_rect: Cell<Rect>,

    window_resized: Cell<bool>,
    full_screen: bool,
    aspect: f64,

    frame_count: Cell<u32>,
    sub_frame_duration: Cell<Duration>,
    last_frame_instant: Cell<Instant>,

    event_subsystem: RefCell<EventSubsystem>,
    event_pump: RefCell<EventPump>,

    sw_rendering: bool,

    // Originally in SJ3HELP.PAS
    pub ch: Cell<u8>,
    pub ch2: Cell<u8>,

    // Async state
    frame_requested: Cell<bool>,
}

impl<'a> SDLPortModule<'a> {
    pub fn init(
        canvas: &'a mut Canvas<Window>,
        texture_creator: &'a TextureCreator<WindowContext>,
        event_subsystem: EventSubsystem,
        event_pump: EventPump,
        window_multiplier: u32,
    ) -> Self {
        // let renderer_flags = 0;
        let aspect = ASPECT_RES;
        let sw_rendering = false;

        // if (sw_rendering = true) then
        // begin
        //   renderer_flags = renderer_flags
        //       OR SDL_RENDERER_SOFTWARE;
        // end;
        //
        // renderer = SDL_CreateRenderer(window, -1, renderer_flags);
        // if renderer = nil then Halt;

        // 8-bit surface for original data
        let original_surface = Surface::new(X_RES, Y_RES, PixelFormatEnum::Index8).unwrap();

        // Finally a texture for displaying 32-bit display data
        let display_texture = texture_creator
            .create_texture_streaming(RGBA8888, X_RES, Y_RES)
            .unwrap();

        let render_dest_rect = get_render_rect(canvas, aspect);
        SDLPortModule {
            window_multiplier,
            canvas: RefCell::new(canvas),
            texture_creator,
            original_surface: RefCell::new(original_surface),
            display_texture: RefCell::new(display_texture),
            palette: [Color::RGB(0, 0, 0); 256],
            render_dest_rect: Cell::new(render_dest_rect),

            window_resized: Cell::new(false),
            full_screen: false,
            aspect,

            frame_count: Cell::new(0),
            sub_frame_duration: Cell::new(Duration::new(0, 0)),
            last_frame_instant: Cell::new(Instant::now()),

            event_subsystem: RefCell::new(event_subsystem),
            event_pump: RefCell::new(event_pump),
            sw_rendering,

            ch: Cell::new(1),
            ch2: Cell::new(1),

            // frame_requested: RefCell::new(None),
            frame_requested: Cell::new(false),
        }
    }

    pub fn deinit_graphics(&self) {
        unimplemented!()
    }

    pub fn deinit(&self) {
        unimplemented!()
    }

    pub fn set_palette(&self, input: &TPalette) {
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

    pub fn render_phase1(&self, buffer: &[u8]) {
        let mut original_surface = self.original_surface.borrow_mut();
        original_surface.with_lock_mut(|pixels| {
            pixels.copy_from_slice(buffer);
        });
    }

    pub async fn render_phase2(&self) {
        self.frame_requested.replace(true);
        RenderFuture { s: self }.await
    }

    pub fn render_phase3(&self) {
        if !self.frame_requested.get() {
            return;
        }
        self.frame_requested.set(false);

        self.wait_raster();

        if self.window_resized.replace(false) {
            self.render_dest_rect
                .set(get_render_rect(&self.canvas.borrow(), self.aspect));
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

    pub fn wait_raster(&self) {
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

    pub fn key_pressed(&self) -> bool {
        let mut pressed = false;

        for event in self.event_pump.borrow_mut().poll_iter() {
            match event {
                Event::Window {
                    win_event: WindowEvent::Resized(..),
                    ..
                } => {
                    self.window_resized.set(true);
                }
                Event::Quit { .. } => {
                    process::exit(0);
                    // if let Some(cb) = &self.close_callback {
                    //     cb();
                    // }
                }
                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => {
                    match keycode {
                        // Ignore modifier and status keys, since they did not trigger a keypress in DOS version.
                        // F11, F12 and probably others were ignored too, but that would probably be counterintuitive.
                        Keycode::LCtrl
                        | Keycode::RCtrl
                        | Keycode::LShift
                        | Keycode::RShift
                        | Keycode::LAlt
                        | Keycode::RAlt
                        | Keycode::LGui
                        | Keycode::RGui
                        | Keycode::CapsLock
                        | Keycode::ScrollLock
                        | Keycode::NumLockClear
                        | Keycode::PrintScreen
                        | Keycode::Sysreq
                        | Keycode::Pause => {}
                        _ => {
                            pressed = true;
                            self.event_subsystem.borrow_mut().push_event(event).unwrap();
                        }
                    }
                }
                _ => {}
            }
        }
        pressed
    }

    fn wait_for_key_press_internal(&self) -> (u8, u8) {
        // Setting ch2 to a correct scancode value for the key pressed can be largely ignored.
        // It is only checked by the game for special keys, where ch1 is checked or assumed to be 0.
        // This also means that the game cannot differentiate between different keys producing the same character.
        let mut ch1 = 0u8;
        let mut ch2 = 0u8;

        loop {
            for event in self.event_pump.borrow_mut().poll_iter() {
                if let Event::KeyDown {
                    //scancode,
                    keycode: Some(mut keycode),
                    keymod,
                    ..
                } = event
                {
                    /*
                    // SDL version specific shortcuts
                    if ((keyMod and KMOD_LALT) > 0) then
                    begin
                      Case scancode of
                        SDL_SCANCODE_RETURN :
                          begin
                            toggleFullscreen;
                            exit
                          end;
                        SDL_SCANCODE_R :
                          begin
                            ResetWindowSize;
                            exit
                          end;
                        SDL_SCANCODE_KP_PLUS, SDL_SCANCODE_EQUALS :
                          begin
                            windowMultiplier := windowMultiplier + 1;
                            ResetWindowSize;
                            exit
                          end;
                        SDL_SCANCODE_KP_MINUS, SDL_SCANCODE_MINUS :
                          begin
                            if (windowMultiplier > 1) then
                            begin
                              windowMultiplier := windowMultiplier - 1;
                              ResetWindowSize;
                              exit
                            end;
                          end;
                        SDL_SCANCODE_A :
                          begin
                            if (aspect <> aspectRes) then
                              aspect := aspectRes
                            else
                              aspect := 4 / 3;
                            ResetWindowSize;
                            exit;
                          end;
                      end;
                    end;
                                        */
                    // Special cases for key combinations used throughout the game.
                    // Check for Right Alt/AltGr first, since pressing it can make Left Ctrl look as pressed too.
                    if keymod.contains(Mod::MODEMOD | Mod::RALTMOD) {
                        return (0, 0);
                    }
                    if keymod.contains(Mod::LALTMOD) {
                        return match keycode {
                            // ALT-X
                            Keycode::X => (0, 45),
                            _ => (0, 0),
                        };
                    }
                    if keymod.contains(Mod::LCTRLMOD) {
                        return match keycode {
                            // CTRL-C
                            Keycode::C => (3, 0),
                            _ => (0, 0),
                        };
                    }
                    if keymod.contains(Mod::LGUIMOD | Mod::RGUIMOD) {
                        return (0, 0);
                    }

                    // Handle conversion of letters to uppercase.
                    // Convert only if Caps Lock is off and Shift is pressed or vice versa.
                    keycode = match keycode as i32 {
                        97..=122 | 224..=246 | 248..=254
                            if keymod.contains(Mod::LSHIFTMOD | Mod::RSHIFTMOD)
                                ^ keymod.contains(Mod::CAPSMOD) =>
                        {
                            Keycode::from_i32(keycode as i32 - 32).unwrap()
                        }
                        _ => keycode,
                    };
                    /*
                    // If modifier is Shift, convert the key pressed accordingly.
                    // For special keys, standard US QWERTY layout is assumed.
                    if ((keyMod and KMOD_SHIFT) > 0) then
                    begin
                      Case keyPressed of
                        65..90, 97..122, 190..214, 216..222, 224..246, 248..254: ; // Already handled above
                        SDLK_1 : keyPressed:=SDLK_EXCLAIM;
                        SDLK_2 : keyPressed:=SDLK_AT;
                        SDLK_3 : keyPressed:=SDLK_HASH;
                        SDLK_4 : keyPressed:=SDLK_DOLLAR;
                        SDLK_5 : keyPressed:=SDLK_PERCENT;
                        SDLK_6 : keyPressed:=SDLK_CARET;
                        SDLK_7 : keyPressed:=SDLK_AMPERSAND;
                        SDLK_8 : keyPressed:=SDLK_ASTERISK;
                        SDLK_9 : keyPressed:=SDLK_LEFTPAREN;
                        SDLK_0 : keyPressed:=SDLK_RIGHTPAREN;
                        SDLK_MINUS : keyPressed:=SDLK_UNDERSCORE;
                        SDLK_EQUALS : keyPressed:=SDLK_PLUS;
                        SDLK_LEFTBRACKET : keyPressed:=TSDL_KeyCode('{');
                        SDLK_RIGHTBRACKET : keyPressed:=TSDL_KeyCode('}');
                        SDLK_SEMICOLON : keyPressed:=SDLK_COLON;
                        {SDLK_QUOTE} TSDL_KeyCode('''') : keyPressed:=SDLK_QUOTEDBL; // bug in ev1313/Pascal-SDL-2-Headers - wrong value of SDLK_QUOTE
                        SDLK_BACKQUOTE : keyPressed:=TSDL_KeyCode('~');
                        SDLK_BACKSLASH : keyPressed:=TSDL_KeyCode('|');
                        SDLK_COMMA : keyPressed:=SDLK_LESS;
                        SDLK_PERIOD : keyPressed:=SDLK_GREATER;
                        SDLK_SLASH : keyPressed:=SDLK_QUESTION;
                      else
                        exit
                      end;
                    end;

                    // If NumLock modifier is not set, convert the charaters accordingly
                    if ((keyMod and KMOD_NUM) = 0) then
                    begin
                      Case keyPressed of
                        SDLK_KP_1 : keyPressed:=SDLK_END;
                        SDLK_KP_2 : keyPressed:=SDLK_DOWN;
                        SDLK_KP_3 : keyPressed:=SDLK_PAGEDOWN;
                        SDLK_KP_4 : keyPressed:=SDLK_LEFT;
                        SDLK_KP_5 : begin ch1:=#0; ch2:=#76; exit end;
                        SDLK_KP_6 : keyPressed:=SDLK_RIGHT;
                        SDLK_KP_7 : keyPressed:=SDLK_HOME;
                        SDLK_KP_8 : keyPressed:=SDLK_UP;
                        SDLK_KP_9 : keyPressed:=SDLK_PAGEUP;
                        SDLK_KP_0 : keyPressed:=SDLK_INSERT;
                        SDLK_KP_PERIOD : keyPressed:=SDLK_DELETE;
                      end;
                    end;

                    // Merge keypad characters with their regular counterparts.
                    // It is not needed to differentiate between them, since the scancode isn't checked for normal characters.
                    // Checks for Shift and Num Lock were already done, so they won't interfere with the merge.
                    Case keyPressed of
                      SDLK_KP_DIVIDE : keyPressed:=SDLK_SLASH;
                      SDLK_KP_MULTIPLY : keyPressed:=SDLK_ASTERISK;
                      SDLK_KP_MINUS : keyPressed:=SDLK_MINUS;
                      SDLK_KP_PLUS : keyPressed:=SDLK_PLUS;
                      SDLK_KP_ENTER : keyPressed:=SDLK_RETURN;
                      SDLK_KP_1 : keyPressed:=SDLK_1;
                      SDLK_KP_2 : keyPressed:=SDLK_2;
                      SDLK_KP_3 : keyPressed:=SDLK_3;
                      SDLK_KP_4 : keyPressed:=SDLK_4;
                      SDLK_KP_5 : keyPressed:=SDLK_5;
                      SDLK_KP_6 : keyPressed:=SDLK_6;
                      SDLK_KP_7 : keyPressed:=SDLK_7;
                      SDLK_KP_8 : keyPressed:=SDLK_8;
                      SDLK_KP_9 : keyPressed:=SDLK_9;
                      SDLK_KP_0 : keyPressed:=SDLK_0;
                      SDLK_KP_PERIOD : keyPressed:=SDLK_PERIOD;
                      SDLK_KP_EQUALS : keyPressed:=SDLK_EQUALS;
                    end;
                    */
                    match keycode {
                        Keycode::Return => ch1 = 13,
                        Keycode::Escape => ch1 = 27,
                        Keycode::Backspace => ch1 = 8,
                        Keycode::Tab => ch1 = 9,
                        _ => {}
                    }
                    match keycode as i32 {
                        32..=126 => ch1 = (keycode as u8).to_ascii_uppercase(),

                        // Special characters supported by the game, OEM 865 nordic encoding is used for ch1 values
                        196 => ch1 = 142, // A with diaeresis
                        197 => ch1 = 143, // A with ring above
                        198 => ch1 = 146, // AE
                        214 => ch1 = 153, // O with diaeresis
                        216 => ch1 = 157, // O with stroke
                        220 => ch1 = 154, // U with diaeresis
                        223 => ch1 = 225, // sharp s

                        // a with diaeresis
                        228 => {
                            ch1 = 132;
                            ch2 = 36;
                        }
                        // a with ring above
                        229 => {
                            ch1 = 134;
                            ch2 = 26;
                        }
                        230 => ch1 = 145, // ae
                        // o with diaeresis
                        246 => {
                            ch1 = 148;
                            ch2 = 36;
                        }
                        248 => ch1 = 156, // o with stroke
                        // u with diaeresis
                        252 => {
                            ch1 = 153;
                            ch2 = 36;
                        }
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

                        Keycode::Left => ch2 = 75,
                        Keycode::Right => ch2 = 77,
                        Keycode::Up => ch2 = 72,
                        Keycode::Down => ch2 = 80,

                        Keycode::Insert => ch2 = 82,
                        Keycode::Delete => ch2 = 83,
                        Keycode::Home => ch2 = 71,
                        Keycode::End => ch2 = 79,
                        Keycode::PageUp => ch2 = 73,
                        Keycode::PageDown => ch2 = 81,

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

                    return (ch1, ch2);
                }
            }
            thread::sleep(Duration::from_millis(10));
        }
    }

    pub fn wait_for_key_press(&self) -> (u8, u8) {
        let (ch, ch2) = self.wait_for_key_press_internal();
        self.ch.set(ch);
        self.ch2.set(ch2);
        (ch, ch2)
    }

    pub fn wait(&self, ms: u32) {
        thread::sleep(Duration::from_millis(ms as u64));
    }

    // Originally in SJ3UNIT.PAS
    pub fn kword(&self) -> u16 {
        ((self.ch.get() as u16) << 8) + self.ch2.get() as u16
    }

    // Everything down from here originally in SJ3HELP.PAS
    pub fn putsaa(&self) {
        while self.key_pressed() {
            self.wait_for_key_press();
        }
    }

    pub fn clearchs(&self) {
        self.ch.set(1);
        self.ch2.set(1);
    }

    pub fn wait_for_key(&self) {
        self.putsaa();
        self.wait_for_key_press();
    }

    pub fn wait_for_key2(&self) -> bool {
        self.putsaa();
        let (ch, ch2) = self.wait_for_key_press();
        ch == 0 && ch2 == 68
    }
}

fn get_render_rect(canvas: &Canvas<Window>, aspect: f64) -> Rect {
    let (window_w, window_h) = canvas.output_size().unwrap();
    if window_w as f64 / window_h as f64 <= aspect {
        let h = f64::round(window_w as f64 / aspect) as u32;
        Rect::new(0, ((window_h - h) / 2) as i32, window_w, h)
    } else {
        let w = f64::round(window_h as f64 * aspect) as u32;
        Rect::new(((window_w - w) / 2) as i32, 0, w, window_h)
    }
}

struct RenderFuture<'s, 'si> {
    s: &'s SDLPortModule<'si>,
}

impl<'s, 'si> Future for RenderFuture<'s, 'si> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        if !self.s.frame_requested.get() {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}
