mod cli;
mod commands;
mod config;
mod error;

use clap::Parser;

fn main() -> std::process::ExitCode {
    let cli = cli::Cli::parse();

    match commands::run(cli) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            std::process::ExitCode::FAILURE
        }
    }
}
