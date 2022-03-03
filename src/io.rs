//! Types to handle the output of simulation data and retrieval of encoded
//! metadata and configuration settings

use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Lines, Write};
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_tuple::*;

use crate::{
    cfg::{CLIOutputConfig, SimConfig, SummaryOutputConfig},
    sim::{summarize, LineagesData, Mutation, MutationsData},
};

/// Type which handles the details of outputting simulation results from files
pub struct OutputHandler {
    /// Frequency at which to output results
    ///
    /// Outputs must be done when the transfer number is a multiple
    /// of `sampling_frequency`
    sampling_frequency: u32,

    /// Outputter for the raw output mode, if applicable
    raw_outputter: Option<RawOutputter<BufWriter<File>>>,
    /// Outputter for the summary output mode, if applicable
    summary_outputter: Option<SummaryOutputter<BufWriter<File>>>,
    /// Outputter for the sequencing output mode, if applicable
    sequencing_outputter: Option<SequencingOutputter<BufWriter<File>>>,
    /// Outputter for the mutation summary output mode, if applicable
    mutation_summary_outputter: Option<MutationSummaryOutputter<BufWriter<File>>>,
}

impl OutputHandler {
    /// Create a new `OutputHandler` from options in an `OutputConfig` and `SimConfig`
    pub fn new(output_cfg: &CLIOutputConfig, sim_cfg: &SimConfig) -> Result<Self, Box<dyn Error>> {
        let raw_outputter = if let Some(path) = &output_cfg.raw_output_path {
            Some(RawOutputter::new(create_buffered_file(path)?, sim_cfg)?)
        } else {
            None
        };

        let summary_outputter = if let Some(path) = &output_cfg.summary_output_path {
            Some(SummaryOutputter::new(
                create_buffered_file(path)?,
                output_cfg.summary_cfg.clone(),
                sim_cfg,
            )?)
        } else {
            None
        };

        let sequencing_outputter = if let Some(path) = &output_cfg.sequencing_output_path {
            Some(SequencingOutputter::new(
                create_buffered_file(path)?,
                sim_cfg,
            )?)
        } else {
            None
        };

        let mutation_summary_outputter =
            if let Some(path) = &output_cfg.mutation_summary_output_path {
                Some(MutationSummaryOutputter::new(
                    create_buffered_file(path)?,
                    sim_cfg,
                )?)
            } else {
                None
            };

        Ok(Self {
            sampling_frequency: output_cfg.sampling_frequency,
            raw_outputter,
            summary_outputter,
            sequencing_outputter,
            mutation_summary_outputter,
        })
    }

    /// Output information from `lineages` as necessary
    #[inline(always)]
    pub fn handle_output_for_transfer(
        &mut self,
        r: u32,
        t: u32,
        lineages: &LineagesData,
    ) -> Result<(), Box<dyn Error>> {
        // Only output if at the sampling frequency
        if t % self.sampling_frequency == 0 {
            if let Some(raw_outputter) = &mut self.raw_outputter {
                raw_outputter.record_lineages(r, t, lineages)?;
            }

            if let Some(summary_outputter) = &mut self.summary_outputter {
                summary_outputter.record_lineages(r, t, lineages)?;
            }
        }

        Ok(())
    }

    /// Output the pruned mutations from a `MutationsData`
    ///
    /// This will output *all* currently stored pruned mutations,
    /// so mutations will be output more than once if this is called
    /// repeatedly without clearing the pruned mutations in between calls
    pub fn output_pruned_mutations(
        &mut self,
        r: u32,
        mutations_data: &MutationsData,
    ) -> Result<(), Box<dyn Error>> {
        if let Some(sequencing_outputter) = self.sequencing_outputter.as_mut() {
            sequencing_outputter.record_pruned_mutations(mutations_data)?;
        }
        if let Some(mutation_summary_outputter) = self.mutation_summary_outputter.as_mut() {
            mutation_summary_outputter.record_pruned_mutations(r, mutations_data)?;
        }

        Ok(())
    }

    /// Finish outputting mutations at the end of a replicate
    ///
    /// This outputs all mutations, pruned or otherwise,
    /// so if `output_pruned_mutations` was called without
    /// clearing pruned mutations in between, pruned
    /// mutations would be output again
    ///
    /// Additionally, calling this and continuing to run transfers
    /// afterwards will cause undesired behavior, because upon outputting
    /// again will output a second and newer version of the data for some
    /// mutations, coexisting with the old version. The function should therefore
    /// only be called at the *end* of a replicate, whether or not pruned
    /// mutations are cleared.
    pub fn finish_replicate_mutations(
        &mut self,
        r: u32,
        mutations_data: &MutationsData,
    ) -> Result<(), Box<dyn Error>> {
        self.output_pruned_mutations(r, mutations_data)?;

        if let Some(sequencing_outputter) = self.sequencing_outputter.as_mut() {
            sequencing_outputter.record_active_mutations(mutations_data)?;
            sequencing_outputter.deliminate_replicate_end()?;
        }
        if let Some(mutation_summary_outputter) = self.mutation_summary_outputter.as_mut() {
            mutation_summary_outputter.record_active_mutations(r, mutations_data)?;
        }

        Ok(())
    }
}

