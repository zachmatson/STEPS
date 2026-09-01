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

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    use tempfile::TempDir;

    use crate::cfg::{CliCommand, CliConfig};

    fn sim_cfg() -> SimConfig {
        SimConfig {
            replicates: 1,
            transfers: 2,
            markers: 1,
            dilution_factor: 100.0,
            beneficial_mutation_rate: 1.7e-6,
            neutral_mutation_rate: 0.0,
            deleterious_mutation_rate: 0.0,
            initial_beneficial_mutation_size: 0.012,
            fixed_deleterious_mutation_size: None,
            diminishing_returns_epistasis_strength: 6.0,
            seed: Some(42),
            max_pop_size: 5e8,
        }
    }

    /// Parse the output config out of a `simulate` command line
    fn output_cfg_from(args: &[&str]) -> CliOutputConfig {
        match CliConfig::try_parse_from(args).unwrap().command {
            CliCommand::Simulate(simulate) => simulate.output_cfg,
            CliCommand::Reproduce(_) => panic!("expected a simulate command"),
        }
    }

    #[test]
    fn test_group_creates_a_file_for_each_requested_output() {
        let dir = TempDir::new().unwrap();
        let paths = ["summary.csv", "raw.ndjson", "seq.ndjson", "muts.csv"]
            .map(|name| dir.path().join(name));

        let output_cfg = output_cfg_from(&[
            "steps",
            "simulate",
            "--summary-output",
            paths[0].to_str().unwrap(),
            "--raw-output",
            paths[1].to_str().unwrap(),
            "--sequencing-output",
            paths[2].to_str().unwrap(),
            "--mutation-summary-output",
            paths[3].to_str().unwrap(),
        ]);

        let group = outputter_group_for_cli(&output_cfg, &sim_cfg()).unwrap();
        // Files are created eagerly when the outputters are built
        drop(group);

        for path in &paths {
            assert!(path.exists(), "{} was not created", path.display());
        }
    }

    #[test]
    fn test_group_creates_no_files_when_no_outputs_requested() {
        let dir = TempDir::new().unwrap();
        let output_cfg = output_cfg_from(&["steps", "simulate"]);

        outputter_group_for_cli(&output_cfg, &sim_cfg()).unwrap();

        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 0);
    }

    #[test]
    fn test_group_fails_for_an_unwritable_path() {
        let dir = TempDir::new().unwrap();
        // A path under a directory that does not exist cannot be created
        let path = dir.path().join("missing").join("summary.csv");
        let output_cfg = output_cfg_from(&[
            "steps",
            "simulate",
            "--summary-output",
            path.to_str().unwrap(),
        ]);

        assert!(outputter_group_for_cli(&output_cfg, &sim_cfg()).is_err());
    }

    #[test]
    fn test_extract_sim_config_round_trips_through_an_output_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("summary.csv");
        let output_cfg = output_cfg_from(&[
            "steps",
            "simulate",
            "--summary-output",
            path.to_str().unwrap(),
        ]);

        // Writing the outputter header is what records the config into the file
        let cfg = sim_cfg();
        drop(outputter_group_for_cli(&output_cfg, &cfg).unwrap());

        let extracted = extract_sim_config_from_path(&path).unwrap();
        assert_eq!(extracted.replicates, cfg.replicates);
        assert_eq!(extracted.transfers, cfg.transfers);
        assert_eq!(extracted.seed, cfg.seed);
        assert_eq!(extracted.max_pop_size, cfg.max_pop_size);
    }

    #[test]
    fn test_extract_sim_config_fails_for_a_missing_file() {
        let dir = TempDir::new().unwrap();
        assert!(extract_sim_config_from_path(dir.path().join("nope.csv")).is_err());
    }

    #[test]
    fn test_extract_sim_config_fails_for_a_file_without_headers() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("empty.csv");
        std::fs::write(&path, "").unwrap();

        assert!(extract_sim_config_from_path(&path).is_err());
    }
}
