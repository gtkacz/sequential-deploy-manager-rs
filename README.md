# Deployer CLI

An interactive Rust-based tool for orchestrating multi-repository deployments with precise timing and parallel execution.

## Features

- **Interactive TUI**: Navigate and select repositories using a clean terminal interface.
- **Visual Execution Graph**: Real-time preview of your deployment timeline and total estimated duration based on your selection.
- **Parallel & Staggered Execution**:
  - Repositories in the same group deploy **in parallel**.
  - Subsequent groups start only after the previous group's delay period (configurable per repository).
- **Automated Git Workflow**: Automatically performs `checkout`, `pull`, `empty commit`, and `push` for each target.
- **Safety Checks**: Validates repository existence before starting and requires explicit confirmation.
- **Configurable**: Simple JSON configuration for repository grouping, branch names, and commit messages.

## Usage

1.  **Run the tool**:
    ```bash
    cargo run
    ```
    *(If `config.json` is missing, a sample file will be created automatically.)*

2.  **Interactive Steps**:
    -   **Select**: Use `Up`/`Down` to navigate and `Space` to toggle repositories.
    -   **Review**: Watch the "Execution Graph" update in real-time to see the plan.
    -   **Confirm**: Press `Enter` to proceed. You can verify/edit the root path for your repositories.
    -   **Deploy**: Confirm the prompt to start the automated deployment process.

## Configuration (`config.json`)

The tool is driven by a `config.json` file in the working directory:

```json
{
  "groups": [
    {
      "repositories": [
        { "path": "backend-api", "delta": 0.5 },
        { "path": "worker-service", "delta": 2.0 }
      ]
    },
    {
      "repositories": [
        { "path": "frontend-app", "delta": 0.0 }
      ]
    }
  ],
  "root_path": "../projects",  // Base directory for all repositories
  "branch": "dev",             // Target branch for deployment
  "commit_message": "Trigger deploy" // Commit message for the empty commit
}
```

-   **`groups`**: Arrays of repositories. All repos in a group start simultaneously.
-   **`delta`**: Time in minutes to wait *after* this group starts before the *next* group can begin. The tool waits for the longest delta in the current group.
-   **`root_path`**: (Optional) The parent folder containing your repositories. Defaults to `..`.
-   **`branch`**: (Optional) Defaults to `dev`.
-   **`commit_message`**: (Optional) Defaults to `Force deploy of dev`.

## Controls

| Key | Action |
| :--- | :--- |
| `↑` / `↓` | Navigate list |
| `Space` | Toggle selection |
| `c` | Clear all selections |
| `Enter` | Proceed / Confirm |
| `Esc` | Go back / Cancel |
| `q` | Quit |