/// Type of output to produce
#[derive(Serialize, Deserialize, Copy, Clone)]
enum OutputMode {
    /// Full lineage data for each lineage, as ndjson
    Raw,
    /// Population summary information only, as CSV
    Summary,
    /// Information about each mutation that occurs, as ndjson
    Sequencing,
    /// Summary information about mutations, as CSV
    MutationSummary,
}

/// Get the current version of STEPS as defined in Cargo.toml
fn get_current_version_string() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Information to identify the simulation used to create the data  
/// Used to mark output files as created by these simulations  
/// And for verifying version information in the future when converting outputs
/// as older and newer versions may incompatible formats
#[derive(Serialize, Deserialize)]
struct Metadata {
    name: String,
    version: String,
    description: String,
    output_mode: OutputMode,
}

impl Metadata {
    /// Construct a new `Metadata` instance based on the current version of the
    /// code and the desired `OutputMode`
    fn new(output_mode: OutputMode) -> Self {
        Self {
            name: "STEPS".to_string(),
            version: get_current_version_string(),
            description:
                "STEPS simulation of bacterial evolution written by Devin Lake and Zachary Matson"
                    .to_string(),
            output_mode,
        }
    }
}

#[derive(Serialize_tuple)]
struct LineagesRecord<'a> {
    r: u32,
    t: u32,
    lineages: &'a LineagesData,
}

#[derive(Serialize_tuple, Deserialize_tuple)]
struct OwnedLineagesRecord {
    r: u32,
    t: u32,
    lineages: LineagesData,
}

/// Buffer capacity to use in outputs  
/// Set at 8 MB
const BUFFER_CAPACITY: usize = 8 * (1 << 20);
/// Buffer capacity for writing/reading header
/// Set at 2 KB
const HEADER_BUFFER_CAPACITY: usize = 2 * (1 << 10);

/// Type which outputs data for the `Raw` `OutputMode`,
pub struct RawOutputter<W: Write> {
    /// Buffered file writer to write data into
    writer: W,
}

impl<W: Write> RawOutputter<W> {
    /// Create a new `RawOutputter` from options in an `OutputConfig` and `SimConfig`  
    ///
    /// Writes header data to the underlying `writer`
    pub fn new(mut writer: W, sim_cfg: &SimConfig) -> Result<Self, Box<dyn Error>> {
        initialize_output(&mut writer, sim_cfg, OutputMode::Raw, "")?;
        Ok(Self { writer })
    }

    /// Output the raw data in `Lineages`
    pub fn record_lineages(
        &mut self,
        r: u32,
        t: u32,
        lineages: &LineagesData,
    ) -> Result<(), Box<dyn Error>> {
        let record = LineagesRecord { r, t, lineages };
        serde_json::to_writer(&mut self.writer, &record)?;
        // Separate from next record to be written
        writeln!(&mut self.writer)?;

        Ok(())
    }

    pub fn into_inner(self) -> W {
        self.writer
    }
}

/// Type which outputs data for the `Summary` `OutputMode`,
pub struct SummaryOutputter<W: Write> {
    /// Buffered csv file writer to write data into
    writer: csv::Writer<W>,
    /// What summary stats to output
    cfg: SummaryOutputConfig,
}

/// Create helper methods to get rid of repetitive typing of operations on stats in the SummaryOutputter methods
///
/// Using this as a single macro with functions rather than separate macros ensures the order of the stats is consistent,
/// which we need it to be
macro_rules! create_summary_stats_helpers {
    ($($stat:ident),+) => {
        /// Push labels for enabled stats to the end of headers in proper order
        fn push_enabled_stat_headers(cfg: &SummaryOutputConfig, headers: &mut Vec<&str>) {
            $(
                if cfg.$stat {
                    headers.push(stringify!($stat));
                }
            )+
        }

        /// Write the CSV fields for enabled stats in proper order
        fn write_enabled_stat_fields(&mut self, data: &LineagesData) -> Result<(), Box<dyn Error>> {
            $(
                if self.cfg.$stat {
                    self.writer.write_field(format!("{}", summarize::$stat(data)))?;
                }
            )+

            Ok(())
        }
    }
}

impl<W: Write> SummaryOutputter<W> {
    create_summary_stats_helpers! {
        marker_1_ratio,
        stdev_W,
        max_W,
        stdev_accumulated_muts,
        max_accumulated_muts,
        genotype_count,
        shannon_diversity
    }

