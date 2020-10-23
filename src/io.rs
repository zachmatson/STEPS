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
    summary_outputter: Option<SummaryOutputer>,
}

impl OutputHandler {
    pub fn new(output_cfg: &OuputConfig, sim_cfg: &SimConfig) -> Self {
        let raw_outputter = output_cfg
            .raw_output_path
            .as_ref()
            .map(|_| RawOutputter::initialize(output_cfg, sim_cfg));

        let summary_outputter = output_cfg
            .summary_output_path
            .as_ref()
            .map(|_| SummaryOutputer::initialize(output_cfg, sim_cfg));

        Self {
            sampling_frequency: sim_cfg.sampling_frequency,
            raw_outputter,
            summary_outputter,
        }
    }

    pub fn start_replicate(&mut self) {
        if let Some(raw_outputter) = &mut self.raw_outputter {
            raw_outputter.start_replicate();
        }
    }

    pub fn finish_replicate(&mut self) {
        if let Some(raw_outputter) = &mut self.raw_outputter {
            raw_outputter.finish_replicate();
        }
    }

    #[inline(always)]
    pub fn handle_lineages(&mut self, r: u32, t: u32, lineages: &Lineages) {
        // Only output if at the sampling frequency
        if t % self.sampling_frequency != 0 {
            return;
        }

        if let Some(raw_outputter) = &mut self.raw_outputter {
            raw_outputter.record_lineages(lineages);
        }

        if let Some(summary_outputter) = &mut self.summary_outputter {
            summary_outputter.record_lineages(r, t, lineages);
        }
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
    #![allow(unused_must_use)]
    fn initialize(output_cfg: &OuputConfig, sim_cfg: &SimConfig) -> Self {
        let buf = Some(create_file_with_header(
            output_cfg.raw_output_path.as_ref().unwrap(),
            sim_cfg,
            OutputMode::Raw,
            "",
            BUFFER_CAPACITY,
        ));

        Self { buf }
    }

    fn buf(&mut self) -> &mut BufWriter<File> {
        self.buf.as_mut().unwrap()
    }

    fn start_replicate(&mut self) {
        write!(self.buf(), "[");
    }

    fn finish_replicate(&mut self) {
        writeln!(self.buf(), "]");
    }

    fn record_lineages(&mut self, lineages: &Lineages) {
        serde_json::to_writer(self.buf(), lineages);
        write!(self.buf(), ",");
    }
}

struct SummaryOutputer {
    wtr: csv::Writer<File>,
    needs_ratio: bool,
}

impl SummaryOutputer {
    #![allow(unused_must_use)]
    fn initialize(output_cfg: &OuputConfig, sim_cfg: &SimConfig) -> Self {
        let buf = create_file_with_header(
            output_cfg.summary_output_path.as_ref().unwrap(),
            sim_cfg,
            OutputMode::Summary,
            "# ",
            HEADER_BUFFER_CAPACITY,
        );

        // Release the buffer contents and get file handle back to give to CSV writer
        let file = buf.into_inner().unwrap();
        let mut wtr = csv::Writer::from_writer(file);

        let needs_ratio = sim_cfg.markers == 2;

        // Write header
        if needs_ratio {
            wtr.write_record(&["replicate", "transfer", "mean_fitness", "marker_ratio"]);
        } else {
            wtr.write_record(&["replicate", "transfer", "mean_fitness"]);
        }

        Self { wtr, needs_ratio }
    }

    fn record_lineages(&mut self, r: u32, t: u32, lineages: &Lineages) {
        if self.needs_ratio {
            self.wtr
                .serialize((r, t, lineages.avg_W(), lineages.marker_1_ratio()));
        } else {
            self.wtr.serialize((r, t, lineages.avg_W()));
        }
    }
}

#[allow(unused_must_use)]
fn create_file_with_header<P: AsRef<Path>>(
    path: P,
    sim_cfg: &SimConfig,
    output_mode: OutputMode,
    header_prefix: &'static str,
    buffer_capacity: usize,
) -> BufWriter<File> {
    let file = File::create(path).unwrap();
    let mut buf = BufWriter::with_capacity(buffer_capacity, file);

    // Write the metadata to the file with optional comment character
    write!(&mut buf, "{}", header_prefix);
    let metadata = Metadata::new(output_mode);
    serde_json::to_writer(&mut buf, &metadata);
    writeln!(&mut buf);

    // Write the simulation configuration to the file with optional comment character
    write!(&mut buf, "{}", header_prefix);
    serde_json::to_writer(&mut buf, sim_cfg);
    writeln!(&mut buf);

    buf
}

pub fn extract_sim_config<P: AsRef<Path>>(path: P) -> SimConfig {
    let file = File::open(path).unwrap();
    let reader = BufReader::with_capacity(HEADER_BUFFER_CAPACITY, file);
    let mut lines = reader
        .lines()
        .map(|line| line.unwrap().trim_start_matches("# ").to_string());

    // Read metadata
    let metadata: Metadata = serde_json::from_str(&lines.next().unwrap()).unwrap();
    // Make sure the version is correct
    assert_eq!(metadata.version, "0.1.0");

    // Read config
    let mut sim_cfg: SimConfig = serde_json::from_str(&lines.next().unwrap()).unwrap();
    sim_cfg.finish_initialization();

    sim_cfg
}
