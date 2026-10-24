mod storage;
mod webport;

use crate::storage::LocalStorageStore;
use crate::webport::{Modifiers, WebPlatform};
use common::files::Store;
use common::sj3;
use futures::task::noop_waker;
use std::borrow::Cow;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Waker};
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
    #[wasm_bindgen(js_namespace = console)]
    pub fn warn(s: &str);
}

#[wasm_bindgen]
pub struct SJ3 {
    platform: Rc<WebPlatform>,
    future: Pin<Box<dyn Future<Output = ()>>>,
    waker: Waker,
}

pub struct Files {
    files: HashMap<String, (u32, u32)>,
    data: Vec<u8>,
}

impl Files {
    pub fn new(mut pack_data: &[u8]) -> Self {
        let mut files: HashMap<String, (u32, u32)> = HashMap::new();

        // See the `pack` crate for file format details
        let num_files = read_le_u8(&mut pack_data);
        for i in 0..num_files {
            let name_len = read_le_u8(&mut pack_data);
            let (name, _) = pack_data.split_at(name_len as usize);
            pack_data = &pack_data[name_len as usize..];
            let name = String::from_utf8(name.to_vec()).unwrap();
            let size = read_le_u32(&mut pack_data);
            let offset = read_le_u32(&mut pack_data);
            files.insert(name, (size, offset));
        }

        Self {
            files,
            data: pack_data.to_vec(),
        }
    }

    pub fn file_content<'a>(&'a self, name: &str) -> Option<&'a [u8]> {
        self.files.get(name).map(|(size, offset)| {
            let size = *size as usize;
            let offset = *offset as usize;
            &self.data[offset..offset + size]
        })
    }
}

impl Store for Files {
    fn read(&self, name: &str) -> Option<Cow<'_, [u8]>> {
        self.file_content(name).map(Cow::Borrowed)
    }
    fn contains(&self, name: &str) -> bool {
        self.files.contains_key(name)
    }
    fn names(&self) -> Vec<String> {
        self.files.keys().cloned().collect()
    }
}

fn read_le_u8(input: &mut &[u8]) -> u8 {
    let (int_bytes, rest) = input.split_at(std::mem::size_of::<u8>());
    *input = rest;
    u8::from_le_bytes(int_bytes.try_into().unwrap())
}

fn read_le_u32(input: &mut &[u8]) -> u32 {
    let (int_bytes, rest) = input.split_at(std::mem::size_of::<u32>());
    *input = rest;
    u32::from_le_bytes(int_bytes.try_into().unwrap())
}

fn start(platform: Rc<WebPlatform>) -> Pin<Box<dyn Future<Output = ()>>> {
    Box::pin(async move {
        sj3(platform.as_ref()).await;
    })
}

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AsyncState {
    None,
    Render,
    WaitForKeyPress,
    KeyPressed,
}

#[wasm_bindgen]
impl SJ3 {
    pub fn new(file_data: &[u8]) -> Self {
        console_error_panic_hook::set_once();
        let platform = Rc::new(WebPlatform::new(
            LocalStorageStore::new(),
            Files::new(file_data),
        ));
        let future = start(platform.clone());
        let waker = noop_waker();
        Self {
            platform,
            future,
            waker,
        }
    }

    /// When the game ends, it starts again.
    pub fn tick(&mut self) -> AsyncState {
        let waker = self.waker.clone();
        let mut ctx = Context::from_waker(&waker);
        while self.future.as_mut().poll(&mut ctx).is_ready() {
            self.restart();
        }
        self.platform.async_state.get()
    }

    /// Saves the files as the game does when its window is closed, and
    /// starts the game again.
    pub fn close(&mut self) {
        self.platform.request_close();
        let waker = self.waker.clone();
        let mut ctx = Context::from_waker(&waker);
        if self.future.as_mut().poll(&mut ctx).is_ready() {
            self.restart();
        } else {
            log("The game did not stop when it was closed");
        }
    }

    fn restart(&mut self) {
        self.platform.reset();
        self.future = start(self.platform.clone());
    }

    pub fn resume(&self) {
        self.platform.async_state.set(AsyncState::None);
    }

    pub fn screen(&self) -> *const u32 {
        self.platform.pixels()
    }

    /// Returns whether the game uses the key.
    pub fn keydown(
        &self,
        key: &str,
        code: &str,
        shift: bool,
        ctrl: bool,
        alt: bool,
        alt_graph: bool,
        meta: bool,
        caps_lock: bool,
    ) -> bool {
        let modifiers = Modifiers {
            shift,
            ctrl,
            alt,
            alt_graph,
            meta,
            caps_lock,
        };
        self.platform.keydown(key, code, modifiers)
    }

    pub fn keyup(&self, code: &str) {
        self.platform.keyup(code);
    }

    pub fn set_unshifted_key(&self, code: &str, key: &str) {
        self.platform.set_unshifted_key(code, key);
    }
}