    /// Create a new `SummaryOutputter` from options in an `OutputConfig` and `SimConfig`  
    ///
    /// Writes header data to the underlying `writer`
    pub fn new(
        writer: W,
        summary_cfg: SummaryOutputConfig,
        sim_cfg: &SimConfig,
    ) -> Result<Self, Box<dyn Error>> {
        let mut writer = initialize_output_as_csv(writer, sim_cfg, OutputMode::Summary)?;

        // Header must be done manually for how we handle the output
        let mut header = vec!["replicate", "transfer", "mean_fitness"];
        Self::push_enabled_stat_headers(&summary_cfg, &mut header);
        writer.write_record(header)?;

        Ok(Self {
            writer,
            cfg: summary_cfg,
        })
    }

    /// Output summary data for `Lineages`
    pub fn record_lineages(
        &mut self,
        r: u32,
        t: u32,
        lineages: &LineagesData,
    ) -> Result<(), Box<dyn Error>> {
        #![allow(non_snake_case)]

        let avg_W = summarize::sum_N_and_avg_W(lineages).1;
        self.writer.write_field(r.to_string())?;
        self.writer.write_field(t.to_string())?;
        self.writer.write_field(avg_W.to_string())?;

        self.write_enabled_stat_fields(lineages)?;

        self.writer.write_record(EMPTY_CSV_RECORD)?;

        Ok(())
    }

    pub fn into_inner(self) -> Result<W, csv::IntoInnerError<csv::Writer<W>>> {
        self.writer.into_inner()
    }
}

/// Type which outputs data for the `Sequencing` `OutputMode`,
///
/// Mutations can be recorded with `record_pruned_mutations` and `record_active_mutations`,
/// replicates must be ended with `deliminate_replicate_end`
pub struct SequencingOutputter<W: Write> {
    /// Buffered file writer to write data into
    writer: W,
}

impl<W: Write> SequencingOutputter<W> {
    /// Create a new `SequencingOutputter` from options in an `OutputConfig` and `SimConfig`  
    ///
    /// Writes header data to the underlying `writer`
    pub fn new(mut writer: W, sim_cfg: &SimConfig) -> Result<Self, Box<dyn Error>> {
        initialize_output(&mut writer, sim_cfg, OutputMode::Sequencing, "")?;

        Ok(Self { writer })
    }

    /// Record mutations in a `MutationsData` which have been pruned
    pub fn record_pruned_mutations(
        &mut self,
        mutations: &MutationsData,
    ) -> Result<(), Box<dyn Error>> {
        for mutation in mutations.pruned_muts.iter() {
            self.record_mutation(mutation)?;
        }

        Ok(())
    }

    /// Record mutations in a `MutationsData` which are still being tracked and have not been pruned
    pub fn record_active_mutations(
        &mut self,
        mutations: &MutationsData,
    ) -> Result<(), Box<dyn Error>> {
        for mutation in mutations.muts.values() {
            self.record_mutation(mutation)?;
        }

        Ok(())
    }

    /// Record an individual `Mutation`
    fn record_mutation(&mut self, mutation: &Mutation) -> Result<(), Box<dyn Error>> {
        serde_json::to_writer(&mut self.writer, mutation)?;
        writeln!(&mut self.writer)?;
        Ok(())
    }

    /// Deliminate the end of a replicate
    ///
    /// Currently, this writes an extra newline character to the output
    pub fn deliminate_replicate_end(&mut self) -> Result<(), Box<dyn Error>> {
        writeln!(&mut self.writer)?;
        Ok(())
    }

    pub fn into_inner(self) -> W {
        self.writer
    }
}

/// Type which outputs data for the `MutationSummary` `OutputMode`
pub struct MutationSummaryOutputter<W: Write> {
    /// Buffered csv file writer to write data into
    writer: csv::Writer<W>,
}

impl<W: Write> MutationSummaryOutputter<W> {
    /// Create a new `MutationSummaryOutputter` from options in an `OutputConfig` and `SimConfig`  
    ///
    /// Writes header data to the underlying `writer`
    pub fn new(writer: W, sim_cfg: &SimConfig) -> Result<Self, Box<dyn Error>> {
        let mut writer = initialize_output_as_csv(writer, sim_cfg, OutputMode::MutationSummary)?;

        // Header must be done manually for how we handle the output
        let header = vec!["replicate", "transfer", "ID", "N"];
        writer.write_record(header)?;

        Ok(Self { writer })
    }

    /// Record mutations in a `MutationsData` which have been pruned
    pub fn record_pruned_mutations(
        &mut self,
        r: u32,
        mutations: &MutationsData,
    ) -> Result<(), Box<dyn Error>> {
        for mutation in mutations.pruned_muts.iter() {
            self.record_mutation(r, mutation)?;
        }

        Ok(())
    }

