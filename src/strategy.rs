use crate::card::{Card, Rank};
use std::cmp::{max, min};
use std::io::{self, Write};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Fold,
    Call,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecisionContext {
    pub seat_index: usize,
    pub hole_cards: [Card; 2],
    pub stack: u32,
    pub committed: u32,
    pub to_call: u32,
    pub small_blind: u32,
    pub big_blind: u32,
}

pub trait Strategy {
    fn name(&self) -> &str;
    fn decide(&mut self, context: &DecisionContext) -> Action;
}

#[derive(Debug, Default, Clone)]
pub struct SimpleAgentStrategy {
    name: String,
}

impl SimpleAgentStrategy {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

impl Strategy for SimpleAgentStrategy {
    fn name(&self) -> &str {
        &self.name
    }

    fn decide(&mut self, context: &DecisionContext) -> Action {
        if context.to_call == 0 {
            return Action::Call;
        }

        let [left, right] = context.hole_cards;
        let is_pair = left.rank == right.rank;
        let is_suited = left.suit == right.suit;
        let high_rank = max(left.rank, right.rank);
        let low_rank = min(left.rank, right.rank);
        let gap = high_rank.value() as i32 - low_rank.value() as i32;

        let has_premium = is_pair
            || matches!(high_rank, Rank::Ace | Rank::King)
            || (high_rank.value() >= Rank::Queen.value() && is_suited);
        let has_playability = is_suited && gap <= 2 && high_rank.value() >= Rank::Ten.value();
        let cheap_call = context.to_call <= context.big_blind;

        if has_premium || (has_playability && cheap_call) || (cheap_call && high_rank == Rank::Ace)
        {
            Action::Call
        } else {
            Action::Fold
        }
    }
}

#[derive(Debug, Clone)]
pub struct HumanConsoleStrategy {
    name: String,
}

impl HumanConsoleStrategy {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

impl Strategy for HumanConsoleStrategy {
    fn name(&self) -> &str {
        &self.name
    }

    fn decide(&mut self, context: &DecisionContext) -> Action {
        loop {
            println!(
                "Seat {} ({}) hole: {:?} {:?}, stack: {}, committed: {}, to_call: {}",
                context.seat_index,
                self.name,
                context.hole_cards[0],
                context.hole_cards[1],
                context.stack,
                context.committed,
                context.to_call
            );
            print!("Choose action [c=call/check, f=fold]: ");
            let _ = io::stdout().flush();

            let mut input = String::new();
            match io::stdin().read_line(&mut input) {
                Ok(_) => match input.trim().to_lowercase().as_str() {
                    "c" | "call" | "check" => return Action::Call,
                    "f" | "fold" => return Action::Fold,
                    _ => println!("Invalid action, please enter c or f."),
                },
                Err(_) => return Action::Fold,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Action, DecisionContext, SimpleAgentStrategy, Strategy};
    use crate::card::{Card, Rank, Suit};

    fn c(rank: Rank, suit: Suit) -> Card {
        Card::new(rank, suit)
    }

    #[test]
    fn simple_agent_calls_with_pair() {
        let mut agent = SimpleAgentStrategy::new("agent");
        let action = agent.decide(&DecisionContext {
            seat_index: 0,
            hole_cards: [c(Rank::Ten, Suit::Hearts), c(Rank::Ten, Suit::Spades)],
            stack: 100,
            committed: 0,
            to_call: 10,
            small_blind: 5,
            big_blind: 10,
        });

        assert_eq!(action, Action::Call);
    }

    #[test]
    fn simple_agent_folds_trash_for_large_call() {
        let mut agent = SimpleAgentStrategy::new("agent");
        let action = agent.decide(&DecisionContext {
            seat_index: 1,
            hole_cards: [c(Rank::Seven, Suit::Hearts), c(Rank::Two, Suit::Spades)],
            stack: 100,
            committed: 0,
            to_call: 25,
            small_blind: 5,
            big_blind: 10,
        });

        assert_eq!(action, Action::Fold);
    }
}
