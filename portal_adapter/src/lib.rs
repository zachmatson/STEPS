use paste::paste;
use wasm_bindgen::prelude::*;
use web_sys;

use steps::{cfg::SimConfig, sim::*};

#[wasm_bindgen]
pub struct JSSimulationHandler {
    cfg: SimConfig,
    inner: SimulationHandler,
}

#[wasm_bindgen]
impl JSSimulationHandler {
    pub fn new(cfg_js: JsValue) -> Self {
        let mut cfg: SimConfig = serde_wasm_bindgen::from_value(cfg_js).unwrap();
        cfg.finish_initialization();
        let inner = SimulationHandler::new(cfg.clone(), false);
        Self { cfg, inner }
    }

    pub fn start_replicate(&mut self) {
        self.inner.start_replicate();
    }

    pub fn advance(&mut self, step: u64) {
        for _ in 0..step {
            self.inner.transfer();
        }
    }

    #[allow(non_snake_case)]
    pub fn check_avg_W(&mut self) -> f64 {
        summarize::sum_N_and_avg_W(self.inner.lineages()).1
    }
}

macro_rules! make_stat_check_functions {
    ($(($stat:ident, $ty:ident)),+) => {$(
        paste! {
            #[allow(non_snake_case)]
            #[wasm_bindgen]
            impl JSSimulationHandler {
                pub fn [<check_ $stat>](&mut self) -> $ty {
                    summarize::$stat(self.inner.lineages())
                }
            }
        }
    )+}
}

make_stat_check_functions! {
    (marker_1_ratio, f64),
    (stdev_W, f64),
    (max_W, f64),
    (stdev_accumulated_muts, f64),
    (max_accumulated_muts, u32),
    (genotype_count, usize),
    (shannon_diversity, f64)
}
