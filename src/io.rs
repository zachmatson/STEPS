//! Types to handle the output of simulation data and retrieval of encoded
//! metadata and configuration settings

use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::{
    cfg::{OuputConfig, SimConfig},
    sim::Lineages,
};

/// Type which handles the details of outputting simulation results
///
/// Must call `start_replicate` before each replicate, including the first one  
/// Then use `handle_lineages` to output the information in a `Lineages` instance as needed
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
}

impl OutputHandler {
    /// Create a new `OutputHandler` from options in an `OutputConfig` and `SimConfig`
    pub fn new(output_cfg: &OuputConfig, sim_cfg: &SimConfig) -> Result<Self, Box<dyn Error>> {
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

        Ok(Self {
            sampling_frequency: sim_cfg.sampling_frequency,
            raw_outputter,
            summary_outputter,
        })
    }

    /// Prepare the output for results from a new replicate
    ///
    /// Must also call `finish_replicate` after the replicate is done
    pub fn start_replicate(&mut self) -> Result<(), std::io::Error> {
        if let Some(raw_outputter) = &mut self.raw_outputter {
            raw_outputter.start_replicate()?;
        }

        Ok(())
    }

    /// Clean up after outputting the results from a replicate
    ///
    /// Must also call `start_replicate` after this to prepare for the next replicate,
    /// if another will be outputted
    pub fn finish_replicate(&mut self) -> Result<(), std::io::Error> {
        if let Some(raw_outputter) = &mut self.raw_outputter {
            raw_outputter.finish_replicate()?;
        }

        Ok(())
    }

    /// Output information from `Lineages` as necessary
    #[inline(always)]
    pub fn handle_lineages(
        &mut self,
        r: u32,
        t: u32,
        lineages: &Lineages,
    ) -> Result<(), Box<dyn Error>> {
        // Only output if at the sampling frequency
        if t % self.sampling_frequency != 0 {
            return Ok(());
        }

        if let Some(raw_outputter) = &mut self.raw_outputter {
            raw_outputter.record_lineages(lineages)?;
        }

        if let Some(summary_outputter) = &mut self.summary_outputter {
            summary_outputter.record_lineages(r, t, lineages)?;
        }

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

/// Buffer capacity to use in outputs  
/// Set at 8 MB
const BUFFER_CAPACITY: usize = 8_388_608;
/// Buffer capacity for writing/reading header
/// Set at 2 KB
const HEADER_BUFFER_CAPACITY: usize = 2_048;

/// Type which outputs data for the `Raw` `OutputMode`,
/// including owning the file handle for the output
///
/// Must call `start_replicate` before each replicate and
/// `finish_replicate` after each replicate, including the
/// first and last replicates
struct RawOutputter {
    /// Buffered file writer to write data into
    buf: Option<BufWriter<File>>,
}

impl RawOutputter {
    /// Create a new `RawOutputter` from options in an `OutputConfig` and `SimConfig`  
    ///
    /// Allocates internal buffer and obtains file handle
    fn initialize(output_cfg: &OuputConfig, sim_cfg: &SimConfig) -> Result<Self, Box<dyn Error>> {
        let buf = Some(create_file_with_header(
            output_cfg.raw_output_path.as_ref().unwrap(),
            sim_cfg,
            OutputMode::Raw,
            "",
            BUFFER_CAPACITY,
        )?);

        Ok(Self { buf })
    }

    /// Get a mutable reference to the internal buffer
    ///
    /// Panics if the buffer does not exist
    fn buf(&mut self) -> &mut BufWriter<File> {
        self.buf.as_mut().unwrap()
    }

    /// Prepare the outputter for results from a new replicate
    ///
    /// Must also call `finish_replicate` after the replicate is done
    fn start_replicate(&mut self) -> Result<(), std::io::Error> {
        // Sets up JSON sequence type
        write!(self.buf(), "[")?;
        Ok(())
    }

    /// Clean up after outputting the results from a replicate
    ///
    /// Must also call `start_replicate` after this to prepare for the next replicate,
    /// if another will be outputted
    fn finish_replicate(&mut self) -> Result<(), std::io::Error> {
        // Ends JSON sequence type
        writeln!(self.buf(), "]")?;
        Ok(())
    }

    /// Output the raw data in `Lineages`
    fn record_lineages(&mut self, lineages: &Lineages) -> Result<(), Box<dyn Error>> {
        serde_json::to_writer(self.buf(), lineages)?;
        // Separate from next `Lineages` to be written
        write!(self.buf(), ",")?;
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
    fn initialize(output_cfg: &OuputConfig, sim_cfg: &SimConfig) -> Result<Self, Box<dyn Error>> {
        let buf = create_file_with_header(
            output_cfg.summary_output_path.as_ref().unwrap(),
            sim_cfg,
            OutputMode::Summary,
            "# ",
            HEADER_BUFFER_CAPACITY,
        )?;

        // Release the buffer contents and get file handle back to give to CSV writer
        // Because the csv::Writer already buffers
        let file = buf.into_inner()?;
        let mut wtr = csv::Writer::from_writer(file);

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
        lineages: &Lineages,
    ) -> Result<(), Box<dyn Error>> {
        if self.needs_ratio {
            self.wtr
                .serialize((r, t, lineages.avg_W(), lineages.marker_1_ratio()))?;
        } else {
            self.wtr.serialize((r, t, lineages.avg_W()))?;
        }

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

/// Get the `SimConfig` encoded in a previous output file back out
///
/// Will fail if previous output is from a different version, in the future this  
/// may change (i.e. with SemVer)
pub fn extract_sim_config<P: AsRef<Path>>(path: P) -> Result<SimConfig, Box<dyn Error>> {
    let file = File::open(path)?;
    // BufReader is required for `lines` iterator
    let reader = BufReader::with_capacity(HEADER_BUFFER_CAPACITY, file);
    // Map the lines to remove possible comment characters
    let mut lines = reader
        .lines()
        .map(|line| line.unwrap().trim_start_matches("# ").to_string());

    // Make sure the metadata is present and version is correct
    let metadata: Metadata = match &lines.next() {
        Some(line) => serde_json::from_str(line)?,
        None => Err(ReproductionError::MissingHeaders)?,
    };

    if metadata.version != env!("CARGO_PKG_VERSION") {
        Err(ReproductionError::IncompatibleVersion {
            version: metadata.version,
        })?;
    }

    let mut sim_cfg: SimConfig = match &lines.next() {
        Some(line) => serde_json::from_str(line)?,
        None => Err(ReproductionError::MissingHeaders)?,
    };
    // Must finish initialization steps
    // Because not everything in SimConfig can be serialized
    sim_cfg.finish_initialization();

    Ok(sim_cfg)
}

/// An error originating from processing a previous output file for reproduction of results  
#[derive(Debug)]
pub enum ReproductionError {
    IncompatibleVersion { version: String },
    MissingHeaders,
}

impl std::fmt::Display for ReproductionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReproductionError::IncompatibleVersion { version } => write!(
                f,
                "Previous results from incompatible simulation version {}",
                &version
            ),
            ReproductionError::MissingHeaders => {
                write!(f, "Cannot find headers in input file to reproduce with")
            }
        }
    }
}

impl Error for ReproductionError {}
