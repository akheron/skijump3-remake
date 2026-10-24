use crate::{AsyncState, Files};
use common::files::{MemoryStore, Overlay, OverlayFile};
use common::{Platform, TPalette};
use std::cell::{Cell, RefCell};
use std::future::Future;
use std::io::Cursor;
use std::pin::Pin;
use std::task::{Context, Poll};

pub struct WebPlatform {
    files: Overlay<MemoryStore, Files>,
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
            files: Overlay::new(MemoryStore::default(), files),
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

    type WritableFile<'a> = OverlayFile<'a, MemoryStore, Files>;

    fn create_file<'a, P: AsRef<str>>(&'a self, path: P) -> Self::WritableFile<'a> {
        self.files.create(path.as_ref())
    }

    type ReadableFile<'a> = Cursor<Vec<u8>>;

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
}
