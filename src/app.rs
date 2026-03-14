use crate::cli::{Cli, Command, ConvertArgs, InputArgs};
use crate::error::TabstructError;
use crate::io;
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

/// schema コマンド: 入力取得・形式判定・パース・解析・表示。JSON/YAML 対応。CSV は未対応。
fn run_schema(args: InputArgs) -> Result<(), TabstructError> {
    let input = io::read_input(&args)?;
    let path_for_format = input.path.as_deref().map(Path::new);
    let format = io::detect_input_format(&args, path_for_format)?;
    let doc = parser::parse_document(format, &input.content)?;
    let report = schema::analyze(&doc)?;
    let text = crate::formatter::format_schema_report(&report);
    io::write_stdout(&text)?;
    Ok(())
}

/// convert コマンド: 入力取得・形式判定まで実施。パース・変換・出力は後続単位で実装。
fn run_convert(args: ConvertArgs) -> Result<(), TabstructError> {
    let input = io::read_input(&args.input)?;
    let path_for_format = input.path.as_deref().map(Path::new);
    let _in_format = io::detect_input_format(&args.input, path_for_format)?;
    let _out_type = args.require_output_type()?;
    // パース・converter・write_output は後続単位で実装
    todo!("convert: parse, convert, and write output");
}

