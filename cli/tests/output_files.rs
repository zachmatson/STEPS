//! End-to-end checks that a seeded run writes the expected bytes to each output file.
//!
//! The expected values are recorded from a known-good run, so a change to the stochastic
//! parts of the simulation will fail these and the constants below need updating. See
//! CONTRIBUTING.md.

use std::fs;
use std::path::Path;

use clap::Parser;
use tempfile::TempDir;

use steps_cli::{run_cli_config, CliConfig};

const VERSION: &str = env!("CARGO_PKG_VERSION");

const DESCRIPTION: &str = "STEPS simulation of bacterial evolution written by Devin Lake, Zachary Matson, and Richard Lenski";

const CONFIG_JSON: &str = r#"{"replicates":1,"transfers":2,"markers":1,"dilution_factor":8.0,"beneficial_mutation_rate":0.05,"neutral_mutation_rate":0.0,"deleterious_mutation_rate":0.0,"initial_beneficial_mutation_size":0.012,"fixed_deleterious_mutation_size":null,"diminishing_returns_epistasis_strength":6.0,"seed":42,"max_pop_size":80.0}"#;

const SUMMARY_BODY: &str = "\
replicate,transfer,avg_W,mean_accumulated_muts
1,0,1,0
1,1,1,0
1,2,1.0005281606272116,0.1
";

const RAW_BODY: &str = "\
[1,0,{\"N\":[10.0],\"W\":[1.0],\"U\":[0.05],\"secondary\":[[83.33333333333333,1,0,1,1]]}]
[1,1,{\"N\":[9.0],\"W\":[1.0],\"U\":[0.05],\"secondary\":[[83.33333333333333,1,0,1,1]]}]
[1,2,{\"N\":[9.0,1.0],\"W\":[1.0,1.0052816062721157],\"U\":[0.05,0.05],\"secondary\":[[83.33333333333333,1,0,1,1],[85.9741364693912,4,1,1,2]]}]
";

/// Sorted, because mutation records are emitted in hash map order
const SEQUENCING_LINES: [&str; 4] = [
    "[1,0,0.0,0.0,0,[10.0],1]",
    "[2,1,0.03234604724999546,0.0,1,[],1]",
    "[3,1,0.008009142592353857,0.0,2,[],1]",
    "[4,1,0.005281606272115713,0.0,2,[1.0],1]",
];

/// Sorted, as for `SEQUENCING_LINES`
const MUTATION_SUMMARY_LINES: [&str; 3] = ["1,0,1,10.0", "1,2,4,1.0", "replicate,transfer,ID,N"];

/// The two JSON header lines every output file starts with
fn expected_header(output_mode: &str, comment_prefix: &str) -> String {
    format!(
        "{comment_prefix}{{\"name\":\"STEPS\",\"version\":\"{VERSION}\",\"description\":\"{DESCRIPTION}\",\"output_mode\":\"{output_mode}\"}}\n\
         {comment_prefix}{CONFIG_JSON}\n"
    )
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap()
}

fn sorted_lines(contents: &str) -> Vec<&str> {
    let mut lines: Vec<&str> = contents.lines().collect();
    lines.sort_unstable();
    lines
}

/// Run a short seeded simulation writing all four output types, and return their paths
fn run_all_outputs(dir: &TempDir) -> [std::path::PathBuf; 4] {
    let paths = [
        "summary.csv",
        "raw.ndjson",
        "sequencing.ndjson",
        "mutations.csv",
    ]
    .map(|name| dir.path().join(name));

    let cfg = CliConfig::try_parse_from([
        "steps",
        "simulate",
        "-r",
        "1",
        "-t",
        "2",
        "-m",
        "1",
        "-D",
        "8",
        "--Nmax",
        "80",
        "--Ub",
        "0.05",
        "--seed",
        "42",
        "--summary-output",
        paths[0].to_str().unwrap(),
        "--raw-output",
        paths[1].to_str().unwrap(),
        "--sequencing-output",
        paths[2].to_str().unwrap(),
        "--mutation-summary-output",
        paths[3].to_str().unwrap(),
    ])
    .unwrap();

    run_cli_config(cfg);

    paths
}

#[test]
fn summary_output_matches_expected_bytes() {
    let dir = TempDir::new().unwrap();
    let paths = run_all_outputs(&dir);

    let expected = expected_header("Summary", "# ") + SUMMARY_BODY;
    assert_eq!(read(&paths[0]), expected);
}

#[test]
fn raw_output_matches_expected_bytes() {
    let dir = TempDir::new().unwrap();
    let paths = run_all_outputs(&dir);

    let expected = expected_header("Raw", "") + RAW_BODY;
    assert_eq!(read(&paths[1]), expected);
}

#[test]
fn sequencing_output_matches_expected_records() {
    let dir = TempDir::new().unwrap();
    let paths = run_all_outputs(&dir);

    let contents = read(&paths[2]);
    let header = expected_header("Sequencing", "");
    assert!(contents.starts_with(&header), "header did not match");

    assert_eq!(
        sorted_lines(&contents[header.len()..]),
        SEQUENCING_LINES.to_vec()
    );
}

#[test]
fn mutation_summary_output_matches_expected_records() {
    let dir = TempDir::new().unwrap();
    let paths = run_all_outputs(&dir);

    let contents = read(&paths[3]);
    let header = expected_header("MutationSummary", "# ");
    assert!(contents.starts_with(&header), "header did not match");

    assert_eq!(
        sorted_lines(&contents[header.len()..]),
        MUTATION_SUMMARY_LINES.to_vec()
    );
}
