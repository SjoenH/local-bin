#!/bin/bash
# Setup script for local-bin utilities
# This script automates the installation process for all tools

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
print_header() {
    echo -e "${BLUE}================================${NC}"
    echo -e "${BLUE}  $1${NC}"
    echo -e "${BLUE}================================${NC}"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

print_info() {
    echo -e "${BLUE}ℹ $1${NC}"
}

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Detect OS
detect_os() {
    case "$(uname -s)" in
        Darwin)
            echo "macos"
            ;;
        Linux)
            echo "linux"
            ;;
        CYGWIN*|MINGW32*|MSYS*|MINGW*)
            echo "windows"
            ;;
        *)
            echo "unknown"
            ;;
    esac
}

# Main setup function
main() {
    print_header "local-bin Setup Script"

    OS=$(detect_os)
    print_info "Detected OS: $OS"

    # Check prerequisites
    print_header "Checking Prerequisites"

    # Check for bash
    if command_exists bash; then
        print_success "Bash found"
    else
        print_error "Bash not found. Please install bash first."
        exit 1
    fi

    # Check for Rust (required for epcheck)
    if command_exists cargo; then
        print_success "Rust/Cargo found"
        HAS_RUST=true
    else
        print_warning "Rust not found. epcheck will not be available."
        print_info "Install Rust from: https://rustup.rs/"
        HAS_RUST=false
    fi

    # Check for optional tools
    OPTIONAL_TOOLS=("gh" "ollama" "fd" "jq" "sqlite3" "fzf" "ripgrep")
    for tool in "${OPTIONAL_TOOLS[@]}"; do
        if command_exists "$tool"; then
            print_success "$tool found"
        else
            print_warning "$tool not found (optional)"
        fi
    done

    # Build Rust binaries
    if [ "$HAS_RUST" = true ]; then
        print_header "Building Rust Binaries"

        RUST_PROJECTS=("epcheck-src" "usersecrets-src" "lspkg-src" "epcheck-src/testbench-tui")
        for project in "${RUST_PROJECTS[@]}"; do
            if [ -d "$project" ]; then
                print_info "Building $project (this may take a moment)..."
                # Run builds in a subshell so we don't change the current working directory
                if (cd "$project" && cargo build --release); then
                    print_success "$project built successfully"
                else
                    print_error "Failed to build $project"
                    exit 1
                fi
            else
                print_warning "$project directory not found, skipping"
            fi
        done
    fi

    # Make scripts executable
    print_header "Making Scripts Executable"

    # Find all executable files (excluding certain directories)
    find . -type f -name "*.sh" -o -name "*.bash" -o -name "epcheck*" -o -name "ai-*" -o -name "depcheck" -o -name "lspkg*" -o -name "usersecrets*" -o -name "testbench-tui*" -o -name "gcm" -o -name "labelai" -o -name "webi" | while read -r file; do
        # Skip files in target directories, .git, etc.
        if [[ "$file" != *"/target/"* && "$file" != *"/.git/"* && "$file" != *"/node_modules/"* && "$file" != *"/testbench/"* ]]; then
            if [ -f "$file" ] && [ ! -x "$file" ]; then
                chmod +x "$file"
                print_success "Made executable: $file"
            fi
        fi
    done

    # Special handling for Rust binary symlinks
    RUST_TOOLS=("epcheck:epcheck-src" "usersecrets:usersecrets-src" "lspkg:lspkg-src" "testbench-tui:epcheck-src")
    for tool_info in "${RUST_TOOLS[@]}"; do
        tool_name="${tool_info%%:*}"
        project_dir="${tool_info##*:}"

        if [ -L "$tool_name" ] && [ "$HAS_RUST" = true ]; then
            print_success "$tool_name symlink is properly configured"
        elif [ "$HAS_RUST" = true ] && [ -f "$project_dir/target/release/$tool_name" ]; then
            print_info "Creating $tool_name symlink..."
            ln -sf "$project_dir/target/release/$tool_name" "$tool_name"
            print_success "$tool_name symlink created"
        fi
    done

    # Special handling for script symlinks
    if [ -f "scripts/slack-pr-message.sh" ]; then
        if [ -L "prm" ]; then
            print_success "prm symlink is properly configured"
        else
            print_info "Creating prm symlink..."
            ln -s "scripts/slack-pr-message.sh" "prm"
            print_success "prm symlink created"
        fi
    fi

    if [ -f "scripts/start.sh" ]; then
        if [ -L "start" ]; then
            print_success "start symlink is properly configured"
        else
            print_info "Creating start symlink..."
            ln -s "scripts/start.sh" "start"
            print_success "start symlink created"
        fi
    fi

    if [ -f "scripts/sbd.sh" ]; then
        if [ -L "sbd" ]; then
            print_success "sbd symlink is properly configured"
        else
            print_info "Creating sbd symlink..."
            ln -s "scripts/sbd.sh" "sbd"
            print_success "sbd symlink created"
        fi
    fi

    # Check PATH
    print_header "PATH Configuration"

    SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    CURRENT_SHELL=$(basename "$SHELL")

    # Check if script directory is in PATH
    if echo "$PATH" | tr ':' '\n' | grep -q "^$SCRIPT_DIR$"; then
        print_success "Current directory is in PATH"
    else
        print_warning "Current directory is not in PATH"

        case "$CURRENT_SHELL" in
            "bash")
                SHELL_RC="$HOME/.bashrc"
                ;;
            "zsh")
                SHELL_RC="$HOME/.zshrc"
                ;;
            "fish")
                SHELL_RC="$HOME/.config/fish/config.fish"
                ;;
            *)
                SHELL_RC=""
                ;;
        esac

        if [ -n "$SHELL_RC" ]; then
            print_info "To add to PATH, add this line to $SHELL_RC:"
            echo "export PATH=\"$SCRIPT_DIR:\$PATH\""
            print_info "Then run: source $SHELL_RC"
        else
            print_info "To add to PATH manually:"
            echo "export PATH=\"$SCRIPT_DIR:\$PATH\""
        fi
    fi

    # Final verification
    print_header "Verification"

    # Test a few key scripts
    if [ -x "./epcheck" ] && [ "$HAS_RUST" = true ]; then
        if ./epcheck --version >/dev/null 2>&1; then
            print_success "epcheck is working"
        else
            print_warning "epcheck found but not working properly"
        fi
    elif [ -x "./epcheck-bash" ]; then
        print_success "epcheck-bash is available"
    fi

    if [ -x "./depcheck" ]; then
        print_success "depcheck is executable"
    fi

    if [ -x "./lspkg" ]; then
        if [ "$HAS_RUST" = true ] && [ -L "./lspkg" ]; then
            print_success "lspkg (Rust) is working"
        else
            print_success "lspkg is executable"
        fi
    fi

    if [ -x "./usersecrets" ]; then
        if [ "$HAS_RUST" = true ] && [ -L "./usersecrets" ]; then
            print_success "usersecrets (Rust) is working"
        else
            print_success "usersecrets is executable"
        fi
    fi

    if [ -x "./testbench-tui" ]; then
        if [ "$HAS_RUST" = true ] && [ -L "./testbench-tui" ]; then
            print_success "testbench-tui (Rust) is working"
        else
            print_success "testbench-tui is executable"
        fi
    fi

    if [ -x "./prm" ] || [ -L "./prm" ]; then
        print_success "prm (PR Slack message) is available"
    fi

    if [ -x "./start" ] || [ -L "./start" ]; then
        print_success "start (development tmux session) is available"
    fi

    if [ -x "./sbd" ] || [ -L "./sbd" ]; then
        print_success "sbd (Docker build & deploy) is available"
    fi

    # Summary
    print_header "Setup Complete!"

    echo -e "${GREEN}Available tools:${NC}"
    echo "  - epcheck (Rust) - Fast OpenAPI endpoint checker"
    echo "  - epcheck-bash - Bash version of epcheck"
    echo "  - testbench-tui (Rust) - TUI for running and visualizing epcheck testbench results"
    echo "  - depcheck - Check package dependencies"
    echo "  - lspkg (Rust) - Fast npm package lister"
    echo "  - lspkg-bash - Bash version of lspkg"
    echo "  - usersecrets (Rust) - Fast .NET user secrets manager"
    echo "  - usersecrets-bash - Bash version of usersecrets"
    echo "  - ai-story - Generate stories from GitHub issues"
    echo "  - ai-readme - Generate README files"
    echo "  - gcm - Generate commit messages"
    echo "  - labelai - Generate GitHub issue labels"
    echo "  - webi - Web installer script"
    echo "  - prm - Generate Slack message for current PR"
    echo "  - start - Start development environment with tmux"
    echo "  - sbd - Docker build and deploy to Kubernetes"

    if [ "$HAS_RUST" = true ]; then
        echo -e "\n${GREEN}All tools are ready to use!${NC}"
    else
        echo -e "\n${YELLOW}Most tools are ready. Install Rust for epcheck.${NC}"
    fi

    echo -e "\n${BLUE}For help with any tool, run: <tool-name> --help${NC}"
}

# Run main function
main "$@"