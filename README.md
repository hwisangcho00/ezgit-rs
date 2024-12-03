# EZGit-RS

**EZGit-RS** is a user-friendly terminal-based Git interface written in Rust. It simplifies common Git operations such as viewing commit logs, managing branches, committing changes, and merging branches, all within an intuitive, text-based user interface.

# Hello demo

## Features
- View and scroll through Git commit logs with detailed commit information.
- Manage branches:
  - Create, switch, and delete branches.
  - Highlight the current branch and navigate between branches.
- Commit changes with streamlined workflows.
- Merge the current branch into the `main` or `master` branch, with conflict detection and resolution guidance.
- Key guide for easy navigation and usage.
- Support for Git authentication using personal access tokens.

## Table of Contents
- [Installation](#installation)
- [Setup](#setup)
- [Usage](#usage)
- [Key Bindings](#key-bindings)

## Installation

### Prerequisites
- **Rust** (latest stable version): Install Rust using [rustup](https://rustup.rs/).
- **Git**: Ensure you have Git installed and properly configured on your system.
- **OpenSSL**: Required for the `git2` crate. Install it using your system’s package manager:
  - **Linux (Ubuntu/Debian):**
    ```bash
    sudo apt-get install libssl-dev
    ```
  - **MacOS:**
    ```bash
    brew install openssl
    ```

### Clone the Repository
```bash
git clone https://github.com/yourusername/ezgit-rs.git
cd ezgit-rs
```

### Build the Program
```bash
cargo build --release
```

### Run the Program
```bash
cargo run
```

## Setup

### Personal Access Token for Git

To use EZGit-RS with Git repositories, you need a personal access token (PAT) for authentication:

1. Generate a PAT from your Git hosting service (e.g., GitHub, GitLab).
2. Create a .env file in the root of the project directory:

```bash
touch .env
```
3. Add the following to .env:
```plaintext
GIT_USERNAME=your-username
GIT_PASSWORD=your-personal-access-token
```

## Usage

### Running the Program

#### Option 1: Build and Run with Cargo
If you have Rust installed, you can build and run the program directly with Cargo:
```bash
cargo run
```

#### Option 2: Build the Binary and Add to PATH
To make EZGit-RS available system-wide as a command, you can build the binary and add it to your PATH variable.

1. Build the Binary Navigate to the project directory and build the binary:
```bash
cargo build --release
```
2. Locate the Binary After building, the binary will be located in the target/release/ directory:
```bash
ls target/release/ezgit-rs
```
3. Move the Binary to a Directory in PATH Move the binary to a directory that is included in your system's PATH. For example:
```bash
mv target/release/ezgit-rs ~/.local/bin/ezgit
```
3-2. Make sure ~/.local/bin is in your PATH
For example, zsh user can use the following cmd
```bash
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
```
4. Run the Program You can now run the program from anywhere:
```bash
ezgit
```

# EZGit-RS Key Guide

This guide provides a summary of all key bindings available in **EZGit-RS**.

## Global Key Bindings
- **`q`**: Quit the application (requires confirmation).
- **`Esc`**: Cancel current action or return to the previous screen.
- **`Enter`**: Select an item, confirm an action, or proceed to the next step.
- **`Tab`**: Switch between panels (Commit Log and Branches).
- **`g`**: Show the key guide.

## Navigation
- **`↑` / `↓`**: Navigate up or down through items in the current panel.
- **`PageUp`**: Jump one page up in the list.
- **`PageDown`**: Jump one page down in the list.
- **`←` / `→`**: Scroll left or right in panels with longer content (e.g., Commit Log).

## Commit Details
- **`Enter` (on a commit)**: View detailed information about the selected commit.
- **`Esc` (in Commit Details)**: Return to the normal state from the Commit Details view.
- **`PageUp`**: Scroll up within the commit details.
- **`PageDown`**: Scroll down within the commit details.
- **`f`**: Filter commits by a specific file.

## Branch Management
- **`b`**: Create a new branch and switch to it.
- **`Enter` (on a branch)**: Switch to the selected branch.
- **Current Branch**: Always highlighted in bold with a distinct background color.

## Commit Workflow
- **`c`**: Start the commit workflow (stage, commit, and push changes).
  - Enter commit message, confirm, and push changes to the remote repository.

## Merge Workflow
- **`m`**: Start the merge workflow:
  - Confirm merging the current branch into the `main` or `master` branch.

## Refresh
- **`r`**: Refresh the Commit Log and Branches panels.

---

**Note**: For enhanced usability, actions like `PageUp`, `PageDown`, and branch switching are dynamically reflected in the UI.
