# loopgen — build spec

Build a production-quality standalone Rust CLI binary named **`loopgen`** in this
repository. It is an **agentic loop runner for Claude Code**: it turns a one-line
goal into an iterative loop that drives headless Claude Code (`claude -p`) until a
termination contract trips. This is the "all I do is write loops for Claude"
pattern, as a CLI.

This is a real tool that will ship. Idiomatic, production-quality Rust. No
placeholder TODOs, no stubs.

## Crate

- Rust 2021 edition, binary crate, package + binary name `loopgen`.
- Dependencies (keep minimal, current stable versions):
  - `clap` (derive feature) — argument parsing
  - `anyhow` — error handling
  - `serde` + `serde_json` — parse `claude --output-format json`
  - `regex` — extract the status line
- Must pass all three: `cargo build --release`, `cargo test`,
  `cargo clippy --all-targets -- -D warnings`.
- Include a `.gitignore` with `/target`.

### Cargo.toml metadata (for crates.io readiness)

```toml
[package]
name = "loopgen"
version = "0.1.0"
edition = "2021"
description = "Agentic loop runner for Claude Code — drives `claude -p` in an iterative loop with a LOOP_STATUS termination contract."
license = "MIT"
repository = "https://github.com/marcuspat/loopgen-rs"
keywords = ["claude", "cli", "agentic", "automation", "llm"]
categories = ["command-line-utilities", "development-tools"]
readme = "README.md"
```

A `LICENSE` file (MIT, in Marcus Patman's name) already exists on `main` — keep
it; do not overwrite. The README must carry a license badge and a "License: MIT"
section.

## CLI (clap derive)

| Arg | Type | Default | Meaning |
|---|---|---|---|
| `goal` | positional `String` | — (required) | The outcome to drive toward |
| `--max <N>` | `u32` | `8` | Hard iteration cap (safety rail) |
| `--verify <CMD>` | `Option<String>` | none | Shell command; `DONE` only accepted if it exits 0 |
| `--dod <TEXT>` | `Option<String>` | none | Explicit Definition of Done; else auto-derived |
| `--model <NAME>` | `Option<String>` | none | Forwarded to `claude -p --model` |
| `--dry-run` | flag | false | Render the harness, print it, exit 0 (no claude calls) |
| `--max-state-chars <N>` | `usize` | `4000` | Cap on running state carried between iterations |
| `--claude-bin <PATH>` | `String` | `"claude"` | Override the claude binary path |
| `-v, --verbose` | flag | false | Echo each invocation + raw status lines |

## Harness renderer

Implement `fn render_harness(cfg: &Config) -> String` that produces exactly the
following structure. Fill the slots; **omit the two `Run:` / `It MUST exit 0`
lines entirely when `--verify` is not provided.** Derive `slug` from the goal
(lowercase, non-alphanumerics → `-`, collapse, first ~6 words; fallback `loop`).
When `--dod` is absent, auto-derive: if `--verify` is set, "The stated goal is
achieved and the verify command (`<cmd>`) exits 0."; otherwise "The stated goal
is fully achieved, with concrete evidence cited for each success criterion."

```
# AGENTIC LOOP — {slug}

## Goal
{goal}

## Role
You are the loop CONTROLLER. Execute this as an ITERATIVE loop, not a single pass.

## Cycle (repeat each iteration)
1. PLAN   — state the smallest next increment toward the goal.
2. ACT    — do it. For non-trivial work spawn a worker; otherwise act directly.
3. VERIFY — check progress against the Definition of Done.
          Run: {verify}
          It MUST exit 0 before you may report DONE.
4. REPORT — emit exactly one line, this format:
          LOOP_STATUS: <DONE|CONTINUE|BLOCKED> | iter <n>/{max} | <one-line note>
5. CARRY  — update a running STATE summary: what is done, what remains, key decisions.

## Definition of Done
{dod}

## Termination
- Continue while status == CONTINUE and iter < {max}.
- Stop on DONE, BLOCKED, or iter == {max} (then report partial state — do not silently continue).

## Constraints (hard)
- Max iterations: {max}.
- Do NOT modify or delete files without explicit confirmation.
- Do NOT send any message (email/Slack/GitHub/etc.) without per-action confirmation.
- Infra changes (Terraform/K8s/CI): always dry-run / plan before apply.
- Never log or print secrets.
- If BLOCKED (needs a decision, missing credential, or ambiguity): stop and surface the specific question — do not guess.

## Begin
Start at iteration 1.
```

`--dry-run` prints this and exits 0.

## Loop engine

For `iter` in `1..=max`:

1. Compose the per-iteration prompt:
   `harness + "\n\n## Running state (prior iterations)\n" + state +
   "\n\n## This is iteration {iter} of {max}. Do ONE increment, then emit the LOOP_STATUS line."`
   (On iteration 1, state is "(none — first iteration).")
2. Invoke: `{claude_bin} -p <prompt> --output-format json [--model {model}]`.
   Capture stdout/stderr and exit status.
3. Parse stdout as JSON; extract the assistant result text. `claude -p
   --output-format json` returns an object with a `result` string — read that.
   If JSON parsing fails, treat the entire stdout as the result text (and warn in
   verbose mode).
4. Extract the status line with a case-insensitive regex matching
   `LOOP_STATUS:\s*(DONE|CONTINUE|BLOCKED)\s*\|\s*iter[^|]*\|\s*(.*)`. Use the
   **last** match if several appear. If no match: treat as `CONTINUE` and warn.
5. If status is `DONE` **and** `--verify` is set: run the verify command via
   `sh -c "<cmd>"`. If it exits non-zero, downgrade to `CONTINUE` with note
   `"verify failed (exit <code>)"`.
6. Print one concise line: `[iter n/max] <STATUS> — <note>`.
7. Append a trimmed summary of the result text to `state`, capped at
   `--max-state-chars` (keep the tail if over).
8. `break` on `DONE` or `BLOCKED`; otherwise continue.

After the loop, print a final summary line. **Exit codes:** `0` = DONE,
`2` = BLOCKED, `3` = reached `--max` without DONE, `1` = internal/setup error.

## Robustness

- `claude` binary not found → clear, actionable error to stderr, exit 1.
- Empty or non-JSON output → warn (verbose), treat result as raw text.
- Missing LOOP_STATUS line → warn, treat as CONTINUE (so a quiet model doesn't
  silently terminate the loop).
- Never print secrets or dump the full environment.

## Tests

- `render_harness`: verify slug derivation, that the verify lines are present
  with `--verify` and absent without, and that `--dod` overrides the auto value.
- status parser: DONE/CONTINUE/BLOCKED, case-insensitivity, last-match-wins,
  no-match → CONTINUE.
- exit-code mapping.

## Deliverables

- `Cargo.toml`, `src/` (suggested modules: `main.rs`, `cli.rs`, `harness.rs`,
  `engine.rs`, `status.rs`), tests, `.gitignore`.
- `README.md`: install (`cargo install --path .`), usage, a flags table, a
  `--dry-run` example, a `--verify` example
  (`loopgen "get tests green" --verify "cargo test"`), and a short "How the loop
  works" section explaining the LOOP_STATUS contract and the exit codes.
- Open a **pull request against `main`** with a clear description of what was built.

## Acceptance criteria

`cargo build --release`, `cargo test`, and
`cargo clippy --all-targets -- -D warnings` all succeed; `loopgen --help` shows
all flags; `loopgen "demo goal" --dry-run` prints the harness and exits 0.
