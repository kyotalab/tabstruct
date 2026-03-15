use crate::error::TabstructError;
use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "tabstruct")]
#[command(version)]
#[command(about = "Convert CSV, JSON, and YAML while inspecting structure")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Show structure (format, root type, record count, fields and types) of CSV/JSON/YAML input
    Schema(InputArgs),
    /// Convert input to JSON, YAML, or CSV
    Convert(ConvertArgs),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum InputType {
    Csv,
    Json,
    Yaml,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputType {
    Csv,
    Json,
    Yaml,
}

#[derive(Debug, Args)]
#[group(required = true, id = "input_source")]
pub struct InputArgs {
    /// Input file path (format inferred from extension: .csv, .json, .yaml, .yml)
    #[arg(long, conflicts_with = "stdin", group = "input_source")]
    pub file: Option<PathBuf>,

    /// Read from stdin (requires --type to specify format)
    #[arg(long, conflicts_with = "file", group = "input_source")]
    pub stdin: bool,

    /// Input format when using --stdin (required); ignored when using --file
    #[arg(long, value_enum, required_if_eq("stdin", "true"))]
    pub r#type: Option<InputType>,
}

#[derive(Debug, Args)]
pub struct ConvertArgs {
    #[command(flatten)]
    pub input: InputArgs,

    /// Output as JSON
    #[arg(long, conflicts_with_all = ["yaml", "csv"])]
    pub json: bool,

    /// Output as YAML
    #[arg(long, conflicts_with_all = ["json", "csv"])]
    pub yaml: bool,

    /// Output as CSV (nested fields use dot notation, e.g. settings.interval)
    #[arg(long, conflicts_with_all = ["json", "yaml"])]
    pub csv: bool,

    /// Write result to file (default: stdout)
    #[arg(long)]
    pub output: Option<PathBuf>,
}

impl ConvertArgs {
    /// 出力形式を返す。いずれか1つのみ指定されている場合に Some を返す。
    pub fn output_type(&self) -> Option<OutputType> {
        match (self.json, self.yaml, self.csv) {
            (true, false, false) => Some(OutputType::Json),
            (false, true, false) => Some(OutputType::Yaml),
            (false, false, true) => Some(OutputType::Csv),
            _ => None,
        }
    }

    /// 出力形式を取得する。未指定の場合は MissingOutputFormat を返す。
    pub fn require_output_type(&self) -> Result<OutputType, TabstructError> {
        self.output_type()
            .ok_or(TabstructError::MissingOutputFormat)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_parse_schema_with_file() {
        let cli = Cli::parse_from(["tabstruct", "schema", "--file", "sample.csv"]);
        match &cli.command {
            Command::Schema(args) => {
                assert!(args.file.as_ref().unwrap().to_str().unwrap() == "sample.csv");
                assert!(!args.stdin);
            }
            _ => panic!("expected Schema"),
        }
    }

    #[test]
    fn cli_parse_convert_with_output_format() {
        let cli = Cli::parse_from(["tabstruct", "convert", "--file", "x.csv", "--json"]);
        match &cli.command {
            Command::Convert(args) => {
                assert!(args.output_type() == Some(OutputType::Json));
                assert!(args.require_output_type().is_ok());
            }
            _ => panic!("expected Convert"),
        }
    }

    #[test]
    fn cli_convert_requires_one_output_format() {
        let cli = Cli::parse_from(["tabstruct", "convert", "--file", "x.csv"]);
        match &cli.command {
            Command::Convert(args) => {
                assert!(args.output_type().is_none());
                assert!(args.require_output_type().is_err());
            }
            _ => panic!("expected Convert"),
        }
    }

    #[test]
    fn cli_schema_requires_file_or_stdin() {
        let result = Cli::try_parse_from(["tabstruct", "schema"]);
        assert!(
            result.is_err(),
            "schema without --file or --stdin must fail"
        );
    }
}
