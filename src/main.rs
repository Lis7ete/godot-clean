mod analyzer;
mod reporter;
mod scanner;

use clap::{Parser, Subcommand, ValueEnum};
use colored::Colorize;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
enum OutputFormat {
    Text,
    Json,
}

#[derive(Parser, Debug)]
#[command(
    name = "godot-clean",
    author = "Lis7ete",
    version = env!("CARGO_PKG_VERSION"),
    about = "Blazing-fast parallel CLI linter and cleanup tool for Godot projects",
    long_about = "Scan, lint, and clean Godot 3/4 projects for broken scene references, unresolved UIDs, orphaned .import files, and unused assets."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Path to Godot project (defaults to current directory)
    #[arg(global = true, default_value = ".")]
    path: PathBuf,

    /// Output format
    #[arg(short, long, global = true, value_enum, default_value_t = OutputFormat::Text)]
    format: OutputFormat,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Check project for broken references and orphaned files (default)
    Check {
        /// Treat all warnings and I/O errors as failures (exit code 1)
        #[arg(long)]
        strict: bool,
    },
    /// Fix and remove orphaned .import files
    Fix {
        /// Preview files to be deleted without actually deleting
        #[arg(long)]
        dry_run: bool,
    },
    /// Display project statistics and resource counts
    Stats,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let target_path = if cli.path == Path::new(".") {
        current_dir
    } else {
        cli.path
    };

    let project_root = match scanner::find_godot_project(&target_path) {
        Some(root) => root,
        None => {
            if cli.format == OutputFormat::Text {
                reporter::print_banner();
                eprintln!(
                    "{} Could not find '{}' in '{}' or any parent directory.",
                    "Error:".bright_red().bold(),
                    "project.godot".yellow(),
                    target_path.display()
                );
            } else {
                eprintln!(r#"{{"error": "project.godot not found"}}"#);
            }
            return ExitCode::FAILURE;
        }
    };

    if cli.format == OutputFormat::Text {
        reporter::print_banner();
        println!(
            "{} {}\n",
            "📂 Found Godot project at:".bright_white(),
            project_root.display().to_string().bright_cyan().bold()
        );
    }

    let files = scanner::scan_project(&project_root);
    let analysis = analyzer::analyze_project(&files);

    match cli.command.unwrap_or(Commands::Check { strict: false }) {
        Commands::Check { strict } => {
            if cli.format == OutputFormat::Json {
                reporter::print_json(&analysis);
            } else {
                reporter::print_report(&analysis);
            }

            let has_errors = !analysis.broken_references.is_empty();
            let has_warnings = !analysis.orphaned_imports.is_empty()
                || !analysis.scan_errors.is_empty()
                || !analysis.read_errors.is_empty();

            if has_errors || (strict && has_warnings) {
                return ExitCode::FAILURE;
            }
        }
        Commands::Fix { dry_run } => {
            if cli.format == OutputFormat::Json {
                reporter::print_json(&analysis);
            }

            if analysis.orphaned_imports.is_empty() {
                if cli.format == OutputFormat::Text {
                    println!(
                        "{}",
                        "✔ No orphaned .import files found. Nothing to clean!".bright_green()
                    );
                }
            } else {
                if cli.format == OutputFormat::Text {
                    println!(
                        "{} Found {} orphaned .import file(s):",
                        if dry_run { "🔍 [Dry Run]" } else { "🧹 Cleaning" }.bright_yellow().bold(),
                        analysis.orphaned_imports.len()
                    );
                }

                let mut had_deletion_error = false;
                for rel_import in &analysis.orphaned_imports {
                    let full_path = project_root.join(rel_import);
                    if dry_run {
                        if cli.format == OutputFormat::Text {
                            println!("  [Would delete] {}", rel_import.dimmed());
                        }
                    } else {
                        match fs::remove_file(&full_path) {
                            Ok(_) => {
                                if cli.format == OutputFormat::Text {
                                    println!("  {} Deleted {}", "✔".bright_green(), rel_import);
                                }
                            }
                            Err(e) => {
                                had_deletion_error = true;
                                eprintln!(
                                    "  {} Failed to delete {}: {}",
                                    "✖".bright_red(),
                                    rel_import,
                                    e
                                );
                            }
                        }
                    }
                }

                if had_deletion_error {
                    return ExitCode::FAILURE;
                }
            }
        }
        Commands::Stats => {
            if cli.format == OutputFormat::Json {
                reporter::print_json(&analysis);
            } else {
                reporter::print_report(&analysis);
            }
        }
    }

    ExitCode::SUCCESS
}
