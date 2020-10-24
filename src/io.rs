use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::{
    cfg::{OuputConfig, SimConfig},
    sim::Lineages,
};

#[derive(Serialize, Deserialize)]
enum OutputMode {
    Raw,
    Summary,
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
    fn new(output_mode: OutputMode) -> Self {
        Self {
            name: "ReLLTEE".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description:
                "ReLLTEE simulation of bacterial evolution written by Devin Lake and Zachary Matson"
                    .to_string(),
            output_mode,
        }
    }
}

pub struct OutputHandler {
    sampling_frequency: u32,
    raw_outputter: Option<RawOutputter>,
    summary_outputter: Option<SummaryOutputter>,
}

impl OutputHandler {
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

    pub fn start_replicate(&mut self) -> Result<(), std::io::Error> {
        if let Some(raw_outputter) = &mut self.raw_outputter {
            raw_outputter.start_replicate()?;
        }

        Ok(())
    }

    pub fn finish_replicate(&mut self) -> Result<(), std::io::Error> {
        if let Some(raw_outputter) = &mut self.raw_outputter {
            raw_outputter.finish_replicate()?;
        }

        Ok(())
    }

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

/// Buffer capacity to use in outputs  
/// Set at 8 MB
const BUFFER_CAPACITY: usize = 8_388_608;
/// Buffer capacity for writing/reading header
/// Set at 2 KB
const HEADER_BUFFER_CAPACITY: usize = 2_048;

struct RawOutputter {
    buf: Option<BufWriter<File>>,
}

impl RawOutputter {
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

    fn buf(&mut self) -> &mut BufWriter<File> {
        self.buf.as_mut().unwrap()
    }

    fn start_replicate(&mut self) -> Result<(), std::io::Error> {
        write!(self.buf(), "[")?;
        Ok(())
    }

    fn finish_replicate(&mut self) -> Result<(), std::io::Error> {
        writeln!(self.buf(), "]")?;
        Ok(())
    }

    fn record_lineages(&mut self, lineages: &Lineages) -> Result<(), Box<dyn Error>> {
        serde_json::to_writer(self.buf(), lineages)?;
        write!(self.buf(), ",")?;
        Ok(())
    }
}

struct SummaryOutputter {
    wtr: csv::Writer<File>,
    needs_ratio: bool,
}

impl SummaryOutputter {
    fn initialize(output_cfg: &OuputConfig, sim_cfg: &SimConfig) -> Result<Self, Box<dyn Error>> {
        let buf = create_file_with_header(
            output_cfg.summary_output_path.as_ref().unwrap(),
            sim_cfg,
            OutputMode::Summary,
            "# ",
            HEADER_BUFFER_CAPACITY,
        )?;

        // Release the buffer contents and get file handle back to give to CSV writer
        let file = buf.into_inner()?;
        let mut wtr = csv::Writer::from_writer(file);

        let needs_ratio = sim_cfg.markers == 2;

        // Write header
        if needs_ratio {
            wtr.write_record(&["replicate", "transfer", "mean_fitness", "marker_ratio"])?;
        } else {
            wtr.write_record(&["replicate", "transfer", "mean_fitness"])?;
        }

        Ok(Self { wtr, needs_ratio })
    }

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

pub fn extract_sim_config<P: AsRef<Path>>(path: P) -> Result<SimConfig, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::with_capacity(HEADER_BUFFER_CAPACITY, file);
    let mut lines = reader
        .lines()
        .map(|line| line.unwrap().trim_start_matches("# ").to_string());

    // Read metadata
    let metadata: Metadata = match &lines.next() {
        Some(line) => serde_json::from_str(line)?,
        None => Err(ReproductionError::MissingHeaders)?,
    };
    // Make sure the version is correct
    if metadata.version != env!("CARGO_PKG_VERSION") {
        Err(ReproductionError::IncompatibleVersion {
            version: metadata.version,
        })?;
    }

    // Read config
    let mut sim_cfg: SimConfig = match &lines.next() {
        Some(line) => serde_json::from_str(line)?,
        None => Err(ReproductionError::MissingHeaders)?,
    };
    sim_cfg.finish_initialization();

    Ok(sim_cfg)
}

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
