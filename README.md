# Deployer CLI

An interactive Rust-based tool for orchestrating multi-repository deployments with precise timing and parallel execution.

## Features

- **Interactive TUI**: Navigate and select repositories using a clean terminal interface.
- **Visual Execution Graph**: Real-time preview of your deployment timeline and total estimated duration based on your selection.
- **Parallel & Staggered Execution**:
  - Repositories in the same group deploy **in parallel**.
  - Subsequent groups start only after the previous group's delay period (configurable per repository).
- **Background Execution**: Run the deployment process in the background, freeing up your terminal.
- **Timer Control**: Skip waiting periods manually if needed.
- **Automated Git Workflow**: Automatically performs `checkout`, `pull`, `empty commit`, and `push` for each target.
- **Safety Checks**: Validates repository existence before starting and requires explicit confirmation.
- **Configurable**: Simple JSON configuration for repository grouping, branch names, and commit messages.

## Installation

### Linux / macOS (curl)

```sh
curl -fsSL https://raw.githubusercontent.com/gtkacz/sequential-deploy-manager-rs/main/scripts/install.sh | bash
```

This detects your OS and architecture, downloads the latest release binary to `/usr/local/bin`, and makes it executable. Set `INSTALL_DIR` to change the target:

```sh
INSTALL_DIR=~/.local/bin curl -fsSL https://raw.githubusercontent.com/gtkacz/sequential-deploy-manager-rs/main/scripts/install.sh | bash
```

### Windows (PowerShell)

```powershell
irm https://raw.githubusercontent.com/gtkacz/sequential-deploy-manager-rs/main/scripts/install.ps1 | iex
```

This downloads the latest release to `%LOCALAPPDATA%\cgen\` and adds it to your user PATH.

## Usage

1.  **Run the tool**:
    ```bash
    deployer
    ```
    *(If `deployer.config.json` is missing, a sample file will be created automatically.)*

2.  **Interactive Steps**:
    -   **Select**: Use `Up`/`Down` to navigate and `Space` to toggle repositories.
    -   **Review**: Watch the "Execution Graph" update in real-time to see the plan.
    -   **Confirm**: Press `Enter` to proceed. You can verify/edit the root path for your repositories.
    -   **Deploy**: Confirm the prompt to start the automated deployment process.

## Configuration (`deployer.config.json`)

The tool is driven by a `deployer.config.json` file in the working directory, e.g.:

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
-   **`delta`**: Time in minutes to wait *after* this group starts before the *next* group can begin (or after the last group finishes). The tool waits for the longest delta in the current group.
-   **`dry`**: (Optional) If `true`, the repository will be part of the delta calculation but no git commands will be executed. Defaults to `false`. This can also be toggled at runtime using the `d` key.
-   **`root_path`**: (Optional) The parent folder containing your repositories. Defaults to `..`.
-   **`branch`**: (Optional) Defaults to `dev`.
-   **`commit_message`**: (Optional) Defaults to `Force deploy of dev`.

## Controls

| Key | Action |
| :--- | :--- |
| `↑` / `↓` | Navigate list |
| `Space` | Toggle selection |
| `d` | Toggle dry run |
| `c` | Clear all selections |
| `b` | Run in background (detached) |
| `Enter` | Proceed / Confirm |
| `Esc` | Go back / Cancel |
| `q` | Quit |

### During Execution (Wait Period)

| Key | Action |
| :--- | :--- |
| `s` | Skip remaining wait time |
