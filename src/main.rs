#![feature(async_fn_in_trait)]

mod graph;
mod help;
mod info;
mod lang;
mod list;
mod lumi;
mod maki;
mod pcx;
mod platform;
mod regfree;
mod rs_util;
mod sdlport;
mod sj3;
mod table;
mod tuuli;
mod unit;

use crate::graph::GraphModule;
use crate::info::InfoModule;
use crate::lang::LangModule;
use crate::list::ListModule;
use crate::lumi::LumiModule;
use crate::maki::MakiModule;
use crate::pcx::PcxModule;
use crate::sdlport::{AsyncState, RenderResult, SDLPortModule, X_RES, Y_RES};
use crate::sj3::SJ3Module;
use crate::tuuli::TuuliModule;
use crate::unit::UnitModule;
use futures::task::noop_waker;
use platform::Platform;
use sdl2::event::{Event, WindowEvent};
use std::future::Future;
use std::task::Context;
use std::thread;
use std::time::Duration;

async fn sj3<P: Platform>(port: &P) {
    let maki_module = MakiModule::new();
    let pcx_module = PcxModule::new(&maki_module, port);
    let graph_module = GraphModule::new(&maki_module, &pcx_module, port);

    let mut lang_module = LangModule::new();
    lang_module.init();

    let unit_module = UnitModule::new(&graph_module, &lang_module, &maki_module, &pcx_module, port);
    let list_module =
        ListModule::new(&graph_module, &lang_module, &pcx_module, port, &unit_module).await;
    let tuuli_module = TuuliModule::new(&graph_module);
    let info_module = InfoModule::new(&graph_module, &lang_module, &pcx_module, port, &unit_module);
    let lumi_module = LumiModule::init();
    let mut sj3_module = SJ3Module::new(
        &graph_module,
        &info_module,
        &lang_module,
        lumi_module,
        list_module,
        &maki_module,
        &pcx_module,
        port,
        &tuuli_module,
        &unit_module,
    );

    sj3_module.alku();
    sj3_module.main_menu().await;
}

fn main() {
    let sdl = sdl2::init().unwrap();
    let video_subsystem = sdl.video().unwrap();
    let mut event_pump = sdl.event_pump().unwrap();

    let window_multiplier = 2;
    let window = video_subsystem
        .window(
            "Ski Jump International v3",
            X_RES * window_multiplier,
            Y_RES * window_multiplier,
        )
        .resizable()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();

    let sdl_port_module = SDLPortModule::init(&mut canvas, &texture_creator, window_multiplier);

    // We only have one task, there's no need for a real waker
    let waker = noop_waker();
    let mut ctx = Context::from_waker(&waker);

    let mut future = Box::pin(async {
        sj3(&sdl_port_module).await;
    });
    loop {
        if future.as_mut().poll(&mut ctx).is_ready() {
            break;
        }
        for event in event_pump.poll_iter() {
            match event {
                Event::Window {
                    win_event: WindowEvent::Resized(..),
                    ..
                } => {
                    sdl_port_module.handle_resized();
                }
                Event::Quit { .. } => {
                    return;
                }
                Event::KeyDown {
                    keycode: Some(keycode),
                    keymod,
                    ..
                } => {
                    sdl_port_module.handle_keydown(keycode, keymod);
                }
                _ => {}
            }
        }
        sdl_port_module.main_loop();
    }
}
