#!/bin/bash
# Block file modifications when on protected branches (main/master)

BRANCH=$(git rev-parse --abbrev-ref HEAD 2>/dev/null)

# Only check protected branches
if [[ "$BRANCH" != "main" && "$BRANCH" != "master" ]]; then
  exit 0
fi

# Read stdin to get tool input
INPUT=$(cat)

# Extract file_path from JSON input (works for both Edit and Write tools)
FILE_PATH=$(echo "$INPUT" | grep -o '"file_path"[[:space:]]*:[[:space:]]*"[^"]*"' | head -1 | sed 's/.*"file_path"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/')

# Allow writes outside the project directory (e.g., ~/.claude/plans/)
if [[ -n "$FILE_PATH" ]]; then
  if [[ "$FILE_PATH" != "$CLAUDE_PROJECT_DIR"* ]]; then
    exit 0
  fi
fi

# Block writes to project files on protected branch
cat << 'EOF'
{
  "hookSpecificOutput": {
    "hookEventName": "PreToolUse",
    "permissionDecision": "deny",
    "permissionDecisionReason": "Cannot modify files on protected branch. Create a feature branch first: git checkout -b feature/your-feature-name"
  }
}
EOF
exit 0
