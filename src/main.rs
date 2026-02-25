use std::path::PathBuf;
use std::time::Duration;
use anyhow::Result;
use clap::Parser;
use tokio::time::sleep;

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
    #[arg(short, long, default_value = "config.json")]
    config: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // 1. Load Config
    // Check if config file exists, if not create a dummy one for demo purposes
    if !args.config.exists() {
        println!("Config file not found at {:?}. Creating a sample config.json...", args.config);
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
  ]
}"#;
        std::fs::write(&args.config, sample_config)?;
    }

    let config = Config::load(&args.config)?;

    // 2. Run TUI
    let mut app = App::new(config.clone());
    app.run()?;

    if !app.confirmed {
        println!("Execution cancelled.");
        return Ok(())
    }

    println!("Starting execution...");
    
    let root_path = PathBuf::from(&app.root_path);
    let mut handles = Vec::new();

    // Iterate through groups
    for (g_idx, group) in config.groups.iter().enumerate() {
        let mut group_max_delta = 0.0;
        let mut group_has_selected = false;

        // Collect selected repos in this group
        let mut group_tasks = Vec::new();
        
        for (r_idx, repo) in group.repositories.iter().enumerate() {
            if app.selected_repos.contains(&(g_idx, r_idx)) {
                group_has_selected = true;
                if repo.delta > group_max_delta {
                    group_max_delta = repo.delta;
                }
                
                let repo_path = root_path.join(&repo.path);
                let repo_path_clone = repo_path.clone();
                let repo_name = repo.path.clone();

                // Check existence - already validated in TUI but good to check again
                if !Git::check_exists(&repo_path) {
                    eprintln!("Warning: Repository {} does not exist at {:?}", repo_name, repo_path);
                    continue;
                }

                println!("Spawning task for {}", repo_name);
                
                let handle = tokio::spawn(async move {
                    println!("Running {}", repo_name);
                    match Git::run_deploy(&repo_path_clone, "Force deploy of dev") {
                        Ok(_) => println!("Successfully deployed {}", repo_name),
                        Err(e) => eprintln!("Failed to deploy {}: {:?}", repo_name, e),
                    }
                });
                group_tasks.push(handle);
            }
        }
        
        handles.extend(group_tasks);

        if group_has_selected {
            let delay_seconds = group_max_delta * 60.0;
            if delay_seconds > 0.0 {
                println!("Waiting {:.1} seconds before next group...", delay_seconds);
                sleep(Duration::from_secs_f64(delay_seconds)).await;
            }
        }
    }

    // Wait for all tasks to complete
    for handle in handles {
        let _ = handle.await;
    }

    println!("All tasks completed.");
    Ok(())
}
