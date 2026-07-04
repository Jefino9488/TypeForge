# Quick Start

Welcome to TypeForge! This guide will help you get started with predictive typing in minutes.

## 1. Verify Installation
First, let's make sure TypeForge is fully integrated with your system:
```bash
typeforge doctor
```
If you see all green checkmarks, you're good to go.

## 2. Using TypeForge
There are no special shortcuts or hotkeys to learn. Just start typing in any application—your IDE, your terminal, or your web browser.

When you type a few characters (like `funct`), a small Fcitx5 popup will appear with predictions (e.g., `function`).
- **To accept a prediction**: Press the `Number Key` corresponding to the candidate in the list (e.g., `1`), or press `Space` if it's the first candidate.
- **To ignore a prediction**: Just keep typing! TypeForge will silently get out of your way.

## 3. Training TypeForge
TypeForge uses a **Positive Reinforcement Model**. Out of the box, it knows standard dictionary words. But it really shines when it learns *your* vocabulary.

Every time you type a word manually that isn't in the base dictionary (like a specific project name `MyCoolProject`), TypeForge learns it in the background. The next time you type `MyC`, it will suggest `MyCoolProject`.

## 4. Diagnostics
If you ever feel like the predictions are slow, or if the popup stops appearing, use the CLI:

```bash
# Get current status
typeforge info

# See daemon logs for crashes
cat ~/.local/state/typeforge/logs/daemon.log
```
