use std::ops::Deref;

use wasm_bindgen::prelude::*;

use steps::io::{LineagesOutputter, SummaryOutputter};
use steps::sim::{summarize, LineagesData, SimulationHandler, SimulationState};

mod types;

use types::*;

#[wasm_bindgen]
pub struct JsSimulationHandler {
    cfg: PortalRunConfig,
    inner: SimulationHandler,
    outputter: Option<Box<SummaryOutputter<Vec<u8>>>>,
}

#[wasm_bindgen]
impl JsSimulationHandler {
    pub fn new(cfg_js: JsPortalRunConfig) -> Self {
        console_error_panic_hook::set_once();
        let cfg: PortalRunConfig =
            serde_wasm_bindgen::from_value(cfg_js.deref().to_owned()).unwrap();
        let sim_cfg = extract_sim_config(&cfg);

        Self {
            outputter: match cfg.dataConfig.prepareCSV {
                true => Some(Box::new(
                    SummaryOutputter::new(
                        Vec::new(),
                        extract_summary_output_config(&cfg),
                        &sim_cfg,
                    )
                    .unwrap(),
                )),
                false => None,
            },
            inner: SimulationHandler::new(sim_cfg, false),
            cfg,
        }
    }

    pub fn next_fragment(&mut self, max_time_ms: u32) -> Option<JsSimResultsFragments> {
        let end_at = js_sys::Date::now() + max_time_ms as f64;

        if self.inner.is_finished() {
            return None;
        }

        let resolution = self.cfg.dataConfig.dataResolution;
        let mut results: Vec<SimResultsFragment> = Vec::new();

        while js_sys::Date::now() < end_at {
            if let Some(state) = self.inner.next_state() {
                if state.end_of_replicate || state.transfer % resolution == 0 {
                    let SimulationState {
                        replicate,
                        transfer,
                        lineages,
                        ..
                    } = state;

                    let fragment = match results.iter_mut().last() {
                        Some(fragment) if fragment.replicate == replicate => fragment,
                        _ => {
                            results.push(SimResultsFragment {
                                replicate,
                                points: Vec::new(),
                            });
                            results.iter_mut().last().unwrap()
                        }
                    };

                    fragment.points.push(make_data_point(
                        transfer_to_generation(transfer, self.cfg.simParams.dilutionFactor),
                        &self.cfg.dataConfig.trackedStatistics,
                        lineages,
                    ));

                    if let Some(outputter) = &mut self.outputter {
                        outputter
                            .record_lineages(replicate, transfer, lineages)
                            .unwrap();
                    }
                }
            }
        }

        Some(JsValue::from_serde(&results).unwrap().into())
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
}

fn transfer_to_generation(transfer: u32, dilution_factor: f64) -> f64 {
    transfer as f64 * dilution_factor.log2()
}

macro_rules! impl_make_data_point {
    ($(($config_name:ident, $fn_name:ident)),*) => {
        fn make_data_point(
            generation: f64,
            tracked_statistics: &TrackedStatistics,
            lineages: &LineagesData,
        ) -> SimDataPoint {
            SimDataPoint {
                generation,
                $(
                    $config_name: tracked_statistics
                        .$config_name
                        .then(|| summarize::$fn_name(lineages))
                ),*
            }
        }
    }
}

impl_make_data_point! {
    (avgW, avg_W),
    (marker1Ratio, marker_1_ratio),
    (stdevW, stdev_W),
    (maxW, max_W),
    (stdevAccumulatedMuts, stdev_accumulated_muts),
    (maxAccumulatedMuts, max_accumulated_muts),
    (genotypeCount, genotype_count),
    (shannonDiversity, shannon_diversity)
}
