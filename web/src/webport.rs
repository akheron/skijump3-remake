use crate::{AsyncState, Files};
use common::{Platform, TPalette};
use std::cell::{Cell, RefCell};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

pub struct WebPlatform {
    files: Files,
    palette: RefCell<TPalette>,
    screen: RefCell<Vec<u32>>,
    ch: Cell<u8>,
    ch2: Cell<u8>,

    pub async_state: Cell<AsyncState>,
    keypresses: RefCell<Vec<(u8, u8)>>,
}

impl WebPlatform {
    pub fn new(files: Files) -> Self {
        Self {
            files,
            palette: RefCell::new([[0; 3]; 256]),
            screen: RefCell::new(vec![0; 320 * 200]),
            ch: Cell::new(0),
            ch2: Cell::new(0),

            async_state: Cell::new(AsyncState::None),
            keypresses: RefCell::new(vec![]),
        }
    }

    pub fn pixels(&self) -> *const u32 {
        self.screen.borrow().as_ptr()
    }

    pub fn keydown(&self, ch1: u8, ch2: u8) {
        self.keypresses.borrow_mut().push((ch1, ch2));
    }

    fn is_key_pressed(&self) -> bool {
        let mut keypresses = self.keypresses.borrow_mut();
        while keypresses.len() > 0 {
            let keycode = keypresses.remove(0);
            match keycode {
                // TODO: Filter out modifier key presses
                _ => {
                    keypresses.insert(0, keycode);
                    return true;
                }
            }
        }
        false
    }

    fn process_keypress(&self) -> (u8, u8) {
        let mut keypresses = self.keypresses.borrow_mut();
        let (ch, ch2) = keypresses.remove(0);
        self.ch.set(ch);
        self.ch2.set(ch2);
        (ch, ch2)
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

    type WritableFile<'a> = Vec<u8>;

    fn create_file<'a, P: AsRef<str>>(&'a self, path: P) -> Self::WritableFile<'a> {
        todo!()
    }

    type ReadableFile<'a> = &'a [u8];

    fn open_file<'a, P: AsRef<str>>(&'a self, path: P) -> Self::ReadableFile<'a> {
        match path.as_ref() {
            "LANGBASE.SKI" => &self.files.langbase,
            "ANIM.SKI" => &self.files.anim,
            "HISCORE.SKI" => &self.files.hiscore,
            "CONFIG.SKI" => &self.files.config,
            "PLAYERS.SKI" => &self.files.players,
            "NAMES0.SKI" => &self.files.names0,
            "MOREHILL.SKI" => &self.files.morehill,
            "HILLBASE.SKI" => &self.files.hillbase,
            "MAIN.PCX" => &self.files.main_pcx,
            "LOAD.PCX" => &self.files.load_pcx,
            "FRONT1.PCX" => &self.files.front1_pcx,
            "BACK1.PCX" => &self.files.back1_pcx,
            "GOALS.SKI" => &self.files.goals_ski,
            _ => panic!("Unknown file {}", path.as_ref()),
        }
    }

    fn file_exists<P: AsRef<str>>(&self, path: P) -> bool {
        // TODO: support custom files
        false
    }

    fn remove_file<P: AsRef<str>>(&self, path: P) {
        todo!()
    }
}
