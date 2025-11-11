#!/bin/bash

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

# Start tmux session
tmux new-session -d -s dev

# Start backend in left pane
tmux send-keys -t dev:0.0 "cd $BACKEND_DIR && dotnet watch run" C-m

# Split window horizontally and start frontend in right pane
tmux split-window -h -t dev
tmux send-keys -t dev:0.1 "cd $FRONTEND_DIR && npm run dev" C-m

# Attach to the session
echo "Attaching to tmux session. Use Ctrl-b d to detach."
tmux attach -t dev