use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode},
    style::Stylize,
    terminal::{disable_raw_mode, enable_raw_mode},
};
use std::collections::HashSet;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use tokio::time::sleep;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

mod config;
mod git;
mod tui;

use config::Config;
use git::Git;
use tui::App;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the configuration file
    #[arg(short, long, default_value = "deployer.config.json")]
    config: PathBuf,

    /// Run in headless mode (no TUI)
    #[arg(long, default_value_t = false)]
    headless: bool,

    /// Delete config file after execution
    #[arg(long, default_value_t = false)]
    delete_config: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // 1. Load Config
    // Check if config file exists, if not create a dummy one for demo purposes
    if !args.config.exists() {
        println!(
            "{}",
            format!(
                "Config file not found at {:?}. Creating a sample deployer.config.json...",
                args.config
            )
            .yellow()
        );
        let sample_config = r#"{
  "groups": [
    {
      "repositories": [
        { "path": "repo1", "delta": 0.1 },
        { "path": "repo2", "delta": 0.2 }
      ]
    },
    {
      "repositories": [
        { "path": "repo3", "delta": 0.1 }
      ]
    }
  ],
  "branch": "dev",
  "commit_message": "Force deploy of dev"
}"#;
        std::fs::write(&args.config, sample_config)?;
    }

    let config = Config::load(&args.config)?;

    let mut selected_repos = HashSet::new();
    let mut dry_repos = HashSet::new();
    let mut root_path_str = config.root_path.clone().unwrap_or_else(|| ".".to_string());

    if !args.headless {
        // 2. Run TUI
        let mut app = App::new(config.clone());
        app.run()?;

        if app.run_in_background {
            // Generate temp config with selected repos
            let timestamp = chrono::Local::now().format("%Y%m%d%H%M%S").to_string();
            let temp_config_path = PathBuf::from(format!("config_bg_{}.json", timestamp));

            // Filter config
            let mut new_groups = Vec::new();
            for (g_idx, group) in config.groups.iter().enumerate() {
                let mut new_repos = Vec::new();
                for (r_idx, repo) in group.repositories.iter().enumerate() {
                    if app.selected_repos.contains(&(g_idx, r_idx)) {
                        let mut r = repo.clone();
                        // Inherit dry status from TUI selection
                        r.dry = app.dry_repos.contains(&(g_idx, r_idx));
                        new_repos.push(r);
                    }
                }
                if !new_repos.is_empty() {
                    new_groups.push(config::Group {
                        repositories: new_repos,
                    });
                }
            }

            let new_config = Config {
                groups: new_groups,
                root_path: Some(app.root_path.clone()),
                branch: config.branch.clone(),
                commit_message: config.commit_message.clone(),
            };

            let json = serde_json::to_string_pretty(&new_config)?;
            std::fs::write(&temp_config_path, json)?;

            println!("{}", "Starting background process...".blue());

            // Spawn detached process
            let exe = std::env::current_exe()?;

            // Redirect output to log file
            let log_file = File::create("deployer.log")?;

            let mut cmd = Command::new(exe);
            cmd.arg("--config")
                .arg(&temp_config_path)
                .arg("--headless")
                .arg("--delete-config")
                .stdout(Stdio::from(log_file.try_clone()?))
                .stderr(Stdio::from(log_file));

            #[cfg(target_os = "windows")]
            {
                const DETACHED_PROCESS: u32 = 0x00000008;
                const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;
                cmd.creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP);
            }

            cmd.spawn()?;

            println!(
                "{}",
                "Background process started. Logs redirected to deployer.log".green()
            );
            return Ok(());
        }

        if !app.confirmed {
            println!("{}", "Execution cancelled.".red());
            return Ok(());
        }

        selected_repos = app.selected_repos;
        dry_repos = app.dry_repos;
        root_path_str = app.root_path;
    } else {
        // Headless mode: select all repos in the config
        for (g_idx, group) in config.groups.iter().enumerate() {
            for (r_idx, repo) in group.repositories.iter().enumerate() {
                selected_repos.insert((g_idx, r_idx));
                if repo.dry {
                    dry_repos.insert((g_idx, r_idx));
                }
            }
        }
    }

    println!("{}", "Starting execution...".blue());

    let root_path = PathBuf::from(&root_path_str);
    let mut handles = Vec::new();

    // Iterate through groups
    for (g_idx, group) in config.groups.iter().enumerate() {
        let mut group_max_delta = 0.0;
        let mut group_has_selected = false;

        // Collect selected repos in this group
        let mut group_tasks = Vec::new();

        for (r_idx, repo) in group.repositories.iter().enumerate() {
            if selected_repos.contains(&(g_idx, r_idx)) {
                group_has_selected = true;
                if repo.delta > group_max_delta {
                    group_max_delta = repo.delta;
                }

                let repo_path = root_path.join(&repo.path);
                let repo_path_clone = repo_path.clone();
                let repo_name = repo.path.clone();
                let branch_name = config.branch.clone();
                let commit_msg = config.commit_message.clone();
                let is_dry_run = dry_repos.contains(&(g_idx, r_idx));

                // Check existence - already validated in TUI but good to check again
                if !Git::check_exists(&repo_path) {
                    eprintln!(
                        "{}",
                        format!(
                            "Warning: Repository {} does not exist at {:?}",
                            repo_name, repo_path
                        )
                        .yellow()
                    );
                    continue;
                }

                println!(
                    "{} {}",
                    "Spawning task for".magenta(),
                    repo_name.as_str().cyan()
                );

                let handle = tokio::spawn(async move {
                    if is_dry_run {
                        println!(
                            "{} {}: {}",
                            "Dry run for".yellow(),
                            repo_name.as_str().cyan(),
                            "Skipping git operations.".yellow()
                        );
                        return;
                    }
                    println!("{} {}", "Running".blue(), repo_name.as_str().cyan());
                    match Git::run_deploy(&repo_path_clone, &branch_name, &commit_msg) {
                        Ok(_) => println!(
                            "{} {}",
                            "Successfully deployed".green(),
                            repo_name.as_str().cyan()
                        ),
                        Err(e) => eprintln!(
                            "{} {}: {:?}",
                            "Failed to deploy".red(),
                            repo_name.as_str().cyan(),
                            e
                        ),
                    }
                });
                group_tasks.push(handle);
            }
        }

        handles.extend(group_tasks);

        if group_has_selected {
            // Check if we should wait
            let delay_seconds = group_max_delta * 60.0;
            if delay_seconds > 0.0 {
                let start_wait = Instant::now();
                let duration = Duration::from_secs_f64(delay_seconds);

                // Enable raw mode for input detection
                if !args.headless {
                    enable_raw_mode()?;
                }

                let mut skipped = false;
                while start_wait.elapsed() < duration {
                    let remaining = duration - start_wait.elapsed();
                    print!(
                        "\r{} {:.0} {}   ",
                        "Waiting".yellow(),
                        remaining.as_secs_f64().ceil(),
                        "seconds before next group... (press 's' to skip)".yellow()
                    );
                    std::io::stdout().flush()?;

                    if !args.headless {
                        if event::poll(Duration::from_millis(100))?
                            && let Event::Key(key) = event::read()?
                            && key.code == KeyCode::Char('s')
                        {
                            skipped = true;
                            break;
                        }
                    } else {
                        sleep(Duration::from_millis(100)).await;
                    }
                }

                if !args.headless {
                    disable_raw_mode()?;
                }

                println!(); // New line after wait is done
                if skipped {
                    println!("{}", "Skipped wait.".yellow());
                }
            }
        }
    }

    // Wait for all tasks to complete
    for handle in handles {
        let _ = handle.await;
    }

    if args.delete_config {
        std::fs::remove_file(&args.config)?;
    }

    println!("{}", "All tasks completed.".green());
    Ok(())
}
