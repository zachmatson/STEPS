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
