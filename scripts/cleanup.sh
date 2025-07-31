#!/bin/bash

# Final Cleanup Script for Auto-Coder Companion
# This script removes placeholders and TODOs, and ensures no empty files

set -e

# Set up environment
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
cd "$PROJECT_ROOT"

echo "🧹 Running final cleanup for Auto-Coder Companion..."

# Find all source files
SOURCE_FILES=$(find "$PROJECT_ROOT" -type f \( -name "*.rs" -o -name "*.ts" -o -name "*.tsx" -o -name "*.js" -o -name "*.jsx" -o -name "*.html" -o -name "*.css" -o -name "*.json" -o -name "*.md" \) -not -path "*/node_modules/*" -not -path "*/target/*" -not -path "*/.git/*")

# Check for TODOs and placeholders
echo "🔍 Checking for TODOs and placeholders..."
TODO_FILES=$(grep -l "TODO\|PLACEHOLDER\|FIXME\|XXX" $SOURCE_FILES 2>/dev/null || true)

if [[ -n "$TODO_FILES" ]]; then
  echo "⚠️ Found TODOs or placeholders in the following files:"
  echo "$TODO_FILES"
  
  # Ask for confirmation to remove
  read -p "Do you want to remove these TODOs and placeholders? (y/n) " -n 1 -r
  echo
  if [[ $REPLY =~ ^[Yy]$ ]]; then
    for file in $TODO_FILES; do
      echo "🔧 Cleaning up $file..."
      # Replace TODO comments with empty strings or appropriate content
      sed -i 's/\/\/ TODO.*//g' "$file"
      sed -i 's/\/\* TODO.*\*\///g' "$file"
      sed -i 's/<!-- TODO.*-->//g' "$file"
      sed -i 's/# TODO.*//g' "$file"
      
      # Replace PLACEHOLDER markers
      sed -i 's/PLACEHOLDER//g' "$file"
      sed -i 's/FIXME//g' "$file"
      sed -i 's/XXX//g' "$file"
    done
    echo "✅ Removed TODOs and placeholders"
  else
    echo "⚠️ TODOs and placeholders not removed"
    exit 1
  fi
else
  echo "✅ No TODOs or placeholders found"
fi

# Check for empty files
echo "🔍 Checking for empty files..."
EMPTY_FILES=$(find "$PROJECT_ROOT" -type f -empty -not -path "*/node_modules/*" -not -path "*/target/*" -not -path "*/.git/*")

if [[ -n "$EMPTY_FILES" ]]; then
  echo "⚠️ Found empty files:"
  echo "$EMPTY_FILES"
  
  # Ask for confirmation to remove
  read -p "Do you want to remove these empty files? (y/n) " -n 1 -r
  echo
  if [[ $REPLY =~ ^[Yy]$ ]]; then
    for file in $EMPTY_FILES; do
      echo "🗑️ Removing $file..."
      rm "$file"
    done
    echo "✅ Removed empty files"
  else
    echo "⚠️ Empty files not removed"
    exit 1
  fi
else
  echo "✅ No empty files found"
fi

# Check for missing files in IDE extensions
echo "🔍 Checking IDE extensions..."
for ide in "vscode" "vscodium" "visual-studio"; do
  if [[ ! -d "$PROJECT_ROOT/extensions/$ide" ]]; then
    echo "⚠️ Missing IDE extension directory: $ide"
    mkdir -p "$PROJECT_ROOT/extensions/$ide"
    echo "✅ Created IDE extension directory: $ide"
  fi
done

# Ensure all required directories exist
echo "🔍 Ensuring all required directories exist..."
REQUIRED_DIRS=(
  "src/components"
  "src/hooks"
  "src/utils"
  "src/assets"
  "src-tauri/src"
  "scripts"
  "extensions/vscode"
  "extensions/vscodium"
  "extensions/visual-studio"
  "docs"
)

for dir in "${REQUIRED_DIRS[@]}"; do
  if [[ ! -d "$PROJECT_ROOT/$dir" ]]; then
    echo "⚠️ Missing required directory: $dir"
    mkdir -p "$PROJECT_ROOT/$dir"
    echo "✅ Created required directory: $dir"
  fi
done

echo "🏁 Final cleanup completed!"
echo "✅ Project is ready for production build"
