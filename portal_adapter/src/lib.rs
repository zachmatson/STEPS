use std::ops::Deref;

use wasm_bindgen::prelude::*;

use steps_core::io::{LineagesOutputter, SummaryOutputter};
use steps_core::sim::{summarize, LineagesData, SimulationHandler, SimulationState};

use types::*;

mod types;

/// Handler type which JavaScript code can use to run and get results from the STEPS simulations
#[wasm_bindgen]
pub struct JsSimulationHandler {
    /// Config used to create this handler
    cfg: PortalRunConfig,
    /// Inner handler from STEPS library
    inner: SimulationHandler,
    /// Optional outputter to record CSV summary data if enabled
    ///
    /// Boxed to avoid the memory usage of this type unless necessary
    outputter: Option<Box<SummaryOutputter<Vec<u8>>>>,
}

#[wasm_bindgen]
impl JsSimulationHandler {
    /// Create new handler for the given configuration
    pub fn new(cfg_js: JsPortalRunConfig) -> Self {
        console_error_panic_hook::set_once();
        let cfg: PortalRunConfig =
            serde_wasm_bindgen::from_value(cfg_js.deref().to_owned()).unwrap();
        let sim_cfg = extract_sim_config(&cfg);

        Self {
            outputter: cfg.dataConfig.prepareCsv.then(|| {
                Box::new(
                    SummaryOutputter::new(
                        Vec::new(),
                        extract_summary_output_config(&cfg),
                        &sim_cfg,
                    )
                    .unwrap(),
                )
            }),
            inner: SimulationHandler::new(sim_cfg, false),
            cfg,
        }
    }

    /// Get next set of simulation result fragments back from the simulations, running for
    /// approximately up to `max_time_ms`
    pub fn next_fragment(&mut self, max_time_ms: u32) -> Option<JsSimResultsFragments> {
        if self.inner.is_finished() {
            return None;
        }

        let end_at = js_sys::Date::now() + max_time_ms as f64;

        let resolution = self.cfg.dataConfig.dataResolution;
        let mut results: Vec<SimResultsFragment> = Vec::new();

        // Checking the time every transfer instead of every time we are at the data resolution
        // seems to give a slightly smoother look, and performance impact is minimal

        while js_sys::Date::now() < end_at {
            if let Some(state) = self.inner.next_state() {
                if state.end_of_replicate || state.transfer % resolution == 0 {
                    let SimulationState {
                        replicate,
                        transfer,
                        lineages,
                        ..
                    } = state;

                    let fragment = match results.last_mut() {
                        Some(fragment) if fragment.replicate == replicate => fragment,
                        _ => {
                            results.push(SimResultsFragment {
                                replicate,
                                points: Vec::new(),
                            });
                            results.last_mut().unwrap()
                        }
                    };

                    fragment.points.push(make_data_point(
                        transfer,
                        &self.cfg.dataConfig.trackedStatistics,
                        lineages,
                    ));

                    if let Some(outputter) = &mut self.outputter {
                        outputter
                            .record_lineages(replicate, transfer, lineages)
                            .unwrap();
                    }
                }
            } else {
                break;
            }
        }

        Some(serde_wasm_bindgen::to_value(&results).unwrap().into())
    }

    /// Consume the handler, rendering it unusable, and return a string corresponding to the URL
    /// at which CSV results can be downloaded (if they are being tracked)
    pub fn into_output_object_url(self) -> Option<String> {
        let output = self.outputter?.into_inner().ok()?;
        let blob = {
            // Safety: Buffer invalidated on malloc from wasm code,
            // this buffer is only used here to create the blob on the next line,
            // so its validity doesn't matter afterwards
            let buffer = unsafe { js_sys::Uint8Array::view(&output) };
            web_sys::Blob::new_with_u8_array_sequence(&js_sys::Array::of1(&buffer)).ok()?
        };
        web_sys::Url::create_object_url_with_blob(&blob).ok()
    }
}

/// Automate the repetitive process of generating the `make_data_point` function
///
/// Macro should be provided with a series of mappings between the name of the flag in
/// `TrackedStatistics` that corresponds to a stat and the function to compute that stat for
/// the lineages
macro_rules! impl_make_data_point {
    ($($config_name:ident => $fn_name:ident),*,) => {
        /// Create a data point corresponding to some `LineagesData` at a generation by computing
        /// the requested statistics
        fn make_data_point(
            transfer: u32,
            tracked_statistics: &TrackedStatistics,
            lineages: &LineagesData,
        ) -> SimDataPoint {
            SimDataPoint {
                transfer,
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
    avgW => avg_W,
    marker1Ratio => marker_1_ratio,
    stdevW => stdev_W,
    maxW => max_W,
    stdevAccumulatedMuts => stdev_accumulated_muts,
    maxAccumulatedMuts => max_accumulated_muts,
    meanAccumulatedMuts => mean_accumulated_muts,
    minAccumulatedMuts => min_accumulated_muts,
    genotypeCount => genotype_count,
    shannonDiversity => shannon_diversity,
}
