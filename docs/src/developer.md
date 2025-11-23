# Developer Guide

Welcome to SnapCrab development!
We appreciate your interest in our project.
This guide will help you get started contributing to the project.


Whether you're fixing a bug, adding a feature, or improving documentation,
this guide covers the development practices and tools that will help you contribute effectively.

Before making code changes, we recommend creating an issue if one doesn't exist yet,
and commenting that you would like to work on it.
This helps avoid conflicting contributions and allows for discussion about the approach.

## Git Usage

For simplicity, we try to keep the history of main linear. Ideally, try to keep
each commit small, and make sure it at least compiles.

### Commit Message Format

We try to keep our messages coherent with widely adopted conventions:

- **Title**: Maximum 50 characters
- **Body**: Maximum 80 characters per line
- **Types**: The title should include the type of change introduced by the
  commit.
  Follow [conventional commit](https://www.conventionalcommits.org/en/v1.0.0/) format.

### Git Hooks

We recommend Git hooks to enforce code quality and commit message standards.

#### Pre-commit Hook

The pre-commit hook runs code formatting and linting checks:

```bash
#!/bin/bash

echo "Running pre-commit checks..."

# Run cargo check
if ! cargo check --quiet; then
    echo "Error: Code does not compile. Fix compilation errors before committing."
    exit 1
fi

# Run cargo fmt check
if ! cargo fmt --check; then
    echo "Error: Code is not formatted. Run 'cargo fmt' to fix."
    exit 1
fi

# Run cargo clippy
if ! cargo clippy -- -D warnings; then
    echo "Error: Clippy found issues. Fix them before committing."
    exit 1
fi

echo "Pre-commit checks passed!"
exit 0
```

Save this as `.git/hooks/pre-commit` and make it executable:

```bash
chmod +x .git/hooks/pre-commit
```

#### Commit Message Hook

The commit message hook enforces conventional commit format and character limits:

```bash
#!/bin/bash

commit_file="$1"
commit_msg=$(cat "$commit_file")

# Extract title (first line) and body (rest)
title=$(echo "$commit_msg" | head -n1)
body=$(echo "$commit_msg" | tail -n +3)

error_found=false

# Check title length
if [ ${#title} -gt 50 ]; then
    echo "Error: Commit title exceeds 50 characters (${#title})"
    error_found=true
fi

# Check conventional commit format
if ! echo "$title" | grep -qE '^(feat|fix|docs|style|refactor|test|chore|perf|ci|build|revert)(\(.+\))?: .+'; then
    echo "Error: Title must follow conventional commit format"
    echo "Format: type(scope): description"
    echo "Types: feat, fix, docs, style, refactor, test, chore, perf, ci, build, revert"
    error_found=true
fi

# Check body line lengths
while IFS= read -r line; do
    if [ ${#line} -gt 80 ]; then
        echo "Error: Body line exceeds 80 characters (${#line})"
        error_found=true
    fi
done <<< "$body"

if [ "$error_found" = true ]; then
    echo ""
    echo "Rejected commit message:"
    echo "========================"
    echo "$commit_msg"
    exit 1
fi

exit 0
```

Save this as `.git/hooks/commit-msg` and make it executable:

```bash
chmod +x .git/hooks/commit-msg
```
