//! Types to handle the output of simulation data and retrieval of encoded
//! metadata and configuration settings

use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Lines, Write};
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_tuple::*;

use crate::{
    cfg::{OutputConfig, SimConfig},
    sim::{self, LineagesData, Mutation, MutationsData},
};

/// Type which handles the details of outputting simulation results
pub struct OutputHandler {
    /// Frequency at which to output results
    ///
    /// Outputs must be done when the transfer number is a multiple
    /// of `sampling_frequency`
    sampling_frequency: u32,

    /// Outputter for the raw output mode, if applicable
    raw_outputter: Option<RawOutputter>,
    /// Outputter for the summary output mode, if applicable
    summary_outputter: Option<SummaryOutputter>,
    /// Outputter for the sequencing output mode, if applicable
    sequencing_outputter: Option<SequencingOutputter>,
}

impl OutputHandler {
    /// Create a new `OutputHandler` from options in an `OutputConfig` and `SimConfig`
    pub fn new(output_cfg: &OutputConfig, sim_cfg: &SimConfig) -> Result<Self, Box<dyn Error>> {
        let raw_outputter = if output_cfg.raw_output_path.is_some() {
            Some(RawOutputter::initialize(output_cfg, sim_cfg)?)
        } else {
            None
        };

        let summary_outputter = if output_cfg.summary_output_path.is_some() {
            Some(SummaryOutputter::initialize(output_cfg, sim_cfg)?)
        } else {
            None
        };

        let sequencing_outputter = if output_cfg.sequencing_output_path.is_some() {
            Some(SequencingOutputter::initialize(output_cfg, sim_cfg)?)
        } else {
            None
        };

        Ok(Self {
            sampling_frequency: sim_cfg.sampling_frequency,
            raw_outputter,
            summary_outputter,
            sequencing_outputter,
        })
    }

    /// Output information from `lineages` as necessary
    #[inline(always)]
    pub fn handle_lineages_output(
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
        mutations_data: &MutationsData,
    ) -> Result<(), Box<dyn Error>> {
        self.sequencing_outputter
            .as_mut()
            .unwrap()
            .record_pruned_mutations(mutations_data)?;
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
        mutations_data: &MutationsData,
    ) -> Result<(), Box<dyn Error>> {
        self.output_pruned_mutations(mutations_data)?;
        let sequencing_outputter = self.sequencing_outputter.as_mut().unwrap();
        sequencing_outputter.record_active_mutations(mutations_data)?;
        sequencing_outputter.deliminate_replicate_end()?;
        Ok(())
    }
}

/// Type of output to produce
#[derive(Serialize, Deserialize)]
enum OutputMode {
    /// Full lineage data for each lineage, as ndjson
    Raw,
    /// Population summary information only, as CSV
    Summary,
    /// Information about each mutation that occurs, as ndjson
    Sequencing,
}

/// Get the current version of ReLLTEE as defined in Cargo.toml
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
            name: "ReLLTEE".to_string(),
            version: get_current_version_string(),
            description:
                "ReLLTEE simulation of bacterial evolution written by Devin Lake and Zachary Matson"
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

impl OwnedLineagesRecord {
    fn borrowed(&self) -> LineagesRecord {
        LineagesRecord {
            r: self.r,
            t: self.t,
            lineages: &self.lineages,
        }
    }
}

/// Buffer capacity to use in outputs  
/// Set at 8 MB
const BUFFER_CAPACITY: usize = 8 * (1 << 20);
/// Buffer capacity for writing/reading header
/// Set at 2 KB
const HEADER_BUFFER_CAPACITY: usize = 2 * (1 << 10);

/// Type which outputs data for the `Raw` `OutputMode`,
/// including owning the file handle for the output
struct RawOutputter {
    /// Buffered file writer to write data into
    buf: BufWriter<File>,
}

impl RawOutputter {
    /// Create a new `RawOutputter` from options in an `OutputConfig` and `SimConfig`  
    ///
    /// Allocates internal buffer and obtains file handle
    fn initialize(output_cfg: &OutputConfig, sim_cfg: &SimConfig) -> Result<Self, Box<dyn Error>> {
        let buf = create_file_with_header(
            output_cfg.raw_output_path.as_ref().unwrap(),
            sim_cfg,
            OutputMode::Raw,
            "",
            BUFFER_CAPACITY,
        )?;

        Ok(Self { buf })
    }

