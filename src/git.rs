use std::path::Path;
use std::process::{Command, Stdio};
use anyhow::{Context, Result};
use crossterm::style::Stylize;

pub struct Git;

impl Git {
    pub fn run_deploy(repo_path: &Path, branch_name: &str, commit_message: &str) -> Result<()> {
        let repo_name = repo_path.file_name().unwrap_or_default().to_string_lossy();
        let prefix = format!("[{}]", repo_name).yellow();

        // 1. Switch to target branch
        println!("{} {} {}", prefix, "Checking out".cyan(), branch_name.green());
        Self::run_command(repo_path, "git", &["checkout", "-q", branch_name])
            .context(format!("Failed to checkout {} branch", branch_name))?;

        // 2. Pull latest changes
        println!("{} {}", prefix, "Pulling latest changes...".cyan());
        Self::run_command(repo_path, "git", &["pull", "-q"])
            .context("Failed to pull latest changes")?;

        // 3. Create empty commit
        println!("{} {}", prefix, "Creating empty commit...".cyan());
        Self::run_command(repo_path, "git", &["commit", "--allow-empty", "-q", "-m", commit_message])
            .context("Failed to create empty commit")?;

        // 4. Push changes
        println!("{} {}", prefix, "Pushing changes...".cyan());
        Self::run_command(repo_path, "git", &["push", "-q"])
            .context("Failed to push changes")?;

        Ok(())
    }

    fn run_command(cwd: &Path, program: &str, args: &[&str]) -> Result<()> {
        let status = Command::new(program)
            .args(args)
            .current_dir(cwd)
            .stdout(Stdio::null())
            .status()
            .with_context(|| format!("Failed to execute {} {:?}", program, args))?;

        if !status.success() {
            anyhow::bail!("Command {} {:?} failed with status {}", program, args, status);
        }

        Ok(())
    }
    
    pub fn check_exists(repo_path: &Path) -> bool {
        repo_path.exists() && repo_path.join(".git").exists()
    }
}
