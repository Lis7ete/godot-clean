use crate::analyzer::AnalysisResult;
use colored::Colorize;

pub fn print_banner() {
    println!(
        "{}",
        r#"
  __ _  ___   __| | ___ | |_         ___ | | ___   __ _  _ __  
 / _` |/ _ \ / _` |/ _ \| __| _____ / __|| |/ _ \ / _` || '_ \ 
| (_| | (_) | (_| | (_) | |_ |_____| (__ | |  __/| (_| || | | |
 \__, |\___/ \__,_|\___/ \__|       \___||_|\___| \__,_||_| |_|
 |___/                                                         
"#
        .bright_cyan()
        .bold()
    );
    println!(
        "{} v{} — {}",
        "godot-clean".bright_magenta().bold(),
        env!("CARGO_PKG_VERSION"),
        "Fast parallel linter & cleanup tool for Godot projects"
            .white()
            .dimmed()
    );
    println!("{}", "—".repeat(65).dimmed());
}

pub fn print_json(result: &AnalysisResult) {
    if let Ok(json) = serde_json::to_string_pretty(result) {
        println!("{}", json);
    }
}

pub fn print_report(result: &AnalysisResult) {
    // 1. Scan / Read Errors
    let total_io_errors = result.scan_errors.len() + result.read_errors.len();
    if total_io_errors > 0 {
        println!(
            "\n{} {}",
            "❗ I/O & SCAN ERRORS".bright_red().bold(),
            format!("({})", total_io_errors).red()
        );
        for err in &result.scan_errors {
            println!("  {} [Scan] {}", "•".red(), err.dimmed());
        }
        for err in &result.read_errors {
            println!("  {} [Read] {}", "•".red(), err.dimmed());
        }
    }

    // 2. Broken References
    if !result.broken_references.is_empty() {
        println!(
            "\n{} {}",
            "❌ BROKEN REFERENCES".bright_red().bold(),
            format!("({})", result.broken_references.len()).red()
        );
        for item in &result.broken_references {
            println!(
                "  {} {}:{} -> {} ({})",
                "•".bright_red(),
                item.source_file.bright_yellow(),
                item.line_number.to_string().cyan(),
                item.broken_path.bright_red().underline(),
                item.reason.dimmed()
            );
        }
    } else {
        println!(
            "\n{} {}",
            "✔".bright_green().bold(),
            "No broken references found!".green()
        );
    }

    // 3. Orphaned Imports
    if !result.orphaned_imports.is_empty() {
        println!(
            "\n{} {}",
            "⚠️  ORPHANED .IMPORT FILES".bright_yellow().bold(),
            format!("({})", result.orphaned_imports.len()).yellow()
        );
        for import in &result.orphaned_imports {
            println!("  {} {}", "•".yellow(), import.dimmed());
        }
    }

    // 4. Unused Assets (Candidates)
    if !result.unused_assets.is_empty() {
        println!(
            "\n{} {}",
            "📦 UNUSED ASSETS (CANDIDATES)".bright_blue().bold(),
            format!(
                "({} files, ~{})",
                result.unused_assets.len(),
                format_bytes(result.unused_bytes)
            )
            .bright_cyan()
        );
        println!(
            "  {}",
            "Note: Verify before deleting; some assets may be loaded dynamically in code."
                .italic()
                .dimmed()
        );
        for asset in result.unused_assets.iter().take(15) {
            println!("  {} {}", "•".bright_blue(), asset);
        }
        if result.unused_assets.len() > 15 {
            println!(
                "  {}",
                format!("... and {} more assets", result.unused_assets.len() - 15).dimmed()
            );
        }
    }

    // 5. Summary Table
    println!("\n{}", "📊 PROJECT OVERVIEW".bright_white().bold());
    println!("{}", "—".repeat(65).dimmed());
    println!(
        "  {:<25} {:>10}",
        "Scenes (.tscn):",
        result.total_scenes.to_string().bright_cyan()
    );
    println!(
        "  {:<25} {:>10}",
        "Scripts (.gd, .cs):",
        result.total_scripts.to_string().bright_cyan()
    );
    println!(
        "  {:<25} {:>10}",
        "Resources (.tres):",
        result.total_resources.to_string().bright_cyan()
    );
    println!(
        "  {:<25} {:>10}",
        "Assets scanned:",
        result.total_assets.to_string().bright_cyan()
    );
    println!("{}", "—".repeat(65).dimmed());

    let total_issues = result.broken_references.len() + result.orphaned_imports.len();
    if total_issues == 0 && total_io_errors == 0 {
        println!(
            "{}",
            "✨ Project is in pristine condition! All links are valid."
                .bright_green()
                .bold()
        );
    } else {
        println!(
            "{} Found {} issue(s) that need attention.",
            "⚠️".yellow(),
            (total_issues + total_io_errors).to_string().bright_yellow().bold()
        );
    }
}

pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
