use std::path::Path;
use std::process::Command;
use anyhow::{Context, Result};

pub struct Git;

impl Git {
    pub fn run_deploy(repo_path: &Path, commit_message: &str) -> Result<()> {
        // 1. Switch to dev branch
        Self::run_command(repo_path, "git", &["checkout", "dev"])
            .context("Failed to checkout dev branch")?;

        // 2. Pull latest changes
        Self::run_command(repo_path, "git", &["pull"])
            .context("Failed to pull latest changes")?;

        // 3. Create empty commit
        Self::run_command(repo_path, "git", &["commit", "--allow-empty", "-m", commit_message])
            .context("Failed to create empty commit")?;

        // 4. Push changes
        Self::run_command(repo_path, "git", &["push"])
            .context("Failed to push changes")?;

        Ok(())
    }

    fn run_command(cwd: &Path, program: &str, args: &[&str]) -> Result<()> {
        let status = Command::new(program)
            .args(args)
            .current_dir(cwd)
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
