use console::style;
use tinybridge_core::EnvironmentStatus;

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
}
