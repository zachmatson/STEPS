//! Output tools for STEPS

use std::io::Write;

use anyhow::Result;
use derive_builder::Builder;

use crate::cfg::SimConfig;
use crate::sim::{LineagesData, Mutation, MutationsData};

use crate::io::{Metadata, OutputMode};

mod outputter_impls;

pub use outputter_impls::{
    MutationSummaryOutputter, RawOutputter, SequencingOutputter, SummaryOutputter,
};

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// OutputterGroup
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// A handler which broadcasts recording functions to a group of underlying outputters
#[allow(missing_docs)] // Builder will not have doc comment
#[derive(Builder)]
#[builder(pattern = "owned")]
pub struct OutputterGroup {
    /// Frequency in transfers at which managed lineage outputters will be recorded to,
    /// only transfers that are a multiple of this number will actually be passed on.
    ///
    /// No effect on mutation outputs, defaults to `1`
    #[builder(default = "1")]
    lineage_sampling_frequency: u32,
    /// Outputters for lineage data
    #[builder(setter(each(name = "lineage_outputter")), default)]
    lineages_outputters: Vec<Box<dyn LineagesOutputter>>,
    /// Outputters for mutation dataa
    #[builder(setter(each(name = "mutation_outputter")), default)]
    mutations_outputters: Vec<Box<dyn MutationsOutputter>>,
}

impl OutputterGroup {
    /// Record information for the provided `LineagesData` for the given replicate and transfer in
    /// all of the managed `LineageOutputter`s
    pub fn record_lineages(
        &mut self,
        replicate: u32,
        transfer: u32,
        lineages: &LineagesData,
    ) -> Result<()> {
        if transfer % self.lineage_sampling_frequency == 0 {
            for outputter in &mut self.lineages_outputters {
                outputter.record_lineages(replicate, transfer, lineages)?;
            }
        }
        Ok(())
    }

    /// Record information for the pruned mutations in the provided `MutationsData` for the given
    /// replicate and transfer in all of the managed `MutationsOutputter`s
    ///
    /// Pruned mutations should be recorded at each transfer to avoid missing any
    pub fn record_pruned_mutations(
        &mut self,
        replicate: u32,
        mutations: &MutationsData,
    ) -> Result<()> {
        for outputter in &mut self.mutations_outputters {
            outputter.record_pruned_mutations(replicate, mutations)?;
        }
        Ok(())
    }

