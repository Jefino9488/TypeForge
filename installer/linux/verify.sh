#!/bin/bash
set -e

if ! command -v typeforge &> /dev/null; then
    echo "TypeForge CLI not found in PATH."
    echo "Make sure ~/.local/bin is in your PATH."
    exit 1
fi

typeforge doctor
