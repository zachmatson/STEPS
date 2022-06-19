use serde::Deserialize;
use wasm_bindgen::prelude::*;

use steps::cfg::{SimConfig, SummaryOutputConfig};

/*
   Due to current limitations, the TypeScript interface and Rust types defined below it must be
   manually kept in sync. Assuming this is done, everything else should get caught statically.
*/

#[wasm_bindgen(typescript_custom_section)]
const TS_TYPES: &'static str = r#"
interface JsPortalRunConfig {
  simParams: {
    replicates: number;
    transfers: number;
    maxPopSize: number;
    dilutionFactor: number;
    markers: number;
    beneficialMutationRate: number;
    neutralMutationRate: number;
    deleteriousMutationRate: number;
    mutationRateMutationRate: number;
    initialBeneficialMutationSize: number;
    deleteriousMutationSizeFactor: number;
    mutationRateMutationSizeFactor: number;
    diminishingReturnsEpistasisStrength: number;
    seed: BigInt;
  };
  dataConfig: {
    prepareCSV: boolean;
    trackedStatistics: {
      avgW: boolean;
      marker1Ratio: boolean;
      stdevW: boolean;
      maxW: boolean;
      stdevAccumulatedMuts: boolean;
      maxAccumulatedMuts: boolean;
      genotypeCount: boolean;
      shannonDiversity: boolean;
    };
    dataResolution?: number;
  };
}
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "JsPortalRunConfig")]
    pub type JsPortalRunConfig;
}

#[allow(non_snake_case)]
#[derive(Deserialize)]
pub struct SimParams {
    pub replicates: u32,
    pub transfers: u32,
    pub maxPopSize: f64,
    pub dilutionFactor: f64,
    pub markers: u16,
    pub beneficialMutationRate: f64,
    pub neutralMutationRate: f64,
    pub deleteriousMutationRate: f64,
    pub mutationRateMutationRate: f64,
    pub initialBeneficialMutationSize: f64,
    pub deleteriousMutationSizeFactor: f64,
    pub mutationRateMutationSizeFactor: f64,
    pub diminishingReturnsEpistasisStrength: f64,
    pub seed: u64,
}

#[allow(non_snake_case)]
#[derive(Deserialize)]
pub struct TrackedStatistics {
    pub avgW: bool,
    pub marker1Ratio: bool,
    pub stdevW: bool,
    pub maxW: bool,
    pub stdevAccumulatedMuts: bool,
    pub maxAccumulatedMuts: bool,
    pub genotypeCount: bool,
    pub shannonDiversity: bool,
}

#[allow(non_snake_case)]
#[derive(Deserialize)]
pub struct DataConfig {
    pub prepareCSV: bool,
    pub trackedStatistics: TrackedStatistics,
    pub dataResolution: Option<u32>,
}

#[allow(non_snake_case)]
#[derive(Deserialize)]
pub struct PortalRunConfig {
    pub simParams: SimParams,
    pub dataConfig: DataConfig,
}

pub fn extract_sim_config(cfg: &PortalRunConfig) -> SimConfig {
    let sim_params = &cfg.simParams;
    SimConfig {
        replicates: sim_params.replicates,
        transfers: sim_params.transfers,
        max_pop_size: sim_params.maxPopSize,
        dilution_factor: sim_params.dilutionFactor,
        markers: sim_params.markers,
        beneficial_mutation_rate: sim_params.beneficialMutationRate,
        neutral_mutation_rate: sim_params.neutralMutationRate,
        deleterious_mutation_rate: sim_params.deleteriousMutationRate,
        mutation_rate_mutation_rate: sim_params.mutationRateMutationRate,
        initial_beneficial_mutation_size: sim_params.initialBeneficialMutationSize,
        deleterious_mutation_size_factor: sim_params.deleteriousMutationSizeFactor,
        mutation_rate_mutation_size_factor: sim_params.mutationRateMutationSizeFactor,
        diminishing_returns_epistasis_strength: sim_params.diminishingReturnsEpistasisStrength,
        seed: Some(sim_params.seed),
    }
}

pub fn extract_summary_output_config(cfg: &PortalRunConfig) -> SummaryOutputConfig {
    let tracked_statistics = &cfg.dataConfig.trackedStatistics;
    SummaryOutputConfig {
        marker_1_ratio: tracked_statistics.marker1Ratio,
        stdev_W: tracked_statistics.stdevW,
        max_W: tracked_statistics.maxW,
        stdev_accumulated_muts: tracked_statistics.stdevAccumulatedMuts,
        max_accumulated_muts: tracked_statistics.maxAccumulatedMuts,
        genotype_count: tracked_statistics.genotypeCount,
        shannon_diversity: tracked_statistics.shannonDiversity,
    }
}