    /// Record information for the active mutations in the provided `MutationsData` for the given
    /// replicate and transfer in all of the managed `MutationsOutputter`s
    ///
    /// Active mutations may eventually become pruned, and should probably only be recorded at the
    /// end of a replicate to avoid duplicate recording
    pub fn record_active_mutations(
        &mut self,
        replicate: u32,
        mutations: &MutationsData,
    ) -> Result<()> {
        for outputter in &mut self.mutations_outputters {
            outputter.record_active_mutations(replicate, mutations)?;
        }
        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Traits
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// An outputter that can record the data for `LineagesData`
pub trait LineagesOutputter {
    /// Record the data in `lineages`, at a specific replicate and transfer
    fn record_lineages(
        &mut self,
        replicate: u32,
        transfer: u32,
        lineages: &LineagesData,
    ) -> Result<()>;
}

/// And outputter that can record the data for `MutationsData`
pub trait MutationsOutputter {
    /// Record a single `mutation` at a specific replicate and transfer
    fn record_mutation(&mut self, replicate: u32, mutation: &Mutation) -> Result<()>;
}

impl dyn MutationsOutputter {
    /// Record all pruned mutations in some `MutationsData`
    ///
    /// Pruned mutations should be recorded at each transfer to avoid missing any
    pub fn record_pruned_mutations(
        &mut self,
        replicate: u32,
        mutations: &MutationsData,
    ) -> Result<()> {
        for mutation in &mutations.pruned_muts {
            self.record_mutation(replicate, mutation)?;
        }
        Ok(())
    }

    /// Record all active mutations in some `MutationsData`
    ///
    /// Active mutations may eventually become pruned, and should probably only be recorded at the
    /// end of a replicate to avoid duplicate recording
    pub fn record_active_mutations(
        &mut self,
        replicate: u32,
        mutations: &MutationsData,
    ) -> Result<()> {
        for mutation in mutations.muts.values() {
            self.record_mutation(replicate, mutation)?;
        }
        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Utils
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Output `Metadata` and `SimConfig` options into a header using provided `writer`
///
/// Allow an optional prefix for lines of the header (e.g. for comments)
fn initialize_output<W: Write>(
    writer: &mut W,
    sim_cfg: &SimConfig,
    output_mode: OutputMode,
    header_prefix: &'static str,
) -> Result<()> {
    // Write the metadata to the file with optional comment character
    write!(writer, "{}", header_prefix)?;
    let metadata = Metadata::new(output_mode);
    serde_json::to_writer(writer.by_ref(), &metadata)?;
    writeln!(writer)?;

    // Write the simulation configuration to the file with optional comment character
    write!(writer, "{}", header_prefix)?;
    serde_json::to_writer(writer.by_ref(), sim_cfg)?;
    writeln!(writer)?;

    Ok(())
}

/// Manually moving onto the next record in the `csv` crate requires writing an empty record
const EMPTY_CSV_RECORD: [&[u8]; 0] = [];

/// Buffer capacity to use for CSV writer
///
/// Set at 128 KB
const CSV_BUFFER_CAPACITY: usize = 128 * (1 << 10);

/// Initialize a `writer` as described in `initialize_output` and get a `csv::Writer` over the
/// underlying `writer`
fn initialize_output_as_csv<W: Write>(
    mut writer: W,
    sim_cfg: &SimConfig,
    output_mode: OutputMode,
) -> Result<csv::Writer<W>> {
    initialize_output(&mut writer, sim_cfg, output_mode, "# ")?;

    Ok(csv::WriterBuilder::new()
        .buffer_capacity(CSV_BUFFER_CAPACITY)
        .from_writer(writer))
}

#[cfg(test)]
pub(super) mod tests {
    use super::*;
    use crate::cfg::SummaryOutputConfig;
    use std::cell::RefCell;
    use std::rc::Rc;

    /// A `SimConfig` to write into output headers
    pub(super) fn sim_cfg() -> SimConfig {
        SimConfig {
            replicates: 1,
            transfers: 3,
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

    /// A `SummaryOutputConfig` with every stat turned off, to be overridden per test
    pub(super) fn all_stats_disabled() -> SummaryOutputConfig {
        SummaryOutputConfig {
            avg_W: false,
            marker_1_ratio: false,
            stdev_W: false,
            max_W: false,
            stdev_accumulated_muts: false,
            max_accumulated_muts: false,
            mean_accumulated_muts: false,
            min_accumulated_muts: false,
            genotype_count: false,
            shannon_diversity: false,
        }
    }

    /// Two equally sized lineages with fitnesses 1.0 and 2.0
    ///
    /// Built through `serde` because the fields of `LineagesData` are private to `sim`
    pub(super) fn two_lineages() -> LineagesData {
        serde_json::from_str(
            r#"{"N":[100.0,100.0],"W":[1.0,2.0],"U":[0.0,0.0],
                "secondary":[[0.0,1,0,1,1],[0.0,2,0,1,1]]}"#,
        )
        .unwrap()
    }

    /// A `LineagesOutputter` which records the transfers it was called for
    ///
    /// The group takes ownership of its outputters, so the log is shared through an `Rc`
    struct RecordingOutputter(Rc<RefCell<Vec<u32>>>);

    impl LineagesOutputter for RecordingOutputter {
        fn record_lineages(
            &mut self,
            _replicate: u32,
            transfer: u32,
            _lineages: &LineagesData,
        ) -> Result<()> {
            self.0.borrow_mut().push(transfer);
            Ok(())
        }
    }

    /// Record transfers `0..=6` into a group with the given sampling frequency
    fn transfers_recorded_at_frequency(frequency: u32) -> Vec<u32> {
        let log = Rc::new(RefCell::new(Vec::new()));
        let mut group = OutputterGroupBuilder::default()
            .lineage_sampling_frequency(frequency)
            .lineage_outputter(Box::new(RecordingOutputter(Rc::clone(&log))))
            .build()
            .unwrap();

        let lineages = two_lineages();
        for transfer in 0..=6 {
            group.record_lineages(1, transfer, &lineages).unwrap();
        }

        drop(group);
        Rc::try_unwrap(log).unwrap().into_inner()
    }

    #[test]
    fn test_initialize_output_writes_metadata_then_config() {
        let mut output = Vec::new();
        initialize_output(&mut output, &sim_cfg(), OutputMode::Summary, "").unwrap();

        let output = String::from_utf8(output).unwrap();
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 2);

        let metadata: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
        assert_eq!(metadata["name"], "STEPS");
        assert_eq!(metadata["output_mode"], "Summary");
        assert_eq!(metadata["version"], env!("CARGO_PKG_VERSION"));

        let cfg: serde_json::Value = serde_json::from_str(lines[1]).unwrap();
        assert_eq!(cfg["replicates"], 1);
        assert_eq!(cfg["transfers"], 3);
        assert_eq!(cfg["seed"], 42);
    }

    #[test]
    fn test_initialize_output_applies_header_prefix() {
        let mut output = Vec::new();
        initialize_output(&mut output, &sim_cfg(), OutputMode::Summary, "# ").unwrap();

        let output = String::from_utf8(output).unwrap();
        for line in output.lines() {
            assert!(line.starts_with("# "), "line was not prefixed: {}", line);
        }
    }

    #[test]
    fn test_initialize_output_records_output_mode() {
        let mut output = Vec::new();
        initialize_output(&mut output, &sim_cfg(), OutputMode::Sequencing, "").unwrap();

        let output = String::from_utf8(output).unwrap();
        let metadata: serde_json::Value =
            serde_json::from_str(output.lines().next().unwrap()).unwrap();
        assert_eq!(metadata["output_mode"], "Sequencing");
    }

    #[test]
    fn test_group_records_every_transfer_at_frequency_one() {
        assert_eq!(transfers_recorded_at_frequency(1), [0, 1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn test_group_skips_transfers_not_matching_frequency() {
        assert_eq!(transfers_recorded_at_frequency(3), [0, 3, 6]);
    }

    #[test]
    fn test_group_default_frequency_records_every_transfer() {
        let mut group = OutputterGroupBuilder::default().build().unwrap();
        // Default frequency is 1, so nothing is skipped and no outputters means no errors
        assert!(group.record_lineages(1, 1, &two_lineages()).is_ok());
    }
}
