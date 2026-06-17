//! `loopgen` — an agentic loop runner for Claude Code.
//!
//! Turns a one-line goal into an iterative loop that drives headless Claude
//! Code (`claude -p`) until the `LOOP_STATUS` termination contract trips.

mod cli;
mod engine;
mod harness;
mod status;

use std::process::ExitCode;

use clap::Parser;

use cli::Config;

fn main() -> ExitCode {
    let cfg = Config::parse();

    if cfg.dry_run {
        println!("{}", harness::render_harness(&cfg));
        return ExitCode::SUCCESS;
    }

    match engine::run(&cfg) {
        Ok(outcome) => ExitCode::from(outcome.exit_code() as u8),
        Err(err) => {
            eprintln!("error: {err:#}");
            ExitCode::from(1)
        }
    }
}
