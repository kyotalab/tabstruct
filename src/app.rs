use crate::cli::{Cli, Command};
use crate::error::TabstructError;
use clap::Parser;

pub fn run() -> Result<(), TabstructError> {
    let cli = Cli::parse();

    match cli.command {
        Command::Schema(_args) => {
            // schema コマンド本体は後続単位で実装
            todo!("schema command is not implemented yet");
        }
        Command::Convert(_args) => {
            // convert コマンド本体は後続単位で実装
            todo!("convert command is not implemented yet");
        }
    }
}

