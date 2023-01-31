use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::{Keycode, Mod};
use sdl2::pixels::PixelFormatEnum::RGBA8888;
use sdl2::pixels::{Color, Palette, PixelFormatEnum};
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::surface::Surface;
use sdl2::timer::Timer;
use sdl2::video::{Window, WindowContext};
use sdl2::{EventPump, EventSubsystem, TimerSubsystem};
use std::cell::RefCell;
use std::time::Duration;
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
    render_dest_rect: RefCell<Rect>,

    window_resized: RefCell<bool>,
    full_screen: bool,
    aspect: f64,

    timer: Timer<'a, 'a>,
    frame_count: u32,
    last_frame_count: u32,
    sub_frame_count: u32,
    last_frame_tick: u32,

    event_subsystem: RefCell<EventSubsystem>,
    event_pump: RefCell<EventPump>,

    sw_rendering: bool,
}

impl<'a> SDLPortModule<'a> {
    pub fn init(
        canvas: &'a mut Canvas<Window>,
        texture_creator: &'a TextureCreator<WindowContext>,
        timer_subsystem: &'a TimerSubsystem,
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
            .create_texture_streaming(PixelFormatEnum::RGBA8888, X_RES, Y_RES)
            .unwrap();

        let timer = timer_subsystem.add_timer(1, /*timer_callback*/ Box::new(|| 0));

        let render_dest_rect = get_render_rect(canvas, aspect);
        SDLPortModule {
            window_multiplier,
            canvas: RefCell::new(canvas),
            texture_creator,
            original_surface: RefCell::new(original_surface),
            display_texture: RefCell::new(display_texture),
            palette: [Color::RGB(0, 0, 0); 256],
            render_dest_rect: RefCell::new(render_dest_rect),

            window_resized: RefCell::new(false),
            full_screen: false,
            aspect,

            timer,
            frame_count: 0,
            last_frame_count: 0,
            sub_frame_count: 0,
            last_frame_tick: timer_subsystem.ticks(),

            event_subsystem: RefCell::new(event_subsystem),
            event_pump: RefCell::new(event_pump),
            sw_rendering,
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

    pub fn render(&self, buffer: &[u8]) {
        if *self.window_resized.borrow() {
            *self.render_dest_rect.borrow_mut() =
                get_render_rect(&self.canvas.borrow(), self.aspect);
            *self.window_resized.borrow_mut() = false;
        }

        let mut canvas = self.canvas.borrow_mut();
        canvas.clear();

        // Actual rendering
        let mut original_surface = self.original_surface.borrow_mut();
        original_surface.with_lock_mut(|pixels| {
            pixels.copy_from_slice(buffer);
        });

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
        // SDL_LockTexture(displayTexture, nil, @pixels, @pitch);
        //
        // SDL_ConvertPixels(displaySurface^.w, displaySurface^.h,
        //                   displaySurface^.format^.format,
        //                   displaySurface^.pixels, displaySurface^.pitch,
        //                   SDL_PIXELFORMAT_RGBA8888,
        //                   pixels, pitch);
        //
        // SDL_UnlockTexture(displayTexture);

        // Render texture to display
        canvas
            .copy(&display_texture, None, *self.render_dest_rect.borrow())
            .unwrap();
        canvas.present();
    }

    pub fn wait_raster(&self) {
        // unimplemented!()
    }

    pub fn key_pressed(&self) -> bool {
        let mut pressed = false;

        for event in self.event_pump.borrow_mut().poll_iter() {
            match event {
                Event::Window {
                    win_event: WindowEvent::Resized(..),
                    ..
                } => {
                    *self.window_resized.borrow_mut() = true;
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

    pub fn wait_for_key_press(&self) -> (u8, u8) {
        /*
        procedure WaitForKeyPress(var ch1, ch2:char);
        var event : TSDL_Event;
            scancode: TSDL_ScanCode;
            keyPressed: TSDL_KeyCode;
            keyMod: UInt16;
        begin
        */
        // Setting ch2 to a correct scancode value for the key pressed can be largely ignored.
        // It is only checked by the game for special keys, where ch1 is checked or assumed to be 0.
        // This also means that the game cannot differentiate between different keys producing the same character.
        let mut ch1 = 0u8;
        let mut ch2 = 0u8;

        loop {
            for event in self.event_pump.borrow_mut().poll_iter() {
                match event {
                    Event::KeyDown {
                        //scancode,
                        keycode,
                        keymod,
                        ..
                    } => {
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
                        if keymod.contains(Mod::MODEMOD) || keymod.contains(Mod::RALTMOD) {
                            return (0, 0);
                        }
                        if keymod.contains(Mod::LALTMOD) {
                            return match keycode {
                                // ALT-X
                                Some(Keycode::X) => (0, 45),
                                _ => (0, 0),
                            };
                        }
                        if keymod.contains(Mod::LCTRLMOD) {
                            return match keycode {
                                // CTRL-C
                                Some(Keycode::C) => (3, 0),
                                _ => (0, 0),
                            };
                        }
                        if keymod.contains(Mod::LGUIMOD) || keymod.contains(Mod::RGUIMOD) {
                            return (0, 0);
                        }

                        /*
                        // Handle conversion of letters to uppercase.
                        // Convert only if Caps Lock is off and Shift is pressed or vice versa.
                        if (
                            ((keyMod and KMOD_SHIFT) > 0) xor ((keyMod and KMOD_CAPS) > 0)
                        ) then
                        begin
                          Case keypressed of
                            97..122, 224..246, 248..254: keyPressed:=TSDL_KeyCode(keyPressed - 32);
                          end;
                        end;

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
                        if let Some(keycode) = keycode {
                            match keycode {
                                Keycode::Return => ch1 = 13,
                                Keycode::Escape => ch1 = 27,
                                Keycode::Backspace => ch1 = 8,
                                Keycode::Tab => ch1 = 9,
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

                                _ => {}
                            }
                        }
                        return (ch1, ch2);
                        /*

                        Case keyPressed of
                          SDLK_RETURN : ch1:=#13;
                          SDLK_ESCAPE : ch1:=#27;
                          SDLK_BACKSPACE : ch1:=#8;
                          SDLK_TAB : ch1:=#9;

                          32..126 : ch1:=chr(keyPressed);

                          // Special characters supported by the game, OEM 865 nordic encoding is used for ch1 values
                          196 : begin ch1:=#142; exit end; // A with diaeresis
                          197 : begin ch1:=#143; exit end; // A with ring above
                          198 : begin ch1:=#146; exit end; // AE
                          214 : begin ch1:=#153; exit end; // O with diaeresis
                          216 : begin ch1:=#157; exit end; // O with stroke
                          220 : begin ch1:=#154; exit end; // U with diaeresis
                          223 : begin ch1:=#225; exit end; // sharp s

                          228 : begin ch1:=#132; ch2:=#36; exit end; // a with diaeresis
                          229 : begin ch1:=#134; ch2:=#26; exit end; // a with ring above
                          230 : begin ch1:=#145; exit end; // ae
                          246 : begin ch1:=#148; ch2:=#39; exit end; // o with diaeresis
                          248 : begin ch1:=#158; exit end; // o with stroke
                          252 : begin ch1:=#129; exit end; // u with diaeresis
                        end;

                        Case keyPressed of
                          SDLK_F1 : ch2:=#59;
                          SDLK_F2 : ch2:=#60;
                          SDLK_F3 : ch2:=#61;
                          SDLK_F4 : ch2:=#62;
                          SDLK_F5 : ch2:=#63;
                          SDLK_F6 : ch2:=#64;
                          SDLK_F7 : ch2:=#65;
                          SDLK_F8 : ch2:=#66;
                          SDLK_F9 : ch2:=#67;
                          SDLK_F10 : ch2:=#68;

                          SDLK_LEFT : ch2:=#75;
                          SDLK_RIGHT : ch2:=#77;
                          SDLK_UP : ch2:=#72;
                          SDLK_DOWN : ch2:=#80;

                          SDLK_INSERT : ch2:=#82;
                          SDLK_DELETE : ch2:=#83;
                          SDLK_HOME : ch2:=#71;
                          SDLK_END : ch2:=#79;
                          SDLK_PAGEUP : ch2:=#73;
                          SDLK_PAGEDOWN : ch2:=#81;

                          SDLK_a : ch2:=#30;
                          SDLK_b : ch2:=#48;
                          SDLK_c : ch2:=#46;
                          SDLK_d : ch2:=#32;
                          SDLK_e : ch2:=#18;
                          SDLK_f : ch2:=#33;
                          SDLK_g : ch2:=#34;
                          SDLK_h : ch2:=#35;
                          SDLK_i : ch2:=#23;
                          SDLK_j : ch2:=#36;
                          SDLK_k : ch2:=#37;
                          SDLK_l : ch2:=#38;
                          SDLK_m : ch2:=#50;
                          SDLK_n : ch2:=#49;
                          SDLK_o : ch2:=#24;
                          SDLK_p : ch2:=#25;
                          SDLK_q : ch2:=#16;
                          SDLK_r : ch2:=#19;
                          SDLK_s : ch2:=#31;
                          SDLK_t : ch2:=#20;
                          SDLK_u : ch2:=#22;
                          SDLK_v : ch2:=#47;
                          SDLK_w : ch2:=#17;
                          SDLK_x : ch2:=#45;
                          SDLK_y : ch2:=#21;
                          SDLK_z : ch2:=#44;
                        end;
                        exit;
                        end;
                        end;
                        SDL_Delay(10);
                        end;
                        end;
                                            */
                    }
                    _ => {}
                }
            }
            thread::sleep(Duration::from_millis(10));
        }
    }

    pub fn wait(&self, ms: u32) {
        unimplemented!()
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
