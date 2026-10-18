use anyhow::{Context, Result};
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::output;
use clap::Parser;

#[derive(Parser)]
pub struct LogsArgs {
    /// Environment name (all if not specified)
    pub name: Option<String>,

    /// Show last N lines
    #[arg(long, short, default_value = "50")]
    pub tail: usize,

    /// Follow log output (like `tail -f`)
    #[arg(long, short)]
    pub follow: bool,

    /// Show only errors
    #[arg(long)]
    pub errors: bool,

    /// Show only warnings
    #[arg(long)]
    pub warnings: bool,

    /// Path to the daemon log file
    #[arg(long, env = "TINYBRIDGE_LOG_FILE")]
    pub file: Option<PathBuf>,
}

/// Default location of the daemon log written by `tinybridged`.
pub fn default_log_path() -> PathBuf {
    tinybridge_core::TinyBridgeConfig::logs_dir().join("tinybridged.log")
}

fn line_matches(line: &str, name: Option<&str>, errors: bool, warnings: bool) -> bool {
    if let Some(name) = name {
        if !line.contains(name) {
            return false;
        }
    }
    if errors {
        return line.contains("ERROR");
    }
    if warnings {
        return line.contains("WARN");
    }
    true
}

pub async fn execute(args: LogsArgs, _socket: Option<PathBuf>) -> Result<()> {
    let env_name = args.name.as_deref().unwrap_or("all");

    output::print_header(&format!("Logs for {}", env_name));

    if args.errors {
        output::print_info("Showing errors only");
    } else if args.warnings {
        output::print_info("Showing warnings only");
    }

    let path = args.file.clone().unwrap_or_else(default_log_path);

    if !path.exists() {
        output::print_warning(&format!("No daemon log file at {}", path.display()));
        output::print_info(
            "The `tinybridged` daemon writes to this file on start. Start it, or point \
             --file / TINYBRIDGE_LOG_FILE at an existing log.",
        );
        return Ok(());
    }

    let name_filter = args.name.as_deref();

    if args.follow {
        output::print_info("Following log output (Ctrl+C to stop)...");
        follow(&path, name_filter, args.errors, args.warnings).await?;
    } else {
        output::print_info(&format!("Last {} lines:", args.tail));
        for line in read_tail(&path, args.tail)? {
            if line_matches(&line, name_filter, args.errors, args.warnings) {
                println!("{}", line);
            }
        }
    }

    Ok(())
}

/// Read up to the last `tail` lines of the file at `path`.
fn read_tail(path: &Path, tail: usize) -> Result<Vec<String>> {
    let file = std::fs::File::open(path).with_context(|| format!("opening {}", path.display()))?;
    let mut lines: Vec<String> = BufReader::new(file)
        .lines()
        .collect::<std::io::Result<_>>()?;
    if lines.len() > tail {
        lines.drain(0..lines.len() - tail);
    }
    Ok(lines)
}

/// Tail the file, printing new matching lines as they are appended.
async fn follow(path: &Path, name: Option<&str>, errors: bool, warnings: bool) -> Result<()> {
    let mut file =
        std::fs::File::open(path).with_context(|| format!("opening {}", path.display()))?;
    file.seek(SeekFrom::End(0))?;
    let mut reader = BufReader::new(file);

    loop {
        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 {
            tokio::time::sleep(Duration::from_millis(250)).await;
            continue;
        }
        if line_matches(line.trim_end(), name, errors, warnings) {
            print!("{}", line);
            use std::io::Write as _;
            std::io::stdout().flush().ok();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn temp_file(contents: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        let unique = format!(
            "tinybridge-logs-test-{}-{:?}.log",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        path.push(unique);
        let mut file = std::fs::File::create(&path).unwrap();
        file.write_all(contents.as_bytes()).unwrap();
        path
    }

    #[test]
    fn read_tail_returns_last_n_lines() {
        let path = temp_file("one\ntwo\nthree\n");
        let lines = read_tail(&path, 2).unwrap();
        assert_eq!(lines, vec!["two", "three"]);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn level_filter_matches_errors_and_warnings() {
        let err = "2026-10-18T00:00:00Z ERROR tinybridged: boom";
        let warn = "2026-10-18T00:00:00Z  WARN tinybridged: careful";
        let info = "2026-10-18T00:00:00Z  INFO tinybridged: ok";

        assert!(line_matches(err, None, true, false));
        assert!(!line_matches(info, None, true, false));
        assert!(line_matches(warn, None, false, true));
        assert!(!line_matches(err, None, false, true));
        assert!(line_matches(info, None, false, false));
    }

    #[test]
    fn name_filter_restricts_lines() {
        let line = "2026-10-18T00:00:00Z  INFO env dev-box booted";
        assert!(line_matches(line, Some("dev-box"), false, false));
        assert!(!line_matches(line, Some("other"), false, false));
    }
}
