use console::style;
use tinybridge_core::EnvironmentStatus;
use tinybridge_error::BridgeError;

#[allow(dead_code)]
pub fn status_badge(status: &str) -> String {
    match status.to_lowercase().as_str() {
        "running" => format!("{} {}", style("✓").green(), style("Running").green()),
        "stopped" => format!("{} {}", style("○").dim(), style("Stopped").dim()),
        "starting" => format!("{} {}", style("⟳").yellow(), style("Starting").yellow()),
        "stopping" => format!("{} {}", style("⟳").yellow(), style("Stopping").yellow()),
        "error" => format!("{} {}", style("✗").red(), style("Error").red()),
        _ => format!("{} {}", style("?").dim(), style(status).dim()),
    }
}

#[allow(dead_code)]
pub fn format_status_enum(status: &EnvironmentStatus) -> String {
    match status {
        EnvironmentStatus::Running { uptime_secs } => {
            format!("{} ({}s uptime)", status_badge("running"), uptime_secs)
        }
        EnvironmentStatus::Starting { progress_pct } => {
            format!("{} {}%", status_badge("starting"), progress_pct)
        }
        EnvironmentStatus::Stopped => status_badge("stopped"),
        EnvironmentStatus::Stopping => status_badge("stopping"),
        EnvironmentStatus::Error { message } => {
            format!("{} {}", status_badge("error"), style(message).red())
        }
    }
}

pub fn print_header(text: &str) {
    println!("{}", style(text).bold().cyan());
}

pub fn print_success(text: &str) {
    println!("{} {}", style("✓").green(), text);
}

#[allow(dead_code)]
pub fn print_error(text: &str) {
    eprintln!("{} {}", style("✗").red(), text);
}

pub fn print_info(text: &str) {
    println!("{} {}", style("→").blue(), text);
}

pub fn print_warning(text: &str) {
    println!("{} {}", style("⚠").yellow(), text);
}

pub fn print_config(key: &str, value: &str) {
    println!("  {} {}: {}", style("•").yellow(), key, style(value).cyan());
}

pub fn print_template(name: &str, description: &str) {
    println!(
        "  {} {} - {}",
        style("▪").cyan(),
        style(name).bold(),
        description
    );
}

pub fn print_image(name: &str, version: &str, size_mb: u32, arch: &str) {
    println!(
        "  {} {} {} ({} MB, {})",
        style("▪").cyan(),
        style(name).bold(),
        style(version).dim(),
        size_mb,
        arch
    );
}

pub fn print_check_pass(text: &str) {
    println!("{} {}", style("✓").green(), text);
}

pub fn print_check_warning(text: &str) {
    println!("{} {}", style("⚠").yellow(), text);
}

pub fn print_check_fail(text: &str) {
    println!("{} {}", style("✗").red(), text);
}

pub fn print_summary(total: usize, warnings: usize, failures: usize) {
    if failures > 0 {
        println!(
            "{} Diagnostics: {}/{} passed, {} warning(s), {} failure(s)",
            style("Summary").bold(),
            total - warnings - failures,
            total,
            warnings,
            failures
        );
    } else if warnings > 0 {
        println!(
            "{} Diagnostics: {}/{} passed, {} warning(s)",
            style("Summary").bold(),
            total - warnings,
            total,
            warnings
        );
    } else {
        println!(
            "{} All {} check(s) passed ✓",
            style("Summary").bold(),
            total
        );
    }
}

#[allow(dead_code)]
pub fn print_bridge_error(error: &BridgeError) {
    let symbol = error.severity.symbol();
    let severity = style(error.severity.to_string()).red();
    eprintln!("{} {}: {}", symbol, severity, style(&error.message).red());

    if let Some(ctx) = &error.context {
        eprintln!();
        eprintln!("{}", style("Context:").dim().italic());
        for (key, value) in &ctx.details {
            eprintln!("  {} {} {}", style("•").dim(), style(key).dim(), value);
        }
    }

    if let Some(sugg) = &error.suggestion {
        eprintln!();
        eprintln!("{}", style("Recovery Steps:").yellow().bold());
        for line in sugg.to_string().lines() {
            eprintln!("  {}", style(line).yellow());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_badge_running() {
        let badge = status_badge("running");
        assert!(badge.contains("Running"));
    }

    #[test]
    fn test_status_badge_stopped() {
        let badge = status_badge("stopped");
        assert!(badge.contains("Stopped"));
    }

    #[test]
    fn test_bridge_error_display() {
        let err = BridgeError::vm("Test error".to_string());
        let msg = err.user_message();
        assert!(msg.contains("Test error"));
    }
}
