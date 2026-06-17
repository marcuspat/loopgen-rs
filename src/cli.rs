//! Command-line argument definitions for `loopgen`.

use clap::Parser;

/// Agentic loop runner for Claude Code.
///
/// Turns a one-line goal into an iterative loop that drives headless Claude
/// Code (`claude -p`) until a termination contract (`LOOP_STATUS`) trips.
#[derive(Parser, Debug, Clone)]
#[command(name = "loopgen", version, about, long_about = None)]
pub struct Config {
    /// The outcome to drive toward.
    pub goal: String,

    /// Hard iteration cap (safety rail).
    #[arg(long, default_value_t = 8)]
    pub max: u32,

    /// Shell command; `DONE` is only accepted if it exits 0.
    #[arg(long)]
    pub verify: Option<String>,

    /// Explicit Definition of Done; otherwise auto-derived.
    #[arg(long)]
    pub dod: Option<String>,

    /// Model name forwarded to `claude -p --model`.
    #[arg(long)]
    pub model: Option<String>,

    /// Render the harness, print it, and exit 0 (no claude calls).
    #[arg(long, default_value_t = false)]
    pub dry_run: bool,

    /// Cap on running state carried between iterations.
    #[arg(long, default_value_t = 4000)]
    pub max_state_chars: usize,

    /// Override the claude binary path.
    #[arg(long, default_value = "claude")]
    pub claude_bin: String,

    /// Echo each invocation and raw status lines.
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,
}
