#![allow(non_snake_case)]

use wasm_bindgen::prelude::*;

use steps_core::cfg::{SimConfig, SummaryOutputConfig};

use create_typescript_interface::create_typescript_interface;

create_typescript_interface! {
    PortalRunConfig {
        simParams: (SimParams, JsSimParams),
        dataConfig: (DataConfig, JsDataConfig),
    }

    SimParams {
        replicates: (u32, number),
        transfers: (u32, number),
        maxPopSize: (f64, number),
        dilutionFactor: (f64, number),
        markers: (u16, number),
        beneficialMutationRate: (f64, number),
        neutralMutationRate: (f64, number),
        deleteriousMutationRate: (f64, number),
        initialBeneficialMutationSize: (f64, number),
        fixedDeleteriousMutationSize?: (f64, number),
        diminishingReturnsEpistasisStrength: (f64, number),
        seed: (u64, BigInt),
    }

    DataConfig {
        prepareCsv: (bool, boolean),
        trackedStatistics: (TrackedStatistics, JsTrackedStatistics),
        dataResolution: (u32, number),
    }

    TrackedStatistics {
        avgW: (bool, boolean),
        marker1Ratio: (bool, boolean),
        stdevW: (bool, boolean),
        maxW: (bool, boolean),
        stdevAccumulatedMuts: (bool, boolean),
        maxAccumulatedMuts: (bool, boolean),
        meanAccumulatedMuts: (bool, boolean),
        minAccumulatedMuts: (bool, boolean),
        genotypeCount: (bool, boolean),
        shannonDiversity: (bool, boolean),
    }

    SimDataPoint {
        transfer: (u32, number),
        avgW?: (f64, number),
        marker1Ratio?: (f64, number),
        stdevW?: (f64, number),
        maxW?: (f64, number),
        stdevAccumulatedMuts?: (f64, number),
        maxAccumulatedMuts?: (u32, number),
        meanAccumulatedMuts?: (f64, number),
        minAccumulatedMuts?: (u32, number),
        genotypeCount?: (usize, number),
        shannonDiversity?: (f64, number),
    }

    SimResultsFragment {
        replicate: (u32, number),
        points: (Vec<SimDataPoint>, JsSimDataPoint[])
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "JsSimResultsFragment[]")]
    pub type JsSimResultsFragments;
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
        initial_beneficial_mutation_size: sim_params.initialBeneficialMutationSize,
        fixed_deleterious_mutation_size: sim_params.fixedDeleteriousMutationSize,
        diminishing_returns_epistasis_strength: sim_params.diminishingReturnsEpistasisStrength,
        seed: Some(sim_params.seed),
    }
}

pub fn extract_summary_output_config(cfg: &PortalRunConfig) -> SummaryOutputConfig {
    let tracked_statistics = &cfg.dataConfig.trackedStatistics;
    SummaryOutputConfig {
        avg_W: tracked_statistics.avgW,
        marker_1_ratio: tracked_statistics.marker1Ratio,
        stdev_W: tracked_statistics.stdevW,
        max_W: tracked_statistics.maxW,
        stdev_accumulated_muts: tracked_statistics.stdevAccumulatedMuts,
        max_accumulated_muts: tracked_statistics.maxAccumulatedMuts,
        mean_accumulated_muts: tracked_statistics.meanAccumulatedMuts,
        min_accumulated_muts: tracked_statistics.minAccumulatedMuts,
        genotype_count: tracked_statistics.genotypeCount,
        shannon_diversity: tracked_statistics.shannonDiversity,
    }
}
