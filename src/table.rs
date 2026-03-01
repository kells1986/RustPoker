use crate::card::Card;
use crate::deck::Deck;
use crate::game::{PlayerInHand, ShowdownError, ShowdownResult, settle_showdown};
use crate::strategy::{
    Action, DecisionContext, HumanConsoleStrategy, SimpleAgentStrategy, Strategy,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableMode {
    AgentsOnly,
    HumanVsAgents,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TableConfig {
    pub seats: usize,
    pub table_stakes: u32,
    pub small_blind: u32,
    pub big_blind: u32,
    pub initial_stack: u32,
}

impl Default for TableConfig {
    fn default() -> Self {
        Self {
            seats: 6,
            table_stakes: 1_000,
            small_blind: 5,
            big_blind: 10,
            initial_stack: 500,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionRecord {
    pub seat_index: usize,
    pub action: Action,
    pub to_call: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandSummary {
    pub community_cards: [Card; 5],
    pub hole_cards: Vec<[Card; 2]>,
    pub committed: Vec<u32>,
    pub folded: Vec<bool>,
    pub payouts: Vec<u32>,
    pub stacks_after: Vec<u32>,
    pub actions: Vec<ActionRecord>,
    pub showdown: ShowdownResult,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TableError {
    InvalidConfig(&'static str),
    StrategyCountMismatch { expected: usize, actual: usize },
    DeckUnderflow,
    Showdown(ShowdownError),
}

impl From<ShowdownError> for TableError {
    fn from(value: ShowdownError) -> Self {
        Self::Showdown(value)
    }
}

impl TableConfig {
    pub fn validate(self) -> Result<Self, TableError> {
        if self.seats < 2 {
            return Err(TableError::InvalidConfig("seats must be at least 2"));
        }
        if self.small_blind == 0 {
            return Err(TableError::InvalidConfig("small_blind must be positive"));
        }
        if self.big_blind < self.small_blind {
            return Err(TableError::InvalidConfig(
                "big_blind must be >= small_blind",
            ));
        }
        if self.initial_stack == 0 {
            return Err(TableError::InvalidConfig("initial_stack must be positive"));
        }
        if self.initial_stack > self.table_stakes {
            return Err(TableError::InvalidConfig(
                "initial_stack must be <= table_stakes",
            ));
        }
        Ok(self)
    }
}

pub fn build_strategies(mode: TableMode, seats: usize) -> Vec<Box<dyn Strategy>> {
    (0..seats)
        .map(|seat_index| match mode {
            TableMode::AgentsOnly => {
                Box::new(SimpleAgentStrategy::new(format!("agent-{seat_index}")))
                    as Box<dyn Strategy>
            }
            TableMode::HumanVsAgents => {
                if seat_index == 0 {
                    Box::new(HumanConsoleStrategy::new("human")) as Box<dyn Strategy>
                } else {
                    Box::new(SimpleAgentStrategy::new(format!("agent-{seat_index}")))
                        as Box<dyn Strategy>
                }
            }
        })
        .collect()
}

pub fn play_single_hand(
    config: TableConfig,
    strategies: &mut [Box<dyn Strategy>],
) -> Result<HandSummary, TableError> {
    let config = config.validate()?;
    if strategies.len() != config.seats {
        return Err(TableError::StrategyCountMismatch {
            expected: config.seats,
            actual: strategies.len(),
        });
    }

    let mut deck = Deck::new();
    deck.do_shuffle();

    let mut hole_cards = Vec::with_capacity(config.seats);
    for _ in 0..config.seats {
        let left = deck.draw().ok_or(TableError::DeckUnderflow)?;
        let right = deck.draw().ok_or(TableError::DeckUnderflow)?;
        hole_cards.push([left, right]);
    }

    let mut stacks = vec![config.initial_stack; config.seats];
    let mut committed = vec![0_u32; config.seats];
    let mut folded = vec![false; config.seats];
    let mut actions = Vec::new();

    post_blind(0, config.small_blind, &mut stacks, &mut committed);
    post_blind(
        1 % config.seats,
        config.big_blind,
        &mut stacks,
        &mut committed,
    );

    let first_to_act = if config.seats == 2 {
        0
    } else {
        2 % config.seats
    };
    for turn in 0..config.seats {
        if active_player_count(&folded) <= 1 {
            break;
        }

        let seat = (first_to_act + turn) % config.seats;
        if folded[seat] || stacks[seat] == 0 {
            continue;
        }

        let highest_commitment = committed.iter().copied().max().unwrap_or(0);
        let to_call = highest_commitment.saturating_sub(committed[seat]);
        let action = strategies[seat].decide(&DecisionContext {
            seat_index: seat,
            hole_cards: hole_cards[seat],
            stack: stacks[seat],
            committed: committed[seat],
            to_call,
            small_blind: config.small_blind,
            big_blind: config.big_blind,
        });
        actions.push(ActionRecord {
            seat_index: seat,
            action,
            to_call,
        });

        match action {
            Action::Fold => folded[seat] = true,
            Action::Call => {
                let paid = to_call.min(stacks[seat]);
                stacks[seat] -= paid;
                committed[seat] += paid;
            }
        }
    }

    let community_cards = draw_community_cards(&mut deck)?;
    let showdown_players = (0..config.seats)
        .map(|seat| PlayerInHand {
            committed: committed[seat],
            folded: folded[seat],
            hole_cards: hole_cards[seat],
        })
        .collect::<Vec<_>>();

    let showdown = settle_showdown(&showdown_players, community_cards)?;
    for (seat, payout) in showdown.payouts.iter().enumerate() {
        stacks[seat] += payout;
    }

    Ok(HandSummary {
        community_cards,
        hole_cards,
        committed,
        folded,
        payouts: showdown.payouts.clone(),
        stacks_after: stacks,
        actions,
        showdown,
    })
}

fn post_blind(seat: usize, amount: u32, stacks: &mut [u32], committed: &mut [u32]) {
    let paid = amount.min(stacks[seat]);
    stacks[seat] -= paid;
    committed[seat] += paid;
}

fn draw_community_cards(deck: &mut Deck) -> Result<[Card; 5], TableError> {
    let community_cards = deck.draw_n(5);
    community_cards
        .try_into()
        .map_err(|_| TableError::DeckUnderflow)
}

fn active_player_count(folded: &[bool]) -> usize {
    folded.iter().filter(|&&is_folded| !is_folded).count()
}

#[cfg(test)]
mod tests {
    use super::{TableConfig, TableError, TableMode, build_strategies, play_single_hand};
    use crate::strategy::{Action, DecisionContext, Strategy};

    #[derive(Debug)]
    struct AlwaysCallStrategy;

    impl Strategy for AlwaysCallStrategy {
        fn name(&self) -> &str {
            "always-call"
        }

        fn decide(&mut self, _context: &DecisionContext) -> Action {
            Action::Call
        }
    }

    #[test]
    fn agents_only_strategy_builder_matches_seat_count() {
        let strategies = build_strategies(TableMode::AgentsOnly, 4);
        assert_eq!(strategies.len(), 4);
    }

    #[test]
    fn table_config_validation_enforces_bounds() {
        let bad = TableConfig {
            seats: 1,
            ..TableConfig::default()
        };
        assert_eq!(
            bad.validate(),
            Err(TableError::InvalidConfig("seats must be at least 2"))
        );
    }

    #[test]
    fn single_hand_runs_with_scripted_strategies() {
        let mut strategies: Vec<Box<dyn Strategy>> = vec![
            Box::new(AlwaysCallStrategy),
            Box::new(AlwaysCallStrategy),
            Box::new(AlwaysCallStrategy),
        ];
        let result = play_single_hand(
            TableConfig {
                seats: 3,
                table_stakes: 1_000,
                small_blind: 5,
                big_blind: 10,
                initial_stack: 100,
            },
            &mut strategies,
        )
        .expect("single hand should resolve");

        assert_eq!(result.hole_cards.len(), 3);
        assert_eq!(result.community_cards.len(), 5);
        assert_eq!(result.committed.len(), 3);
        assert_eq!(
            result.payouts.iter().sum::<u32>(),
            result.committed.iter().sum()
        );
    }
}
