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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn make_valid_header(sim_cfg: &crate::cfg::SimConfig) -> String {
        let metadata = Metadata::new(crate::io::OutputMode::Summary);
        let meta_json = serde_json::to_string(&metadata).unwrap();
        let cfg_json = serde_json::to_string(sim_cfg).unwrap();
        format!("# {}\n# {}\n", meta_json, cfg_json)
    }

    fn default_sim_config() -> crate::cfg::SimConfig {
        crate::cfg::SimConfig {
            replicates: 5,
            transfers: 100,
            markers: 2,
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

    #[test]
    fn test_extract_sim_config_roundtrip() {
        let cfg = default_sim_config();
        let header = make_valid_header(&cfg);
        let cursor = Cursor::new(header);
        let extracted = extract_sim_config(cursor).unwrap();
        assert_eq!(extracted.replicates, 5);
        assert_eq!(extracted.transfers, 100);
        assert_eq!(extracted.markers, 2);
        assert_eq!(extracted.dilution_factor, 100.0);
        assert_eq!(extracted.seed, Some(42));
        assert_eq!(extracted.max_pop_size, 5e8);
    }

    #[test]
    fn test_extract_sim_config_preserves_mutation_rates() {
        let mut cfg = default_sim_config();
        cfg.beneficial_mutation_rate = 0.005;
        cfg.neutral_mutation_rate = 0.001;
        cfg.deleterious_mutation_rate = 0.002;
        let header = make_valid_header(&cfg);
        let cursor = Cursor::new(header);
        let extracted = extract_sim_config(cursor).unwrap();
        assert_eq!(extracted.beneficial_mutation_rate, 0.005);
        assert_eq!(extracted.neutral_mutation_rate, 0.001);
        assert_eq!(extracted.deleterious_mutation_rate, 0.002);
    }

    #[test]
    fn test_extract_sim_config_no_seed() {
        let mut cfg = default_sim_config();
        cfg.seed = None;
        let header = make_valid_header(&cfg);
        let cursor = Cursor::new(header);
        let extracted = extract_sim_config(cursor).unwrap();
        assert_eq!(extracted.seed, None);
    }

    #[test]
    fn test_extract_sim_config_empty_input_fails() {
        let cursor = Cursor::new("");
        let result = extract_sim_config(cursor);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_sim_config_wrong_version_fails() {
        let meta_json = serde_json::to_string(&serde_json::json!({
            "name": "STEPS",
            "version": "0.0.0",
            "description": "test",
            "output_mode": "Summary"
        }))
        .unwrap();
        let cfg_json = serde_json::to_string(&default_sim_config()).unwrap();
        let header = format!("# {}\n# {}\n", meta_json, cfg_json);
        let cursor = Cursor::new(header);
        let result = extract_sim_config(cursor);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_sim_config_missing_second_line_fails() {
        let metadata = Metadata::new(crate::io::OutputMode::Summary);
        let meta_json = serde_json::to_string(&metadata).unwrap();
        let header = format!("# {}\n", meta_json);
        let cursor = Cursor::new(header);
        let result = extract_sim_config(cursor);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_sim_config_invalid_json_fails() {
        let metadata = Metadata::new(crate::io::OutputMode::Summary);
        let meta_json = serde_json::to_string(&metadata).unwrap();
        let header = format!("# {}\n# not valid json\n", meta_json);
        let cursor = Cursor::new(header);
        let result = extract_sim_config(cursor);
        assert!(result.is_err());
    }
}
