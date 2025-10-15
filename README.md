# local-bin

A collection of personal command-line utilities designed to streamline common development and maintenance tasks. These scripts are intended for use in Linux or macOS environments.

## Installation

### Prerequisites

Before installing these scripts, ensure you have the following dependencies installed:
- `bash` (usually pre-installed on Mac/Linux)
- `gh` (GitHub CLI) - for scripts that interact with GitHub
- `ollama` - for AI-powered scripts (ai-story, ai_readme, gcm, labelai)
- `fd` - for scripts that search files (lspkg, usersecrets)
- `jq` - for JSON parsing (lspkg, depcheck)
- `sqlite3` - for the epcheck test suite database functionality

### Installation Steps

1.  **Clone the repository** to a local directory. We recommend using `~/bin` or `~/.local/bin`:

    ```bash
    # Option 1: Clone to ~/bin
    git clone https://github.com/SjoenH/local-bin.git ~/bin

    # Option 2: Clone to ~/.local/bin
    git clone https://github.com/SjoenH/local-bin.git ~/.local/bin
    ```

2.  **Make the scripts executable** (if not already):

    ```bash
    # For ~/bin
    chmod +x ~/bin/*

    # For ~/.local/bin
    chmod +x ~/.local/bin/*
    ```

3.  **Add the directory to your PATH**:

    For **Bash** users, add this line to your `~/.bashrc` or `~/.bash_profile`:
    ```bash
    # For ~/bin
    export PATH="$HOME/bin:$PATH"

    # For ~/.local/bin
    export PATH="$HOME/.local/bin:$PATH"
    ```

    For **Zsh** users (default on macOS), add this line to your `~/.zshrc`:
    ```bash
    # For ~/bin
    export PATH="$HOME/bin:$PATH"

    # For ~/.local/bin
    export PATH="$HOME/.local/bin:$PATH"
    ```

4.  **Reload your shell configuration**:

    ```bash
    # For Bash
    source ~/.bashrc  # or source ~/.bash_profile

    # For Zsh
    source ~/.zshrc
    ```

5.  **Verify the installation**:

    ```bash
    # Test that the scripts are in your PATH
    which ai-story
    which lspkg
    which epcheck

    # Try running a script (this will show the help message)
    ai-story help

    # Test epcheck functionality
    epcheck --help

    # Test the epcheck test suite (if in the repository)
    cd testbench/v1.0 && ./run-tests.sh && ./test-browser.sh list
    ```

### Alternative Installation

If you prefer to keep the repository elsewhere, you can create symbolic links:

```bash
# Clone to any location
git clone https://github.com/SjoenH/local-bin.git ~/projects/local-bin

# Create symbolic links to a directory in your PATH
mkdir -p ~/.local/bin
ln -s ~/projects/local-bin/* ~/.local/bin/

# Make sure ~/.local/bin is in your PATH (see step 3 above)
```

## Scripts

### ai\_story
-----------------

A Bash script that generates a prompt based on the specified issue or pull request number, uses an LLaMA model to generate suggestions, and displays these suggestions along with the issue title and number.

Usage:
```bash
./ai_story --issue <number> | --pr <number>
```

### ai\_readme
----------------

A script that generates a README file for a project by collecting descriptions of each file in the project directory and combining them into a single content string. The output can be saved as a Markdown file (`README.md`).

Usage:
```bash
./ai_readme
```

### depcheck
-------------

A script that displays the dependencies of a given package. It searches for `package.json` files containing the specified dependency.

Usage:
```bash
./depcheck <package_name>
```

### epcheck
------------

A script that checks which OpenAPI endpoints are used in the codebase and where they are referenced. It analyzes your project to show endpoint usage statistics, helping identify unused API endpoints and track endpoint adoption across your codebase.

#### Features:
- Fast searching using ripgrep (falls back to grep if not available)
- Multiple output formats: table, CSV, JSON
- Interactive endpoint selection with fzf
- Pattern-based endpoint filtering
- Quick mode for faster results on large codebases
- Detailed file reference listings

#### Usage:
```bash
./epcheck [OPTIONS]

# Examples:
./epcheck                                    # Show all endpoints with full file lists
./epcheck --unused-only                     # Show only unused endpoints
./epcheck --pattern "Saker.*Drivere"        # Filter endpoints by regex pattern
./epcheck --format csv                      # Output in CSV format
./epcheck --interactive                     # Interactive mode with fzf
./epcheck --quick --truncate                # Fast mode with compact output
```

### epcheck Test Suite
---------------------

The epcheck test suite validates the functionality of the epcheck script across different scenarios. The test suite now uses a SQLite database for robust data storage and includes a comprehensive CLI browser tool for analyzing test results.

#### Database Features:
- **SQLite Storage**: All test results stored in a relational database (`testbench.db`)
- **Complete History**: Full test execution history with timestamps and performance metrics
- **Validation Tracking**: Detailed validation results with diff output for failures
- **Performance Monitoring**: Execution time and memory usage tracking

#### Test Browser CLI Tool:
A powerful command-line tool for browsing and comparing test runs stored in the database.

**Commands:**
- `list [limit]` - Show recent test runs with pass/fail counts
- `show <run_id>` - Display detailed test run information
- `compare <id1> <id2>` - Side-by-side comparison of test runs
- `failures <run_id>` - Show validation failures with diff output
- `stats` - Overall statistics and recent performance
- `trends [days]` - Performance trends over time
- `export <run_id> <format>` - Export data as JSON or CSV

#### Usage:
```bash
# Run the test suite
cd testbench/v1.0
./run-tests.sh

# Browse test results
./test-browser.sh list                    # Show recent test runs
./test-browser.sh show 1                  # Show details of test run 1
./test-browser.sh compare 1 2             # Compare test runs 1 and 2
./test-browser.sh failures 1              # Show failures in test run 1
./test-browser.sh stats                   # Show overall statistics
./test-browser.sh export 1 json           # Export test run 1 as JSON
```

#### Database Schema:
The SQLite database includes these tables:
- `test_runs` - Overall test execution metadata
- `test_executions` - Individual test results with performance metrics
- `test_validations` - Output validation results and differences
- `expected_results` - Static expected test outputs
- `performance_benchmarks` - Performance tracking data

### gcm
----------

A Bash script that generates commit or pull request messages using an LLaMA model. It prompts the user to select a message type, generates a message based on changes made since the last commit or main branch merge, and allows the user to edit or retry the generated message before committing or copying it to the clipboard.

Usage:
```bash
./gcm
```

### labelai
----------------

A script that fetches issues from a GitHub repository, generates a prompt using existing labels and issue details, uses an LLaMA 3.1 model to generate label suggestions based on this prompt, and displays these suggestions along with the issue title and number.

Usage:
```bash
./labelai
```

### lspkg
------------

A Bash script that lists npm packages in the current directory and its subdirectories, displaying package metadata such as name, version, description, and path. It takes command-line arguments to customize its behavior.

Usage:
```bash
./lspkg [options]
```

### usersecrets
-----------------

A script that searches for `.csproj` files in a specified directory (or current directory if not provided) and its subdirectories, listing the locations of secrets associated with them. It uses `fd` to find `.csproj` files and then extracts their corresponding UserSecretsId using `awk`.

Usage:
```bash
./usersecrets [directory]
```

## Conclusion

This `local-bin` collection simplifies your development and maintenance workflow, boosting productivity by automating common tasks.

For setup, refer to the [Installation](#installation) section.
