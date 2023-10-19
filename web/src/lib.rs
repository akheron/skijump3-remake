mod webport;

use crate::webport::WebPlatform;
use common::sj3;
use futures::task::noop_waker;
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
        let platform = Rc::new(WebPlatform::new(Files::new(file_data)));
        let platform2 = platform.clone();
        let future = Box::pin(async move {
            sj3(platform2.as_ref()).await;
        });
        let waker = noop_waker();
        Self {
            platform,
            future,
            waker,
        }
    }

    pub fn tick(&mut self) -> Option<AsyncState> {
        let mut ctx = Context::from_waker(&self.waker);
        if self.future.as_mut().poll(&mut ctx).is_ready() {
            None
        } else {
            Some(self.platform.async_state.get())
        }
    }

    pub fn resume(&self) {
        self.platform.async_state.set(AsyncState::None);
    }

    pub fn screen(&self) -> *const u32 {
        self.platform.pixels()
    }

    pub fn keydown(&self, key: &str) {
        if let Some((ch, ch2)) = match key {
            "ArrowUp" => Some((0, 72)),
            "ArrowLeft" => Some((0, 75)),
            "ArrowRight" => Some((0, 77)),
            "ArrowDown" => Some((0, 80)),
            "Enter" => Some((13, 0)),
            "Escape" => Some((27, 0)),
            " " => Some((32, 0)),
            "R" => Some((82, 19)),
            "T" => Some((84, 20)),
            "r" => Some((114, 19)),
            "t" => Some((116, 20)),
            _ => None,
        } {
            self.platform.keydown(ch, ch2);
        }
    }
}
