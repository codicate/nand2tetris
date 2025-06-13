#!/bin/bash

# === CONFIG: List of directories ===
DIRS=(
  "project1"
  "project2"
  "project3"
)

# === CONFIG: Diff command to use (e.g. diff, colordiff, meld) ===
DIFF_CMD="diff"

# === Run cargo and diff on each directory ===
for dir in "${DIRS[@]}"; do
  echo "🔧 Running in $dir..."

  # Run cargo (assumes it generates *.tokens.xml)
  cargo run -- "$dir" || { echo "❌ cargo run failed in $dir"; cd - > /dev/null || exit; continue; }

  # Navigate into directory
  cd "$dir" || { echo "❌ Failed to enter directory $dir"; continue; }

  # Loop through all *.tokens.xml files
  for tokens_file in *.tokens.xml; do
    # Derive the base name (e.g., Main.tokens.xml → Main)
    base="${tokens_file%.tokens.xml}"
    target_file="${base}T.xml"

    # Check if the T.xml file exists
    if [[ -f "$target_file" ]]; then
      echo "🔍 Comparing $tokens_file <-> $target_file"
      $DIFF_CMD "$target_file" "$tokens_file"
    else
      echo "⚠️  Missing file: $target_file"
    fi
  done

  cd ".."

  # Go back to the original directory
  cd - > /dev/null || exit
done
