//! Implementations of the individual outputters in STEPS

use std::io::Write;

use anyhow::Result;
use serde_tuple::Serialize_tuple;

use crate::cfg::{SimConfig, SummaryOutputConfig};
use crate::sim::{summarize, LineagesData, Mutation};

use crate::io::OutputMode;

use crate::io::output::{
    initialize_output, initialize_output_as_csv, LineagesOutputter, MutationsOutputter,
    EMPTY_CSV_RECORD,
};

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// SummaryOutputter
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

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
macro_rules! summary_lineages_outputter_create_stats_helpers {
    ($($stat:ident),+ $(,)?) => {
        impl<W: Write> SummaryOutputter<W> {
            /// Push labels for enabled stats to the end of headers in proper order
            fn push_enabled_stat_headers(cfg: &SummaryOutputConfig, headers: &mut Vec<&str>) {
                $(
                    if cfg.$stat {
                        headers.push(stringify!($stat));
                    }
                )+
            }

            /// Write the CSV fields for enabled stats in proper order
            fn write_enabled_stat_fields(&mut self, data: &LineagesData) -> Result<()> {
                $(
                    if self.cfg.$stat {
                        self.writer.write_field(format!("{}", summarize::$stat(data)))?;
                    }
                )+

                Ok(())
            }
        }

        // Verify that all available statistics are accounted for in the macro invocation
        // Struct isn't actually used for anything but all fields must be supplied
        const _: () = {
            SummaryOutputConfig {
                $($stat: false),+
            };
        };
    }
}

summary_lineages_outputter_create_stats_helpers! {
    avg_W,
    marker_1_ratio,
    stdev_W,
    max_W,
    stdev_accumulated_muts,
    max_accumulated_muts,
    mean_accumulated_muts,
    min_accumulated_muts,
    genotype_count,
    shannon_diversity,
}

impl<W: Write> SummaryOutputter<W> {
    /// Create a new `SummaryOutputter` from options in an `OutputConfig` and `SimConfig`
    ///
    /// Writes header data to the underlying `writer`
    pub fn new(writer: W, summary_cfg: SummaryOutputConfig, sim_cfg: &SimConfig) -> Result<Self> {
        let mut writer = initialize_output_as_csv(writer, sim_cfg, OutputMode::Summary)?;

        // Header must be done manually for how we handle the output
        let mut header = vec!["replicate", "transfer"];
        Self::push_enabled_stat_headers(&summary_cfg, &mut header);
        writer.write_record(header)?;

        Ok(Self {
            writer,
            cfg: summary_cfg,
        })
    }

    /// Consume the outputter and get back the underlying `writer`
    ///
    /// Will not necessarily flush the writer
    pub fn into_inner(self) -> Result<W, csv::IntoInnerError<csv::Writer<W>>> {
        self.writer.into_inner()
    }
}

