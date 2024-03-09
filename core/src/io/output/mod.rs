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
