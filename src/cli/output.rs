//! Output formatting for analysis results.

use crate::config::OutputFormat;
use crate::core::{AnalysisResult, Confidence, DeadSymbol, Result, SymbolKind};
use colored::Colorize;
use std::collections::HashMap;
use tabled::{
    settings::{object::Rows, Alignment, Modify, Style},
    Table, Tabled,
};

/// Writer for analysis output in various formats.
pub struct OutputWriter {
    format: OutputFormat,
    verbose: bool,
}

impl OutputWriter {
    pub fn new(format: OutputFormat, verbose: bool) -> Self {
        Self { format, verbose }
    }

    /// Write analysis results to stdout.
    pub fn write_result(
        &mut self,
        result: &AnalysisResult,
        dead_symbols: &[&DeadSymbol],
        show_chains: bool,
    ) -> Result<()> {
        match self.format {
            OutputFormat::Table => self.write_table(dead_symbols, show_chains),
            OutputFormat::Json => self.write_json(result, dead_symbols),
            OutputFormat::Compact => self.write_compact(dead_symbols),
        }
    }

    fn write_table(&self, dead_symbols: &[&DeadSymbol], show_chains: bool) -> Result<()> {
        if dead_symbols.is_empty() {
            println!("{}", "No dead code found!".green().bold());
            return Ok(());
        }

        // Group by file
        let mut by_file: HashMap<String, Vec<&DeadSymbol>> = HashMap::new();
        for dead in dead_symbols {
            let file = dead.symbol.location.file_path.display().to_string();
            by_file.entry(file).or_default().push(dead);
        }

        // Sort files
        let mut files: Vec<_> = by_file.keys().cloned().collect();
        files.sort();

        for file in files {
            println!("\n{}", file.cyan().bold());

            let symbols = by_file.get(&file).unwrap();
            let mut rows: Vec<TableRow> = symbols
                .iter()
                .map(|d| TableRow {
                    line: d.symbol.location.line.to_string(),
                    name: d.symbol.name.clone(),
                    kind: format_kind(d.symbol.kind),
                    confidence: format_confidence(d.confidence, d.confidence_score),
                    reason: d.reason.description(),
                })
                .collect();

            rows.sort_by_key(|r| r.line.parse::<u32>().unwrap_or(0));

            let table = Table::new(&rows)
                .with(Style::rounded())
                .with(Modify::new(Rows::first()).with(Alignment::center()))
                .to_string();

            println!("{}", table);

            // Show chains if requested
            if show_chains && self.verbose {
                for dead in symbols {
                    if let crate::core::DeadnessReason::Transitive { chain } = &dead.reason {
                        if !chain.is_empty() {
                            println!(
                                "  {} Chain: {} symbols",
                                "└─".dimmed(),
                                chain.len()
                            );
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn write_json(&self, result: &AnalysisResult, dead_symbols: &[&DeadSymbol]) -> Result<()> {
        let output = JsonOutput {
            total_symbols: result.total_symbols,
            total_files: result.total_files,
            dead_count: dead_symbols.len(),
            duration_ms: result.duration_ms,
            dead_symbols: dead_symbols
                .iter()
                .map(|d| JsonDeadSymbol {
                    name: d.symbol.name.clone(),
                    kind: format!("{:?}", d.symbol.kind),
                    file: d.symbol.location.file_path.display().to_string(),
                    line: d.symbol.location.line,
                    column: d.symbol.location.column,
                    confidence: d.confidence.label().to_string(),
                    confidence_score: d.confidence_score,
                    reason: d.reason.description(),
                    exported: d.symbol.exported,
                })
                .collect(),
            warnings: result
                .warnings
                .iter()
                .map(|w| w.message.clone())
                .collect(),
        };

        let json = serde_json::to_string_pretty(&output)
            .map_err(|e| crate::core::DddError::analysis_error(format!("JSON serialization failed: {}", e)))?;
        println!("{}", json);

        Ok(())
    }

    fn write_compact(&self, dead_symbols: &[&DeadSymbol]) -> Result<()> {
        for dead in dead_symbols {
            println!(
                "{}:{}:{}: {} ({}) - {}",
                dead.symbol.location.file_path.display(),
                dead.symbol.location.line,
                dead.symbol.location.column,
                dead.symbol.name,
                format_kind(dead.symbol.kind),
                dead.confidence.label()
            );
        }
        Ok(())
    }
}

#[derive(Tabled)]
struct TableRow {
    #[tabled(rename = "Line")]
    line: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Kind")]
    kind: String,
    #[tabled(rename = "Confidence")]
    confidence: String,
    #[tabled(rename = "Reason")]
    reason: String,
}

#[derive(serde::Serialize)]
struct JsonOutput {
    total_symbols: usize,
    total_files: usize,
    dead_count: usize,
    duration_ms: u64,
    dead_symbols: Vec<JsonDeadSymbol>,
    warnings: Vec<String>,
}

#[derive(serde::Serialize)]
struct JsonDeadSymbol {
    name: String,
    kind: String,
    file: String,
    line: u32,
    column: u32,
    confidence: String,
    confidence_score: u8,
    reason: String,
    exported: bool,
}

fn format_kind(kind: SymbolKind) -> String {
    match kind {
        SymbolKind::Function => "fn",
        SymbolKind::ArrowFunction => "=>",
        SymbolKind::Class => "class",
        SymbolKind::Method => "method",
        SymbolKind::Variable => "var",
        SymbolKind::Constant => "const",
        SymbolKind::Type => "type",
        SymbolKind::Interface => "interface",
        SymbolKind::Enum => "enum",
        SymbolKind::EnumMember => "member",
        SymbolKind::Namespace => "namespace",
        SymbolKind::Module => "module",
    }
    .to_string()
}

fn format_confidence(confidence: Confidence, score: u8) -> String {
    let label = format!("{} ({})", confidence.label(), score);
    match confidence {
        Confidence::High => label.green().to_string(),
        Confidence::Medium => label.yellow().to_string(),
        Confidence::Low => label.red().to_string(),
    }
}
