//! Parsing of the `LOOP_STATUS` contract line emitted by the model.

use std::fmt;
use std::sync::OnceLock;

use regex::Regex;

/// The three terminal/continuation states of the loop contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Done,
    Continue,
    Blocked,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Status::Done => "DONE",
            Status::Continue => "CONTINUE",
            Status::Blocked => "BLOCKED",
        };
        f.write_str(s)
    }
}

/// A parsed `LOOP_STATUS` line: the status plus its trailing one-line note.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoopStatus {
    pub status: Status,
    pub note: String,
}

fn status_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)LOOP_STATUS:\s*(DONE|CONTINUE|BLOCKED)\s*\|\s*iter[^|]*\|\s*(.*)")
            .expect("status regex is valid")
    })
}

/// Extract the `LOOP_STATUS` line from arbitrary result text.
///
/// Case-insensitive; when several lines match, the **last** one wins. Returns
/// `None` when no line matches (the caller treats that as `CONTINUE`).
pub fn parse_status(text: &str) -> Option<LoopStatus> {
    let re = status_regex();
    let caps = re.captures_iter(text).last()?;

    let status = match caps[1].to_ascii_uppercase().as_str() {
        "DONE" => Status::Done,
        "BLOCKED" => Status::Blocked,
        _ => Status::Continue,
    };
    let note = caps[2].trim().to_string();

    Some(LoopStatus { status, note })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_done() {
        let s = parse_status("LOOP_STATUS: DONE | iter 3/8 | all green").unwrap();
        assert_eq!(s.status, Status::Done);
        assert_eq!(s.note, "all green");
    }

    #[test]
    fn parses_continue() {
        let s = parse_status("LOOP_STATUS: CONTINUE | iter 1/8 | made progress").unwrap();
        assert_eq!(s.status, Status::Continue);
        assert_eq!(s.note, "made progress");
    }

    #[test]
    fn parses_blocked() {
        let s = parse_status("LOOP_STATUS: BLOCKED | iter 2/8 | need a credential").unwrap();
        assert_eq!(s.status, Status::Blocked);
        assert_eq!(s.note, "need a credential");
    }

    #[test]
    fn case_insensitive() {
        let s = parse_status("loop_status: done | ITER 4/8 | finished").unwrap();
        assert_eq!(s.status, Status::Done);
        assert_eq!(s.note, "finished");
    }

    #[test]
    fn last_match_wins() {
        let text = "LOOP_STATUS: CONTINUE | iter 1/8 | early\n\
                    some chatter\n\
                    LOOP_STATUS: DONE | iter 2/8 | finally";
        let s = parse_status(text).unwrap();
        assert_eq!(s.status, Status::Done);
        assert_eq!(s.note, "finally");
    }

    #[test]
    fn no_match_returns_none() {
        assert!(parse_status("the model said nothing useful").is_none());
        assert!(parse_status("LOOP_STATUS without the right shape").is_none());
    }

    #[test]
    fn tolerates_extra_iter_text() {
        let s = parse_status("LOOP_STATUS: CONTINUE | iter 5 of 8 | note here").unwrap();
        assert_eq!(s.status, Status::Continue);
        assert_eq!(s.note, "note here");
    }

    #[test]
    fn empty_note_ok() {
        let s = parse_status("LOOP_STATUS: DONE | iter 1/1 |").unwrap();
        assert_eq!(s.status, Status::Done);
        assert_eq!(s.note, "");
    }
}
