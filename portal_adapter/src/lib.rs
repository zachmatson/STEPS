use std::ops::Deref;

use paste::paste;
use wasm_bindgen::prelude::*;

use steps::io::SummaryOutputter;
use steps::sim::*;

mod types;

use types::*;

// TODO:
//   Lot of duplication here, how much logic can be moved into Rust side?
//   Maybe make most of the WorkerSimRunner live in Rust,
//   and just return sim fragments over and over when called

#[wasm_bindgen]
pub struct JsSimulationHandler {
    inner: SimulationHandler,
    outputter: Option<Box<SummaryOutputter<Vec<u8>>>>,
    replicate: u32,
    transfer: u32,
}

#[wasm_bindgen]
impl JsSimulationHandler {
    pub fn new(cfg_js: JsPortalRunConfig) -> Self {
        let cfg: PortalRunConfig =
            serde_wasm_bindgen::from_value(cfg_js.deref().to_owned()).unwrap();
        let sim_cfg = extract_sim_config(&cfg);
        let inner = SimulationHandler::new(&sim_cfg, false);

        Self {
            replicate: 0,
            transfer: 0,
            inner,
            outputter: if cfg.dataConfig.prepareCSV {
                Some(Box::new(
                    SummaryOutputter::new(
                        Vec::new(),
                        extract_summary_output_config(&cfg),
                        &sim_cfg,
                    )
                        .unwrap(),
                ))
            } else {
                None
            },
        }
    }

    pub fn start_replicate(&mut self) {
        self.inner.start_replicate();
        self.replicate += 1;
        self.transfer = 0;
        self.record();
    }

    pub fn advance_and_record(&mut self, step: u32) {
        for _ in 0..step {
            self.inner.transfer();
        }
        self.transfer += step;
        self.record();
    }

    fn record(&mut self) {
        if let Some(outputter) = &mut self.outputter {
            outputter.record_lineages(self.replicate, self.transfer, self.inner.lineages()).unwrap();
        }
    }

    pub fn into_output_object_url(self) -> Option<String> {
        let output = self.outputter?.into_inner().ok()?;
        // Safety: Buffer invalidated on malloc from wasm code,
        // this buffer is only used here to create the blob on the next line,
        // so its validity doesn't matter afterwards
        let buffer = unsafe { js_sys::Uint8Array::view(&output) };
        let blob = web_sys::Blob::new_with_u8_array_sequence(&js_sys::Array::of1(&buffer)).ok()?;
        web_sys::Url::create_object_url_with_blob(&blob).ok()
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
