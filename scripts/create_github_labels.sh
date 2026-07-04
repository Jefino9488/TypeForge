#!/bin/bash
# Run this script with the GitHub CLI (gh) installed and authenticated.

echo "Creating GitHub labels for TypeForge..."

labels=(
  "bug,d73a4a,Something isn't working"
  "enhancement,a2eeef,New feature or request"
  "good first issue,7057ff,Good for newcomers"
  "help wanted,008672,Extra attention is needed"
  "performance,d4c5f9,Latency or resource usage"
  "documentation,0075ca,Improvements or additions to documentation"
  "testing,fef2c0,Adding tests or CI improvements"
  "architecture,1d76db,Core engine design or refactoring"
  "fcitx,e99695,Fcitx5 adapter specific"
  "engine,5319e7,Core Rust daemon/engine"
  "learning,c2e0c6,Machine learning / Context / SQLite"
  "unicode,fbca04,Emoji or text parsing"
  "benchmark,c5def5,Benchmarking tests"
  "phase-3,ffffff,Next major UI milestone"
)

for label_info in "${labels[@]}"; do
    IFS=',' read -r name color description <<< "$label_info"
    # Attempt to create the label, ignore if it already exists
    gh label create "$name" --color "$color" --description "$description" --force || true
done

echo "Done."
