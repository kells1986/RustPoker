# `src` Agent Guide

This file documents the current source-level architecture so an AI agent can quickly resume work with minimal context.

## Module Map

- `card.rs`
  - Core card primitives: `Suit`, `Rank`, `Card`.
  - Handles rank/suit parsing and rank numeric conversion.
  - Foundation for all other modules.

- `deck.rs`
  - Mutable deck state.
  - Creates a standard 52-card deck and supports `shuffle`, `draw`, `draw_n`.
  - `draw_n` is bounded by remaining cards to prevent panic.

- `hand.rs`
  - Hand representation and evaluation.
  - `evaluate_best` computes best 5-card hand from 5..=7 cards.
  - `EvaluatedHand` implements `Ord` for category + kicker comparison.
  - Includes wheel straight handling (`A-2-3-4-5`) and kicker tie-break semantics.

- `game.rs`
  - Showdown and pot settlement logic.
  - Builds side pots from contribution levels.
  - Determines eligible players per side pot (`!folded`).
  - Splits pots across tied winners and assigns odd chips deterministically by seat order.

- `strategy.rs`
  - Player decision interface (`Strategy` trait).
  - `SimpleAgentStrategy`: basic preflop call/fold heuristic.
  - `HumanConsoleStrategy`: reads actions from stdin (`c`/`f`).

- `table.rs`
  - Single-hand table flow with configurable seats and blinds.
  - Wires deck dealing, blind posting, simple preflop action loop, community board, showdown.
  - Uses `game::settle_showdown` for final payouts.
  - Includes strategy builders for `AgentsOnly` and `HumanVsAgents`.

- `lib.rs`
  - Re-exports all modules for tests and binary use.

- `main.rs`
  - CLI entrypoint.
  - Supports:
    - `simulate`: all agents
    - `play`: one human (seat 0) vs agents
  - Accepts table configuration flags (`--seats`, `--small-blind`, etc.).

## Runtime Workflows

### 1. Agent-only simulation

1. Parse CLI with `simulate`.
2. Build `TableConfig`.
3. Build one strategy per seat via `build_strategies(TableMode::AgentsOnly, seats)`.
4. Run `play_single_hand` for each requested round.
5. Print hand-level and aggregate net results.

### 2. Human vs agents

1. Parse CLI with `play`.
2. Build `TableConfig`.
3. Build strategies with `TableMode::HumanVsAgents` (human is seat 0).
4. During decision points, human enters `c` or `f`.
5. Hand resolves through the same table + showdown path as simulation mode.

## Important Invariants

- `TableConfig::validate` enforces:
  - `seats >= 2`
  - positive blinds and stack
  - `big_blind >= small_blind`
  - `initial_stack <= table_stakes`
- `hand::evaluate_best` expects exactly 5..=7 cards.
- `game::settle_showdown` expects at least one player and at least one eligible player in each pot.
- `table::play_single_hand` expects strategy count to match seat count.

## Where To Extend Next

- Add multi-street betting (`flop/turn/river`) and raise sizes in `table.rs`.
- Introduce persistent table state across hands (button/blind rotation and stack carryover).
- Replace `SimpleAgentStrategy` with richer policy implementations for RL hooks.
