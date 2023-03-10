mod graph;
mod help;
mod info;
mod lang;
mod lumi;
mod maki;
mod pcx;
mod regfree;
mod rs_util;
mod sdlport;
mod sj3;
mod tuuli;
mod unit;

use crate::graph::GraphModule;
use crate::help::HelpModule;
use crate::info::InfoModule;
use crate::lang::LangModule;
use crate::lumi::LumiModule;
use crate::maki::MakiModule;
use crate::pcx::PcxModule;
use crate::sdlport::{SDLPortModule, X_RES, Y_RES};
use crate::sj3::SJ3Module;
use crate::tuuli::TuuliModule;
use crate::unit::UnitModule;
use sdl2::event::Event;
use std::process;

fn main() {
    let sdl = sdl2::init().unwrap();
    let video_subsystem = sdl.video().unwrap();
    let timer_subsystem = sdl.timer().unwrap();
    let event_subsystem = sdl.event().unwrap();
    let event_pump = sdl.event_pump().unwrap();

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

    let sdl_port_module = SDLPortModule::init(
        &mut canvas,
        &texture_creator,
        &timer_subsystem,
        event_subsystem,
        event_pump,
        window_multiplier,
    );

    let help_module = HelpModule::new();
    let maki_module = MakiModule::new();

    let pcx_module = PcxModule::new(&maki_module, &sdl_port_module);

    let mut graph_module = GraphModule::new(&maki_module, &sdl_port_module);
    graph_module.load_anim("ANIM.SKI");

    let mut lang_module = LangModule::new();
    lang_module.load_language(1);

    let info_module = InfoModule::new(&graph_module, &lang_module, &pcx_module);
    let tuuli_module = TuuliModule::new(&graph_module, &help_module);
    let unit_module = UnitModule::new(
        &graph_module,
        &help_module,
        &lang_module,
        &pcx_module,
        &sdl_port_module,
    );

    let lumi_module = LumiModule::init();
    let mut sj3_module = SJ3Module::new(
        &graph_module,
        &help_module,
        &info_module,
        &lang_module,
        lumi_module,
        &maki_module,
        &tuuli_module,
        &unit_module,
    );
    sj3_module.main_menu();

    // loop {
    //     let event = event_pump.wait_event();
    //     if let Event::Quit { .. } = event {
    //         break;
    //     }
    //     graph_module.draw_screen();
    // }
}
