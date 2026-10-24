#![allow(async_fn_in_trait)]

pub mod files;
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
mod sj3;
mod table;
pub mod trace;
mod tuuli;
mod unit;

use crate::graph::GraphModule;
use crate::info::InfoModule;
use crate::lang::LangModule;
use crate::list::ListModule;
use crate::lumi::LumiModule;
use crate::maki::MakiModule;
use crate::pcx::PcxModule;
use crate::sj3::SJ3Module;
use crate::tuuli::TuuliModule;
use crate::unit::{Date, UnitModule};
pub use platform::{Platform, TPalette};
use std::future::{poll_fn, Future};
use std::pin::pin;
use std::task::Poll;

pub async fn sj3<P: Platform>(port: &P) {
    rs_util::reset_console();
    rs_util::until_halt(run(port)).await;
}

async fn run<P: Platform>(port: &P) {
    let maki_module = MakiModule::new();
    let pcx_module = PcxModule::new(&maki_module, port);
    let graph_module = GraphModule::new(&maki_module, &pcx_module, port);

    let mut lang_module = LangModule::new(port);
    lang_module.init();

    let unit_module = UnitModule::new(&graph_module, &lang_module, &maki_module, &pcx_module, port);
    let list_module =
        ListModule::new(&graph_module, &lang_module, &pcx_module, port, &unit_module).await;
    let tuuli_module = TuuliModule::new(&graph_module);
    let info_module = InfoModule::new(&graph_module, &lang_module, &pcx_module, port, &unit_module);
    let lumi_module = LumiModule::init();

    let (year, month, day, day_of_week) = port.get_date();
    let today = Date {
        year,
        month,
        day,
        day_of_week,
    };
    let start = unit_module.get_now();

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

    sj3_module.today = today;
    sj3_module.start = start;

    // A close before alku has read the data files must not save, because the
    // saved files would replace the player's files with the defaults.
    if until_close_requested(port, sj3_module.alku()).await {
        return;
    }
    let close_requested = until_close_requested(port, sj3_module.main_menu()).await;
    if close_requested {
        sj3_module.close_and_halt().await;
    }

    sj3_module.close().await;
}

/// Runs `game` until it ends or until the platform asks to close. A close
/// stops the game at its next await. Returns true if the platform asked to
/// close.
async fn until_close_requested<P: Platform>(port: &P, game: impl Future<Output = ()>) -> bool {
    let mut game = pin!(game);
    poll_fn(|cx| {
        if port.close_requested() {
            return Poll::Ready(true);
        }
        game.as_mut().poll(cx).map(|()| false)
    })
    .await
}
