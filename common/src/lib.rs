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
use crate::sj3::SJ3Module;
use crate::tuuli::TuuliModule;
use crate::unit::UnitModule;
pub use platform::{Platform, TPalette};

pub async fn sj3<P: Platform>(port: &P) {
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
