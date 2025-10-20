// Main entry point for epcheck
use clap::{CommandFactory, Parser};
use epcheck::{run, Cli};
use epcheck::cli::{Commands, Shell};
use std::process;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Check(args)) => {
            // Initialize tracing
            tracing_subscriber::fmt()
                .with_max_level(tracing::Level::INFO)
                .init();

            if let Err(e) = run(args).await {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
        Some(Commands::Completions { shell, install }) => {
            let mut cmd = Cli::command();
            let shell_type = match shell {
                Shell::Bash => clap_complete::Shell::Bash,
                Shell::Zsh => clap_complete::Shell::Zsh,
                Shell::Fish => clap_complete::Shell::Fish,
                Shell::PowerShell => clap_complete::Shell::PowerShell,
                Shell::Elvish => clap_complete::Shell::Elvish,
            };

            if install {
                let completion_path = get_completion_path(shell)?;
                std::fs::create_dir_all(completion_path.parent().unwrap())?;
                let mut file = std::fs::File::create(&completion_path)?;
                clap_complete::generate(shell_type, &mut cmd, "epcheck", &mut file);
                println!("Completions installed to: {}", completion_path.display());
                println!("You may need to restart your shell or source your shell config for changes to take effect.");
            } else {
                clap_complete::generate(shell_type, &mut cmd, "epcheck", &mut std::io::stdout());
            }
        }
        None => {
            // Default to check command for backward compatibility
            let args = epcheck::CheckArgs::parse();
            
            // Initialize tracing
            tracing_subscriber::fmt()
                .with_max_level(tracing::Level::INFO)
                .init();

            if let Err(e) = run(args).await {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
    }

    Ok(())
}

fn get_completion_path(shell: Shell) -> anyhow::Result<PathBuf> {
    let home = std::env::var("HOME")?;
    let home_path = PathBuf::from(home);

    match shell {
        Shell::Bash => {
            // Prefer user directory first
            let user_path = home_path.join(".bash_completion.d").join("epcheck");
            if user_path.parent().map_or(false, |p| p.exists()) || std::fs::create_dir_all(user_path.parent().unwrap()).is_ok() {
                return Ok(user_path);
            }

            // Check if system directories exist and are writable
            let system_paths = vec![
                PathBuf::from("/usr/local/etc/bash_completion.d").join("epcheck"),
                PathBuf::from("/etc/bash_completion.d").join("epcheck"),
            ];

            for path in system_paths {
                if let Some(parent) = path.parent() {
                    if parent.exists() && std::fs::metadata(parent).map_or(false, |m| !m.permissions().readonly()) {
                        return Ok(path);
                    }
                }
            }

            // Default to user directory
            Ok(user_path)
        }
        Shell::Zsh => {
            // Prefer user directory first
            let user_path = home_path.join(".zsh").join("completions").join("_epcheck");
            if user_path.parent().map_or(false, |p| p.exists()) || std::fs::create_dir_all(user_path.parent().unwrap()).is_ok() {
                return Ok(user_path);
            }

            // Check if system directories exist and are writable
            let system_paths = vec![
                PathBuf::from("/usr/local/share/zsh/site-functions").join("_epcheck"),
                PathBuf::from("/usr/share/zsh/site-functions").join("_epcheck"),
            ];

            for path in system_paths {
                if let Some(parent) = path.parent() {
                    if parent.exists() && std::fs::metadata(parent).map_or(false, |m| !m.permissions().readonly()) {
                        return Ok(path);
                    }
                }
            }

            // Default to user directory
            Ok(user_path)
        }
        Shell::Fish => {
            let fish_completions = home_path
                .join(".config")
                .join("fish")
                .join("completions");
            let path = fish_completions.join("epcheck.fish");
            if path.parent().map_or(false, |p| p.exists()) || std::fs::create_dir_all(path.parent().unwrap()).is_ok() {
                return Ok(path);
            }
            Ok(path)
        }
        Shell::PowerShell => {
            // PowerShell profile directory
            let ps_profile = std::env::var("PSModulePath")
                .ok()
                .and_then(|path| std::env::split_paths(&path).next())
                .unwrap_or_else(|| home_path.join("Documents").join("WindowsPowerShell"));
            let path = ps_profile.join("epcheck.ps1");
            if path.parent().map_or(false, |p| p.exists()) || std::fs::create_dir_all(path.parent().unwrap()).is_ok() {
                return Ok(path);
            }
            Ok(path)
        }
        Shell::Elvish => {
            let elvish_rc = home_path
                .join(".config")
                .join("elvish")
                .join("rc.elv");
            if elvish_rc.parent().map_or(false, |p| p.exists()) || std::fs::create_dir_all(elvish_rc.parent().unwrap()).is_ok() {
                return Ok(elvish_rc);
            }
            Ok(elvish_rc)
        }
    }
}