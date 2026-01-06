#!/bin/bash

# Script to generate a nicely formatted Slack message for the current PR using GitHub CLI

# Check if gh CLI is installed
if ! command -v gh &> /dev/null; then
    echo "Error: GitHub CLI (gh) is not installed. Please install it first."
    exit 1
fi

# Get PR details in JSON format
PR_JSON=$(gh pr view --json title,url,isDraft,state 2>/dev/null)

# Check if PR exists
if [ $? -ne 0 ]; then
    echo "No pull request found for the current branch."
    exit 1
fi

# Parse JSON using jq (assuming jq is available)
if ! command -v jq &> /dev/null; then
    echo "Error: jq is not installed. Please install it to parse JSON output."
    exit 1
fi

TITLE=$(echo "$PR_JSON" | jq -r '.title')
URL=$(echo "$PR_JSON" | jq -r '.url')
IS_DRAFT=$(echo "$PR_JSON" | jq -r '.isDraft')
STATE=$(echo "$PR_JSON" | jq -r '.state')

# Determine emoji for status
if [ "$IS_DRAFT" = "true" ]; then
    EMOJI=":memo:"
elif [ "$STATE" = "OPEN" ]; then
    EMOJI=":eyes:"
elif [ "$STATE" = "CLOSED" ]; then
    EMOJI=":x:"
elif [ "$STATE" = "MERGED" ]; then
    EMOJI=":white_check_mark:"
else
    EMOJI="$STATE"
fi

# Generate text-formatted message (title with link and emoji)
SLACK_MESSAGE="$EMOJI PR: $TITLE
$URL"

# Output the message and copy to clipboard
echo "$SLACK_MESSAGE"
echo "$SLACK_MESSAGE" | pbcopy
echo "Copied to clipboard!"