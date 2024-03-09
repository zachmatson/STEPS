//! Tools for parsing information inputted from a past STEPS output

use std::io::{BufRead, BufReader, Lines, Read};

use anyhow::Result;
use thiserror::Error;

use crate::cfg::SimConfig;

use crate::io::{get_current_version_str, Metadata};

/// Get the `SimConfig` encoded in a previous output back out
///
/// Will fail if previous output is from a different version, in the future this may change to allow
/// for backward compatibility (i.e. using SemVer)
pub fn extract_sim_config<R: Read>(source: R) -> Result<SimConfig> {
    Ok(extract_headers(source)?.sim_cfg)
}

/// Get the `Metadata` and `SimConfig` encoded in a previous file back out
///
/// Will fail if previous output is from a different version, in the future this may change
/// (i.e. with SemVer)
fn extract_headers<R: Read>(source: R) -> Result<ExtractedHeaders<R>> {
    // BufReader is required for `lines` iterator
    let reader = BufReader::with_capacity(HEADER_BUFFER_CAPACITY, source);
    let mut lines = reader.lines();

    // Make sure the metadata is present and version is correct
    // Strip comment characters
    let metadata: Metadata = match lines.next() {
        Some(line) => serde_json::from_str(line?.trim_start_matches("# "))?,
        None => return Err(MetadataError::MissingHeaders.into()),
    };

    if metadata.version != get_current_version_str() {
        return Err(MetadataError::IncompatibleVersion {
            version: metadata.version,
        }
        .into());
    }

    let sim_cfg: SimConfig = match lines.next() {
        Some(line) => serde_json::from_str(line?.trim_start_matches("# "))?,
        None => return Err(MetadataError::MissingHeaders.into()),
    };

    Ok(ExtractedHeaders {
        metadata,
        sim_cfg,
        remainder: lines,
    })
}

/// Parts of the file after extracting headers
struct ExtractedHeaders<R: Read> {
    /// Metadata extracted from the file
    #[allow(dead_code)]
    metadata: Metadata,
    /// Simulation configuration extracted from the file
    sim_cfg: SimConfig,
    /// Remainder of file, in lines reader from which the BufReader or inner reader can be extracted
    #[allow(dead_code)]
    remainder: Lines<BufReader<R>>,
}

/// Buffer capacity for writing/reading header
///
/// Set at 2 KB
const HEADER_BUFFER_CAPACITY: usize = 2 * (1 << 10);

/// An error originating from processing a previous output file for reproduction of results  
#[derive(Error, Debug)]
enum MetadataError {
    /// Attempted to load metadata from an incompatible simulation version
    #[error("Input file is from an incompatible simulation version: {version}")]
    IncompatibleVersion {
        /// Version number for the incompatible found version
        version: String,
    },
    /// Attempted to load metadata from a file which is missing STEPS output headers
    #[error("Input file is missing the necessary headers to extract simulation options from")]
    MissingHeaders,
}
