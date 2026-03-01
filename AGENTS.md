# Rust Poker - Agent Guide

This repository contains a Texas Hold'em poker implementation in Rust, serving as a foundation for reinforcement learning experiments.

## ⚡️ Verification Commands

Use these commands to verify changes:

- **Run Tests**: `cargo test` (Runs all unit tests)
- **Run Game Logic Tests Only**: `cargo test game::tests`
- **Run Hand Evaluation Tests Only**: `cargo test hand::tests`
- **Check Formatting**: `cargo fmt -- --check`
- **Linting**: `cargo clippy -- -D warnings`
- **Run Simulation**: `cargo run` (Executes the main game loop in `src/main.rs`)

## ⚠️ Codex Gotchas

Recurring issues or specific implementation details to be aware of:

- **Hand Evaluation**: `evaluate_best` iterates over all 5-card combinations. Ensure this remains efficient if card count increases.
- **Ace Ranking**: Logic exists to handle Ace as low in A-2-3-4-5 straights (Wheel). Verify this behavior if modifying rank logic.
- **Tie-Breaking**: `EvaluatedHand` uses `Ord` for comparison. Ties are broken by `ranks` array (kickers). Always verify kicker sorting in `evaluate_five`.
- **Deck State**: `Deck` operations modify state. Be careful when sharing deck instances across simulations.
- **Showdown Settlement**: Side pots are built from unique contribution levels and paid to eligible (not folded) players only.
- **Odd Chips**: Split-pot remainders are paid deterministically to earliest winner seat indexes.

## 📂 Task-Specific Documentation

Recommended files for specific workflows:

- [TODO.md](TODO.md) (Suggested): Track upcoming features like RL integration and betting logic.
- [ARCHITECTURE.md](ARCHITECTURE.md) (Suggested): Document the interaction between the game engine and future RL agents.
