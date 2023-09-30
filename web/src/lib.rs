#![feature(async_fn_in_trait)]

mod webport;

use crate::webport::WebPlatform;
use common::sj3;
use futures::task::noop_waker;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Waker};
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub struct SJ3 {
    platform: Rc<WebPlatform>,
    future: Pin<Box<dyn Future<Output = ()>>>,
    waker: Waker,
}

#[wasm_bindgen]
pub struct Files {
    langbase: Vec<u8>,
    anim: Vec<u8>,
    hiscore: Vec<u8>,
    config: Vec<u8>,
    players: Vec<u8>,
    names0: Vec<u8>,
    morehill: Vec<u8>,
    hillbase: Vec<u8>,
    main_pcx: Vec<u8>,
    load_pcx: Vec<u8>,
    front1_pcx: Vec<u8>,
    back1_pcx: Vec<u8>,
    goals_ski: Vec<u8>,
}

#[wasm_bindgen]
impl Files {
    pub fn new(
        langbase: &[u8],
        anim: &[u8],
        hiscore: &[u8],
        config: &[u8],
        players: &[u8],
        names0: &[u8],
        morehill: &[u8],
        hillbase: &[u8],
        main_pcx: &[u8],
        load_pcx: &[u8],
        front1_pcx: &[u8],
        back1_pcx: &[u8],
        goals_ski: &[u8],
    ) -> Self {
        Self {
            langbase: langbase.to_vec(),
            anim: anim.to_vec(),
            hiscore: hiscore.to_vec(),
            config: config.to_vec(),
            players: players.to_vec(),
            names0: names0.to_vec(),
            morehill: morehill.to_vec(),
            hillbase: hillbase.to_vec(),
            main_pcx: main_pcx.to_vec(),
            load_pcx: load_pcx.to_vec(),
            front1_pcx: front1_pcx.to_vec(),
            back1_pcx: back1_pcx.to_vec(),
            goals_ski: goals_ski.to_vec(),
        }
    }
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
    pub fn new(files: Files) -> Self {
        console_error_panic_hook::set_once();
        let platform = Rc::new(WebPlatform::new(files));
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
