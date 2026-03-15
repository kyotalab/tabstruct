use crate::cli::{Cli, Command, ConvertArgs, InputArgs};
use crate::converter;
use crate::error::TabstructError;
use crate::io;
use crate::model::{Document, InputFormat};
use crate::parser;
use crate::schema;
use clap::Parser;
use std::path::Path;

pub fn run() -> Result<(), TabstructError> {
    let cli = Cli::parse();

    match cli.command {
        Command::Schema(args) => run_schema(args),
        Command::Convert(args) => run_convert(args),
    }
}

/// 形式と内容から Document を取得する。
/// CSV の場合は parser::csv + converter::csv_to_model で unflatten まで行う。
fn get_document(format: InputFormat, content: &str) -> Result<Document, TabstructError> {
    match format {
        InputFormat::Csv => {
            let raw = parser::csv::parse_csv(content)?;
            let typed = parser::csv::raw_to_typed(raw)?;
            converter::csv_to_model::typed_table_to_document(typed)
        }
        InputFormat::Json | InputFormat::Yaml => parser::parse_document(format, content),
    }
}

/// schema コマンド: 入力取得・形式判定・パース・解析・表示。CSV/JSON/YAML 対応。
fn run_schema(args: InputArgs) -> Result<(), TabstructError> {
    let input = io::read_input(&args)?;
    let path_for_format = input.path.as_deref().map(Path::new);
    let format = io::detect_input_format(&args, path_for_format)?;
    let doc = get_document(format, &input.content)?;
    let report = schema::analyze(&doc)?;
    let text = crate::formatter::format_schema_report(&report);
    io::write_stdout(&text)?;
    Ok(())
}

/// convert コマンド: 入力取得・形式判定・パース・変換・出力。
fn run_convert(args: ConvertArgs) -> Result<(), TabstructError> {
    let input = io::read_input(&args.input)?;
    let path_for_format = input.path.as_deref().map(Path::new);
    let in_format = io::detect_input_format(&args.input, path_for_format)?;
    let out_type = args.require_output_type()?;
    let doc = get_document(in_format, &input.content)?;
    let rendered = converter::convert(&doc, out_type)?;
    io::write_output(args.output.as_deref(), &rendered)?;
    Ok(())
}