impl<W: Write> LineagesOutputter for SummaryOutputter<W> {
    fn record_lineages(
        &mut self,
        replicate: u32,
        transfer: u32,
        lineages: &LineagesData,
    ) -> Result<()> {
        #![allow(non_snake_case)]

        self.writer.write_field(replicate.to_string())?;
        self.writer.write_field(transfer.to_string())?;

        self.write_enabled_stat_fields(lineages)?;

        self.writer.write_record(EMPTY_CSV_RECORD)?;

        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// MutationSummaryOutputter
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Type which outputs data for the `MutationSummary` `OutputMode`
pub struct MutationSummaryOutputter<W: Write> {
    /// CSV writer to write data into
    writer: csv::Writer<W>,
}

impl<W: Write> MutationSummaryOutputter<W> {
    /// Create a new `MutationSummaryOutputter` from options in an `OutputConfig` and `SimConfig`  
    ///
    /// Writes header data to the underlying `writer`
    pub fn new(writer: W, sim_cfg: &SimConfig) -> Result<Self> {
        let mut writer = initialize_output_as_csv(writer, sim_cfg, OutputMode::MutationSummary)?;

        // Header must be done manually for how we handle the output
        let header = vec!["replicate", "transfer", "ID", "N"];
        writer.write_record(header)?;

        Ok(Self { writer })
    }

    /// Consume the outputter and get back the underlying `writer`
    ///
    /// Will not necessarily flush the writer
    pub fn into_inner(self) -> Result<W, csv::IntoInnerError<csv::Writer<W>>> {
        self.writer.into_inner()
    }
}

impl<W: Write> MutationsOutputter for MutationSummaryOutputter<W> {
    fn record_mutation(&mut self, replicate: u32, mutation: &Mutation) -> Result<()> {
        for (i, n) in mutation.N.iter().enumerate() {
            self.writer.serialize((
                replicate,
                mutation.first_transfer + i as u32,
                mutation.id,
                *n,
            ))?;
        }

        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// RawOutputter
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Type which outputs data for the `Raw` `OutputMode`,
pub struct RawOutputter<W: Write> {
    /// Writer to write data into
    writer: W,
}

impl<W: Write> RawOutputter<W> {
    /// Create a new `RawOutputter` from options in an `OutputConfig` and `SimConfig`  
    ///
    /// Writes header data to the underlying `writer`
    pub fn new(mut writer: W, sim_cfg: &SimConfig) -> Result<Self> {
        initialize_output(&mut writer, sim_cfg, OutputMode::Raw, "")?;
        Ok(Self { writer })
    }

    /// Consume the outputter and get back the underlying `writer`
    ///
    /// Will not necessarily flush the writer
    pub fn into_inner(self) -> W {
        self.writer
    }
}

impl<W: Write> LineagesOutputter for RawOutputter<W> {
    fn record_lineages(
        &mut self,
        replicate: u32,
        transfer: u32,
        lineages: &LineagesData,
    ) -> Result<()> {
        let record = RawOutputterRecord {
            r: replicate,
            t: transfer,
            lineages,
        };
        serde_json::to_writer(&mut self.writer, &record)?;
        // Separate from next record to be written
        writeln!(&mut self.writer)?;

        Ok(())
    }
}

/// Record used by `RawOutputter` for serialization
#[derive(Serialize_tuple)]
struct RawOutputterRecord<'a> {
    /// Replicate
    r: u32,
    /// Transfer
    t: u32,
    /// Lineages
    lineages: &'a LineagesData,
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// SequencingOutputter
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Type which outputs data for the `Sequencing` `OutputMode`,
pub struct SequencingOutputter<W: Write> {
    /// Buffered file writer to write data into
    writer: W,
    /// Last replicate written
    last_replicate: u32,
}

impl<W: Write> SequencingOutputter<W> {
    /// Create a new `SequencingOutputter` from options in an `OutputConfig` and `SimConfig`  
    ///
    /// Writes header data to the underlying `writer`
    pub fn new(mut writer: W, sim_cfg: &SimConfig) -> Result<Self> {
        initialize_output(&mut writer, sim_cfg, OutputMode::Sequencing, "")?;

        Ok(Self {
            writer,
            last_replicate: 1,
        })
    }

    /// Consume the outputter and get back the underlying `writer`
    ///
    /// Will not necessarily flush the writer
    pub fn into_inner(self) -> W {
        self.writer
    }

    /// Deliminate the end of a replicate
    ///
    /// Currently, this writes an extra newline character to the output
    fn deliminate_replicate_end(&mut self) -> Result<()> {
        writeln!(&mut self.writer)?;
        Ok(())
    }
}

impl<W: Write> MutationsOutputter for SequencingOutputter<W> {
    fn record_mutation(&mut self, replicate: u32, mutation: &Mutation) -> Result<()> {
        if replicate != self.last_replicate {
            self.deliminate_replicate_end()?;
            self.last_replicate = replicate;
        }
        serde_json::to_writer(&mut self.writer, mutation)?;
        writeln!(&mut self.writer)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::output::tests::{all_stats_disabled, mutation, sim_cfg, two_lineages};

    /// Body lines of some output, with the two JSON header lines dropped
    fn body(output: &[u8]) -> Vec<String> {
        String::from_utf8(output.to_vec())
            .unwrap()
            .lines()
            .skip(2)
            .map(str::to_string)
            .collect()
    }

    #[test]
    fn test_summary_outputter_header_only_has_enabled_stats() {
        let cfg = SummaryOutputConfig {
            avg_W: true,
            max_W: true,
            genotype_count: true,
            ..all_stats_disabled()
        };
        let outputter = SummaryOutputter::new(Vec::new(), cfg, &sim_cfg()).unwrap();

        let output = outputter.into_inner().unwrap();
        assert_eq!(
            body(&output),
            ["replicate,transfer,avg_W,max_W,genotype_count"]
        );
    }

    #[test]
    fn test_summary_outputter_header_stats_are_in_macro_order() {
        // shannon_diversity is declared after genotype_count, so it must come second
        let cfg = SummaryOutputConfig {
            shannon_diversity: true,
            genotype_count: true,
            ..all_stats_disabled()
        };
        let outputter = SummaryOutputter::new(Vec::new(), cfg, &sim_cfg()).unwrap();

        let output = outputter.into_inner().unwrap();
        assert_eq!(
            body(&output),
            ["replicate,transfer,genotype_count,shannon_diversity"]
        );
    }

    #[test]
    fn test_summary_outputter_records_stat_values() {
        let cfg = SummaryOutputConfig {
            avg_W: true,
            max_W: true,
            genotype_count: true,
            ..all_stats_disabled()
        };
        let mut outputter = SummaryOutputter::new(Vec::new(), cfg, &sim_cfg()).unwrap();

        // N = [100, 100], W = [1.0, 2.0], so avg_W = 1.5, max_W = 2, genotype_count = 2
        outputter.record_lineages(3, 7, &two_lineages()).unwrap();

        let output = outputter.into_inner().unwrap();
        assert_eq!(body(&output)[1], "3,7,1.5,2,2");
    }

    #[test]
    fn test_summary_outputter_fields_match_header_count() {
        let cfg = SummaryOutputConfig {
            avg_W: true,
            stdev_W: true,
            max_W: true,
            ..all_stats_disabled()
        };
        let mut outputter = SummaryOutputter::new(Vec::new(), cfg, &sim_cfg()).unwrap();
        outputter.record_lineages(1, 0, &two_lineages()).unwrap();

        let output = outputter.into_inner().unwrap();
        let lines = body(&output);
        let header_fields = lines[0].split(',').count();
        let row_fields = lines[1].split(',').count();
        assert_eq!(header_fields, row_fields);
    }

    #[test]
    fn test_mutation_summary_outputter_writes_fixed_header() {
        let outputter = MutationSummaryOutputter::new(Vec::new(), &sim_cfg()).unwrap();

        let output = outputter.into_inner().unwrap();
        assert_eq!(body(&output), ["replicate,transfer,ID,N"]);
    }

    #[test]
    fn test_raw_outputter_writes_one_json_line_per_record() {
        let mut outputter = RawOutputter::new(Vec::new(), &sim_cfg()).unwrap();

        outputter.record_lineages(1, 0, &two_lineages()).unwrap();
        outputter.record_lineages(1, 1, &two_lineages()).unwrap();

        let output = outputter.into_inner();
        let lines = body(&output);
        assert_eq!(lines.len(), 2);

        // Each record is serialized as the tuple [replicate, transfer, lineages]
        for (line, expected_transfer) in lines.iter().zip([0, 1]) {
            let record: serde_json::Value = serde_json::from_str(line).unwrap();
            assert_eq!(record[0], 1);
            assert_eq!(record[1], expected_transfer);
            assert_eq!(record[2]["N"], serde_json::json!([100.0, 100.0]));
        }
    }

    #[test]
    fn test_raw_outputter_header_has_no_comment_prefix() {
        let outputter = RawOutputter::new(Vec::new(), &sim_cfg()).unwrap();

        let output = String::from_utf8(outputter.into_inner()).unwrap();
        // ndjson output must stay parseable, so the header lines are bare JSON
        for line in output.lines() {
            serde_json::from_str::<serde_json::Value>(line).unwrap();
        }
    }

    #[test]
    fn test_mutation_summary_outputter_writes_one_row_per_transfer() {
        let mut outputter = MutationSummaryOutputter::new(Vec::new(), &sim_cfg()).unwrap();

        // N covers three transfers starting at first_transfer, so transfers run 2, 3, 4
        outputter.record_mutation(1, &mutation(7, 2)).unwrap();

        let output = outputter.into_inner().unwrap();
        assert_eq!(
            body(&output)[1..],
            ["1,2,7,10.0", "1,3,7,20.0", "1,4,7,30.0"]
        );
    }

    #[test]
    fn test_mutation_summary_outputter_writes_nothing_for_empty_sizes() {
        let mut outputter = MutationSummaryOutputter::new(Vec::new(), &sim_cfg()).unwrap();

        let mut empty = mutation(7, 0);
        empty.N.clear();
        outputter.record_mutation(1, &empty).unwrap();

        let output = outputter.into_inner().unwrap();
        assert_eq!(body(&output).len(), 1, "only the header should be present");
    }

    #[test]
    fn test_sequencing_outputter_writes_one_json_line_per_mutation() {
        let mut outputter = SequencingOutputter::new(Vec::new(), &sim_cfg()).unwrap();

        outputter.record_mutation(1, &mutation(4, 0)).unwrap();
        outputter.record_mutation(1, &mutation(5, 0)).unwrap();

        let output = outputter.into_inner();
        let lines = body(&output);
        assert_eq!(lines.len(), 2);

        // Mutations are serialized as a tuple starting with the ID
        for (line, expected_id) in lines.iter().zip([4, 5]) {
            let record: serde_json::Value = serde_json::from_str(line).unwrap();
            assert_eq!(record[0], expected_id);
        }
    }

    #[test]
    fn test_sequencing_outputter_delimits_replicates_with_a_blank_line() {
        let mut outputter = SequencingOutputter::new(Vec::new(), &sim_cfg()).unwrap();

        outputter.record_mutation(1, &mutation(4, 0)).unwrap();
        outputter.record_mutation(2, &mutation(5, 0)).unwrap();

        let output = outputter.into_inner();
        let lines = body(&output);
        assert_eq!(lines.len(), 3);
        assert!(
            lines[1].is_empty(),
            "replicate change should insert a blank line"
        );
    }

    #[test]
    fn test_sequencing_outputter_does_not_delimit_within_a_replicate() {
        // The first replicate is 1, so recording it must not emit a leading blank line
        let mut outputter = SequencingOutputter::new(Vec::new(), &sim_cfg()).unwrap();

        outputter.record_mutation(1, &mutation(4, 0)).unwrap();

        let output = outputter.into_inner();
        assert_eq!(body(&output).len(), 1);
    }
}