    /// Output the raw data in `Lineages`
    fn record_lineages(
        &mut self,
        r: u32,
        t: u32,
        lineages: &LineagesData,
    ) -> Result<(), Box<dyn Error>> {
        let record = LineagesRecord { r, t, lineages };
        serde_json::to_writer(&mut self.buf, &record)?;
        // Separate from next record to be written
        writeln!(&mut self.buf)?;

        Ok(())
    }
}

/// Type which outputs data for the `Summary` `OutputMode`,
/// including owning the file handle for the output
struct SummaryOutputter {
    /// Buffered file writer to write data into
    wtr: csv::Writer<File>,
    /// Whether marker ratios should be outputted
    needs_ratio: bool,
}

impl SummaryOutputter {
    /// Create a new `SummaryOutputter` from options in an `OutputConfig` and `SimConfig`  
    ///
    /// Allocates internal buffer and obtains file handle
    fn initialize(output_cfg: &OutputConfig, sim_cfg: &SimConfig) -> Result<Self, Box<dyn Error>> {
        let mut wtr = csv_writer_with_metadata(
            output_cfg.summary_output_path.as_ref().unwrap(),
            sim_cfg,
            OutputMode::Summary,
        )?;

        // Ratio output only makes sense for two marker scenario
        let needs_ratio = sim_cfg.markers == 2;

        // Header must be done manually
        if needs_ratio {
            wtr.write_record(&["replicate", "transfer", "mean_fitness", "marker_ratio"])?;
        } else {
            wtr.write_record(&["replicate", "transfer", "mean_fitness"])?;
        }

        Ok(Self { wtr, needs_ratio })
    }

    /// Output summary data for `Lineages`
    fn record_lineages(
        &mut self,
        r: u32,
        t: u32,
        lineages: &LineagesData,
    ) -> Result<(), Box<dyn Error>> {
        #![allow(non_snake_case)]
        if self.needs_ratio {
            let (marker_1_ratio, avg_W) = sim::marker_1_ratio_and_avg_W(&lineages);
            self.wtr.serialize((r, t, avg_W, marker_1_ratio))?;
        } else {
            let avg_W = sim::sum_N_and_avg_W(&lineages).1;
            self.wtr.serialize((r, t, avg_W))?;
        }

        Ok(())
    }
}

/// Type which outputs data for the `Sequencing` `OutputMode`,
/// including owning the file handle for the output
struct SequencingOutputter {
    /// Buffered file writer to write data into
    buf: BufWriter<File>,
}

impl SequencingOutputter {
    /// Create a new `SequencingOutputter` from options in an `OutputConfig` and `SimConfig`  
    ///
    /// Allocates internal buffer and obtains file handle
    fn initialize(output_cfg: &OutputConfig, sim_cfg: &SimConfig) -> Result<Self, Box<dyn Error>> {
        let buf = create_file_with_header(
            output_cfg.sequencing_output_path.as_ref().unwrap(),
            sim_cfg,
            OutputMode::Sequencing,
            "",
            BUFFER_CAPACITY,
        )?;

        Ok(Self { buf })
    }

    /// Record mutations in a `MutationsData` which have been pruned
    fn record_pruned_mutations(&mut self, mutations: &MutationsData) -> Result<(), Box<dyn Error>> {
        for mutation in mutations.pruned_muts.iter() {
            self.record_mutation(mutation)?;
        }

        Ok(())
    }

    /// Record mutations in a `MutationsData` which are still being tracked and have not been pruned
    fn record_active_mutations(&mut self, mutations: &MutationsData) -> Result<(), Box<dyn Error>> {
        for mutation in mutations.muts.values() {
            self.record_mutation(mutation)?;
        }

        Ok(())
    }

