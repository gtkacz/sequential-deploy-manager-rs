# Deployer CLI

An interactive Rust CLI tool to manage and deploy groups of repositories with configurable time deltas.

## Features

-   **Interactive Selection**: Select repositories to deploy using a TUI.
-   **Visual Graph**: Real-time visualization of the execution plan and total estimated time.
-   **Configurable Groups**: Define groups of repositories and time deltas in a JSON file.
-   **Parallel Execution**: Repositories in the same group run in parallel.
-   **Staggered Scheduling**: Subsequent groups start after the largest delta of the previous group.
-   **Git Integration**: Automatically checks out `dev`, pulls, creates an empty commit, and pushes.
-   **Cross-Platform**: Works on Windows and Unix-like systems.

## Usage

1.  **Build and Run**:
    ```bash
    cargo run
    ```
    If `config.json` is missing, a sample one will be created.

2.  **Configuration**:
    The `config.json` file structure:
    ```json
    {
      "groups": [
        {
          "repositories": [
            { "path": "repo1", "delta": 0.0 },
            { "path": "repo2", "delta": 5.0 }
          ]
        }
      ],
      "root_path": "path/to/repos" // Optional, defaults to ".."
    }
    ```

3.  **Controls**:
    -   `Up` / `Down`: Navigate the list.
    -   `Space`: Toggle selection of a repository.
    -   `Enter`: Proceed to the next step (Root Path Prompt -> Confirmation).
    -   `Esc`: Go back or cancel.
    -   `q`: Quit.

4.  **Execution**:
    -   After confirmation, the tool will execute the deployment steps for each selected repository.
    -   It waits for the specified deltas between groups.

## Requirements

-   Rust (cargo)
-   Git installed and available in PATH.
