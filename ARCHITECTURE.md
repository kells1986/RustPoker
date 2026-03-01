## Overview

The project is structured as a Rust library (`src/lib.rs`) plus a CLI executable entrypoint (`src/main.rs`).

- `lib.rs` exports core modules: `card`, `deck`, `hand`, `game`, `strategy`, `table`.
- `main.rs` exposes two flows:
  - `simulate` (all seats use agents),
  - `play` (seat 0 is human, remaining seats are agents).

This keeps the game engine testable and reusable for future RL or simulation layers.

## Core Modules

### `card.rs`
- Defines foundational card types:
  - `Suit`
  - `Rank` (including `value()` and string parsing)
  - `Card`
- Parsing and conversion logic lives here to keep rank/suit normalization centralized.

### `deck.rs`
- Defines `Deck` as mutable card state.
- Supports:
  - deck population (52 cards)
  - shuffling
  - single and multi-card draw
- `draw_n` is bounded by remaining cards to avoid panics when over-drawing.

### `hand.rs`
- Contains hand-evaluation primitives:
  - `Hand` (hole + community storage)
  - `evaluate_best` (best 5 from 5..=7 cards)
  - `EvaluatedHand` with strict `Ord` semantics
- Tie-breaking is rank-key driven and includes wheel straight support (`A-2-3-4-5`).

### `game.rs`
- Contains multiplayer showdown settlement:
  - `PlayerInHand` tracks commitment, fold state, and hole cards.
  - `settle_showdown` resolves payouts for heads-up and multiway pots.
  - `ShowdownResult` captures per-seat payouts and side-pot breakdowns.
- Pot logic:
  1. Build side pots from unique commitment levels.
  2. Determine eligible players for each pot (`!folded` and committed at that level).
  3. Compare eligible hands by poker strength (`Ord` on `EvaluatedHand`).
  4. Split pot among winners, with deterministic odd-chip assignment to earliest winner seats.

### `strategy.rs`
- Defines the `Strategy` trait used by table seats.
- Includes:
  - `SimpleAgentStrategy` (heuristic preflop call/fold),
  - `HumanConsoleStrategy` (stdin-driven call/fold input).

### `table.rs`
- Defines `TableConfig` and hand execution flow.
- Responsibilities:
  - validate game parameters (seats, blinds, stacks),
  - deal hole/community cards,
  - post blinds,
  - run a simple preflop action round (call/fold, no raises yet),
  - call `game::settle_showdown` and return payout summary.

## Showdown Behavior Guarantees

The current tests lock in behavior for:

- heads-up board-play ties (pot splitting),
- multiway all-in side-pot payout correctness,
- folded dead-money inclusion in contested pots,
- odd-chip split determinism,
- kicker-based tie-break ordering.

## Forward Path

The existing structure is intentionally compatible with future expansion into:

- richer betting round state machines,
- player action histories and legal-action generation,
- strategy modules (rule-based or RL-driven),
- simulation harnesses that run many hands from controlled initial states.

The recommended next step is introducing an explicit `Table/GameState` type that composes `deck`, player stacks, betting rounds, and `settle_showdown`.
