//! Types to handle the output of simulation data and retrieval of encoded metadata and configuration
//! settings

use serde::{Deserialize, Serialize};

mod input_parsing;
mod output;

pub use input_parsing::extract_sim_config;
pub use output::{
    LineagesOutputter, MutationSummaryOutputter, MutationsOutputter, OutputterGroup,
    OutputterGroupBuilder, RawOutputter, SequencingOutputter, SummaryOutputter,
};

/// Type of output to produce
#[derive(Serialize, Deserialize, Copy, Clone)]
enum OutputMode {
    /// Full lineage data for each lineage, as ndjson
    Raw,
    /// Population summary information only, as CSV
    Summary,
    /// Full data about each mutation that occurs, as ndjson
    Sequencing,
    /// Summary information about mutations, as CSV
    MutationSummary,
}

/// Information used to mark output files as having been created by a specific version of STEPS
#[derive(Serialize, Deserialize)]
struct Metadata {
    name: String,
    version: String,
    description: String,
    output_mode: OutputMode,
}

impl Metadata {
    /// Construct a new `Metadata` instance based on the current version of the code and the desired
    /// `OutputMode`
    fn new(output_mode: OutputMode) -> Self {
        Self {
            name: "STEPS".to_string(),
            version: get_current_version_str().to_string(),
            description:
                "STEPS simulation of bacterial evolution written by Devin Lake and Zachary Matson"
                    .to_string(),
            output_mode,
        }
    }
}

/// Get the current version of STEPS as defined in Cargo.toml
fn get_current_version_str() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
