mod sdlport;

use crate::sdlport::{get_arguments, SDLPortModule, X_RES, Y_RES};
use common::sj3;
use futures::task::noop_waker;
use futures::Future;
use sdl2::event::{Event, WindowEvent};
use std::path::PathBuf;
use std::task::Context;

fn main() {
    #[cfg(feature = "trace")]
    if let Ok(trace_path) = std::env::var("SJ3_TRACE") {
        common::trace::load(&trace_path, std::env::var("SJ3_FRAMES").ok());
    }

    let sw_rendering = get_arguments();

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

    let mut canvas_builder = window.into_canvas();
    if sw_rendering {
        canvas_builder = canvas_builder.software();
    }
    let mut canvas = canvas_builder.build().unwrap();
    let texture_creator = canvas.texture_creator();

    let data_dir = match std::env::var_os("SJ3_DATA_DIR") {
        Some(dir) => PathBuf::from(dir),
        None => PathBuf::from(sdl2::filesystem::pref_path("", "skijump3-remake").unwrap()),
    };
    std::fs::create_dir_all(&data_dir).unwrap();

    let sdl_port_module =
        SDLPortModule::init(&mut canvas, &texture_creator, window_multiplier, data_dir);

    // We only have one task, there's no need for a real waker
    let waker = noop_waker();
    let mut ctx = Context::from_waker(&waker);

    let mut future = Box::pin(async {
        sj3(&sdl_port_module).await;
    });
    loop {
        if future.as_mut().poll(&mut ctx).is_ready() {
            if common::trace::active() {
                common::trace::finish();
            }
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
                    sdl_port_module.handle_quit();
                }
                Event::KeyDown {
                    keycode,
                    scancode,
                    keymod,
                    ..
                } => {
                    sdl_port_module.handle_keydown(keycode, scancode, keymod);
                }
                _ => {}
            }
        }
        sdl_port_module.main_loop();
    }
}