    /// Record mutations in a `MutationsData` which are still being tracked and have not been pruned
    pub fn record_active_mutations(
        &mut self,
        r: u32,
        mutations: &MutationsData,
    ) -> Result<(), Box<dyn Error>> {
        for mutation in mutations.muts.values() {
            self.record_mutation(r, mutation)?;
        }

        Ok(())
    }

    /// Record an individual `Mutation`
    fn record_mutation(&mut self, r: u32, mutation: &Mutation) -> Result<(), Box<dyn Error>> {
        for (i, n) in mutation.N().iter().enumerate() {
            self.writer
                .serialize((r, mutation.first_transfer() + i as u32, mutation.id(), *n))?;
        }

        Ok(())
    }

    pub fn into_inner(self) -> Result<W, csv::IntoInnerError<csv::Writer<W>>> {
        self.writer.into_inner()
    }
}

/// Create a buffered `File` to use
fn create_buffered_file<P: AsRef<Path>>(path: P) -> io::Result<BufWriter<File>> {
    Ok(BufWriter::with_capacity(
        BUFFER_CAPACITY,
        File::create(path)?,
    ))
}

/// Create a file to output simulation results and a variably sized buffered writer for it  
/// while outputting `Metadata` and `SimConfig` options into header at the top of the file
///
/// Allow an optional prefix for lines of the header (e.g. for comments)
fn initialize_output<W: Write>(
    writer: &mut W,
    sim_cfg: &SimConfig,
    output_mode: OutputMode,
    header_prefix: &'static str,
) -> Result<(), Box<dyn Error>> {
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

fn initialize_output_as_csv<W: Write>(
    mut writer: W,
    sim_cfg: &SimConfig,
    output_mode: OutputMode,
) -> Result<csv::Writer<W>, Box<dyn Error>> {
    initialize_output(&mut writer, sim_cfg, output_mode, "# ")?;

    // TODO: Decide what to do about buffering situation
    Ok(csv::WriterBuilder::new()
        .buffer_capacity(BUFFER_CAPACITY)
        .from_writer(writer))
}

const EMPTY_CSV_RECORD: [&[u8]; 0] = [];

/// An error originating from processing a previous output file for reproduction of results  
#[derive(Debug)]
pub enum MetadataError {
    IncompatibleVersion { version: String },
    MissingHeaders,
}

impl std::fmt::Display for MetadataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetadataError::IncompatibleVersion { version } => write!(
                f,
                "Previous results from incompatible simulation version {}",
                &version
            ),
            MetadataError::MissingHeaders => {
                write!(f, "Cannot find headers in input file to reproduce with")
            }
        }
    }
}

impl Error for MetadataError {}

/// Get the `SimConfig` encoded in a previous output file back out
///
/// Will fail if previous output is from a different version, in the future this  
/// may change (i.e. with SemVer)
pub fn extract_sim_config<P: AsRef<Path>>(path: P) -> Result<SimConfig, Box<dyn Error>> {
    Ok(extract_headers(path)?.sim_cfg)
}

/// Parts of the file after extracting headers
struct ExtractedHeaders {
    /// Metadata extracted from the file
    #[allow(dead_code)]
    metadata: Metadata,
    /// Simulation configuration extracted from the file
    sim_cfg: SimConfig,
    /// Remainder of file, in lines reader from which the BufReader or File can be extracted
    #[allow(dead_code)]
    remainder: Lines<BufReader<File>>,
}

/// Get the `Metadata` and `SimConfig` encoded in a previous output file back out
///
/// Will fail if previous output is from a different version, in the future this  
/// may change (i.e. with SemVer)
fn extract_headers<P: AsRef<Path>>(path: P) -> Result<ExtractedHeaders, Box<dyn Error>> {
    let file = File::open(path)?;
    // BufReader is required for `lines` iterator
    let reader = BufReader::with_capacity(HEADER_BUFFER_CAPACITY, file);
    let mut lines = reader.lines();

    // Make sure the metadata is present and version is correct
    // Strip comment characters
    let metadata: Metadata = match lines.next() {
        Some(line) => serde_json::from_str(line?.trim_start_matches("# "))?,
        None => return Err(MetadataError::MissingHeaders.into()),
    };

    if metadata.version != get_current_version_string() {
        return Err(MetadataError::IncompatibleVersion {
            version: (&metadata.version).to_owned(),
        }
        .into());
    }

    let mut sim_cfg: SimConfig = match lines.next() {
        Some(line) => serde_json::from_str(line?.trim_start_matches("# "))?,
        None => return Err(MetadataError::MissingHeaders.into()),
    };
    // Must finish initialization steps
    // Because not everything in SimConfig can be serialized
    sim_cfg.finish_initialization();

    Ok(ExtractedHeaders {
        metadata,
        sim_cfg,
        remainder: lines,
    })
}
