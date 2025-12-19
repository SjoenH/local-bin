#!/bin/bash

# start — starts tmux dev session with backend left and frontend right
# Testing checklist:
#  - Default (no REUSE_SESSION): ensures previous 'dev' session is killed and a fresh session is created with exactly two panes.
#  - With existing 'dev' session and REUSE_SESSION=true: creates a new window in that session and uses two panes there.
# Usage:
#  REUSE_SESSION=true ./start
#
echo "Finding backend directory..."
BACKEND_DIR=$(find . -name "appsettings.json" -type f -not -path "*/node_modules/*" -not -path "*/bin/*" -not -path "*/obj/*" -printf "%h\n" | head -1)

if [ -z "$BACKEND_DIR" ]; then
    echo "Error: Could not find backend directory with appsettings.json"
    exit 1
fi

echo "Backend directory: $BACKEND_DIR"

echo "Finding frontend directory..."
FRONTEND_DIR=$(find . -name "package.json" -type f -not -path "*/node_modules/*" -printf "%h\n" | head -1)

if [ -z "$FRONTEND_DIR" ]; then
    echo "Error: Could not find frontend directory with package.json"
    exit 1
fi

echo "Frontend directory: $FRONTEND_DIR"

echo "Starting tmux session..."

# By default we recreate the `dev` session for deterministic layout.
# Set REUSE_SESSION=true to create a new window in an existing session instead.
if [ "${REUSE_SESSION:-false}" = "true" ]; then
    echo "Reusing existing 'dev' session (creating a new window)..."
    if tmux has-session -t dev 2>/dev/null; then
        WIN_NAME="devwin-$(date +%s)"
        tmux new-window -t dev -n "$WIN_NAME"
    else
        WIN_NAME="devwin"
        tmux new-session -d -s dev -n "$WIN_NAME"
    fi
else
    echo "Recreating 'dev' session..."
    # ignore error if session does not exist
    tmux kill-session -t dev 2>/dev/null || true
    WIN_NAME="devwin"
    tmux new-session -d -s dev -n "$WIN_NAME"
fi

# Start backend in left pane
# Target the specific window/pane to avoid splitting unexpected windows.
# Pane 0 will be the left pane, pane 1 will be created by split-window below.

tmux send-keys -t dev:$WIN_NAME.0 "cd $BACKEND_DIR && dotnet watch run" C-m

# Split window horizontally and start frontend in right pane
tmux split-window -h -t dev:$WIN_NAME.0
tmux send-keys -t dev:$WIN_NAME.1 "cd $FRONTEND_DIR && npm run dev" C-m

# Ensure left pane is selected, then attach
tmux select-pane -t dev:$WIN_NAME.0
tmux select-window -t dev:$WIN_NAME

echo "Attaching to tmux session. Use Ctrl-b d to detach."
tmux attach -t dev