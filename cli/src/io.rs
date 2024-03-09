//! IO helpers specifically for the CLI portion of STEPS

use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use anyhow::Result;

use steps_core::cfg::SimConfig;
use steps_core::io::{
    extract_sim_config, MutationSummaryOutputter, OutputterGroup, OutputterGroupBuilder,
    RawOutputter, SequencingOutputter, SummaryOutputter,
};

use crate::cfg::CliOutputConfig;

/// Get an `OutputterGroup` to generate output corresponding to the provided configs
pub fn outputter_group_for_cli(
    output_cfg: &CliOutputConfig,
    sim_cfg: &SimConfig,
) -> Result<OutputterGroup> {
    let mut builder =
        OutputterGroupBuilder::default().lineage_sampling_frequency(output_cfg.sampling_frequency);

    if let Some(path) = &output_cfg.raw_output_path {
        builder = builder.lineage_outputter(Box::new(RawOutputter::new(
            create_buffered_file(path)?,
            sim_cfg,
        )?));
    }

    if let Some(path) = &output_cfg.summary_output_path {
        builder = builder.lineage_outputter(Box::new(SummaryOutputter::new(
            create_buffered_file(path)?,
            output_cfg.summary_cfg.clone(),
            sim_cfg,
        )?));
    }

    if let Some(path) = &output_cfg.sequencing_output_path {
        builder = builder.mutation_outputter(Box::new(SequencingOutputter::new(
            create_buffered_file(path)?,
            sim_cfg,
        )?));
    }

    if let Some(path) = &output_cfg.mutation_summary_output_path {
        builder = builder.mutation_outputter(Box::new(MutationSummaryOutputter::new(
            create_buffered_file(path)?,
            sim_cfg,
        )?));
    }

    Ok(builder.build()?)
}

/// Buffer capacity to use for files
/// Set at 8 MB
const FILE_BUFFER_CAPACITY: usize = 8 * (1 << 20);

/// Create a buffered `File` to use
fn create_buffered_file<P: AsRef<Path>>(path: P) -> std::io::Result<BufWriter<File>> {
    Ok(BufWriter::with_capacity(
        FILE_BUFFER_CAPACITY,
        File::create(path)?,
    ))
}

/// Extract a `SimConfig` stored from a previous run from the file at a given path
pub fn extract_sim_config_from_path<P: AsRef<Path>>(path: P) -> Result<SimConfig> {
    File::open(path)
        .map_err(anyhow::Error::from)
        .and_then(extract_sim_config)
}
