use std::ops::Deref;

use wasm_bindgen::prelude::*;
use paste::paste;

use steps::sim::*;
use web_sys::console::assert;

mod types;

pub use types::*;

#[wasm_bindgen]
pub struct JsSimulationHandler {
    inner: SimulationHandler,
}

#[wasm_bindgen]
impl JsSimulationHandler {
    pub fn new(cfg_js: JsPortalRunConfig) -> Self {
        let cfg: PortalRunConfig = serde_wasm_bindgen::from_value(cfg_js.deref().to_owned()).unwrap();
        let inner = SimulationHandler::new(&extract_sim_config(cfg), false);

        Self { inner }
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
            impl JsSimulationHandler {
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
