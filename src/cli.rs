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
    Schema(InputArgs),
    Convert(ConvertArgs),
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum InputType {
    Csv,
    Json,
    Yaml,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OutputType {
    Csv,
    Json,
    Yaml,
}

#[derive(Debug, Args)]
pub struct InputArgs {
    #[arg(long, conflicts_with = "stdin")]
    pub file: Option<PathBuf>,

    #[arg(long, conflicts_with = "file")]
    pub stdin: bool,

    #[arg(long, value_enum, required_if_eq("stdin", "true"))]
    pub r#type: Option<InputType>,
}

#[derive(Debug, Args)]
pub struct ConvertArgs {
    #[command(flatten)]
    pub input: InputArgs,

    #[arg(long, conflicts_with_all = ["yaml", "csv"])]
    pub json: bool,

    #[arg(long, conflicts_with_all = ["json", "csv"])]
    pub yaml: bool,

    #[arg(long, conflicts_with_all = ["json", "yaml"])]
    pub csv: bool,

    #[arg(long)]
    pub output: Option<PathBuf>,
}

impl ConvertArgs {
    pub fn output_type(&self) -> OutputType {
        match (self.json, self.yaml, self.csv) {
            (true, false, false) => OutputType::Json,
            (false, true, false) => OutputType::Yaml,
            (false, false, true) => OutputType::Csv,
            _ => unreachable!("clap guarantees exactly one output flag"),
        }
    }
}