    /// Record an individual `Mutation`
    fn record_mutation(&mut self, mutation: &Mutation) -> Result<(), Box<dyn Error>> {
        serde_json::to_writer(&mut self.buf, mutation)?;
        writeln!(&mut self.buf)?;
        Ok(())
    }

    /// Deliminate the end of a replicate
    ///
    /// Currently, this writes an extra newline character to the output
    fn deliminate_replicate_end(&mut self) -> Result<(), Box<dyn Error>> {
        writeln!(&mut self.buf)?;
        Ok(())
    }
}

/// Create a file to output simulation results and a variably sized buffered writer for it  
/// while outputting `Metadata` and `SimConfig` options into header at the top of the file
///
/// Allow an optional prefix for lines of the header (e.g. for comments)
fn create_file_with_header<P: AsRef<Path>>(
    path: P,
    sim_cfg: &SimConfig,
    output_mode: OutputMode,
    header_prefix: &'static str,
    buffer_capacity: usize,
) -> Result<BufWriter<File>, Box<dyn Error>> {
    let file = File::create(path)?;
    let mut buf = BufWriter::with_capacity(buffer_capacity, file);

    // Write the metadata to the file with optional comment character
    write!(&mut buf, "{}", header_prefix)?;
    let metadata = Metadata::new(output_mode);
    serde_json::to_writer(&mut buf, &metadata)?;
    writeln!(&mut buf)?;

    // Write the simulation configuration to the file with optional comment character
    write!(&mut buf, "{}", header_prefix)?;
    serde_json::to_writer(&mut buf, sim_cfg)?;
    writeln!(&mut buf)?;

    Ok(buf)
}

/// Create a file to output simulation results and return a `csv::Writer` pointed to it
/// while outputting `Metadata` and `SimConfig` options into header at the top of the file
fn csv_writer_with_metadata<P: AsRef<Path>>(
    path: P,
    sim_cfg: &SimConfig,
    output_mode: OutputMode,
) -> Result<csv::Writer<File>, Box<dyn Error>> {
    let buf = create_file_with_header(path, sim_cfg, output_mode, "# ", HEADER_BUFFER_CAPACITY)?;

    // Release the buffer contents and get file handle back to give to CSV writer
    // Because the csv::Writer already buffers
    let file = buf.into_inner()?;
    let wtr = csv::WriterBuilder::new()
        .buffer_capacity(BUFFER_CAPACITY)
        .from_writer(file);

    Ok(wtr)
}

/// An error originating from processing a previous output file for reproduction of results  
#[derive(Debug)]
pub enum MetadataError {
    IncompatibleVersion { version: String },
    MissingHeaders,
    WrongOutputMode,
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
            MetadataError::WrongOutputMode => {
                write!(f, "The input file was produced with the wrong output mode")
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
    Ok(extract_headers(path)?.1)
}

/// Get the `Metadata` and `SimConfig` encoded in a previous output file back out
///
/// Will fail if previous output is from a different version, in the future this  
/// may change (i.e. with SemVer)
fn extract_headers<P: AsRef<Path>>(
    path: P,
) -> Result<(Metadata, SimConfig, Lines<BufReader<File>>), Box<dyn Error>> {
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

    if &metadata.version != env!("CARGO_PKG_VERSION") {
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

    Ok((metadata, sim_cfg, lines))
}

struct RawResultsReader {
    lines: Lines<BufReader<File>>,
}

impl RawResultsReader {
    fn new<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let (metadata, _, lines) = extract_headers(path)?;

        match metadata.output_mode {
            OutputMode::Raw => (),
            _ => return Err(MetadataError::WrongOutputMode.into()),
        }

        Ok(Self { lines })
    }

    fn deserialize_line(
        line: Result<String, std::io::Error>,
    ) -> Result<OwnedLineagesRecord, Box<dyn Error>> {
        Ok(serde_json::from_str(&line?)?)
    }
}

impl Iterator for RawResultsReader {
    type Item = Result<(u32, u32, LineagesData), Box<dyn Error>>;

    fn next(&mut self) -> Option<Self::Item> {
        let line = self.lines.next()?;
        let item = Self::deserialize_line(line).map(|record| (record.r, record.t, record.lineages));
        Some(item)
    }
}
