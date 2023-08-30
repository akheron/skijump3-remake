#![feature(async_fn_in_trait)]

mod sdlport;

use crate::sdlport::{SDLPortModule, X_RES, Y_RES};
use common::sj3;
use futures::task::noop_waker;
use futures::Future;
use sdl2::event::{Event, WindowEvent};
use std::task::Context;

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
