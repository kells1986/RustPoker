use crate::card::{Card, Rank, Suit};
use rand::seq::SliceRandom;
use strum::IntoEnumIterator;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Deck {
    cards: Vec<Card>,
    shuffled: bool,
}

impl Deck {
    fn populate_deck(&mut self) {
        for suit in Suit::iter() {
            for rank in Rank::iter() {
                self.cards.push(Card::new(rank, suit));
            }
        }
    }

    pub fn new() -> Self {
        let mut deck = Self {
            cards: Vec::new(),
            shuffled: false,
        };
        deck.populate_deck();
        deck
    }

    pub fn do_shuffle(&mut self) {
        self.shuffled = true;
        let mut rng = rand::rng();
        self.cards.shuffle(&mut rng);
    }

    pub fn draw(&mut self) -> Option<Card> {
        self.cards.pop()
    }

    pub fn draw_n(&mut self, n: usize) -> Vec<Card> {
        let draw_count = n.min(self.cards.len());
        (0..draw_count).filter_map(|_| self.draw()).collect()
    }

    pub fn is_shuffled(&self) -> bool {
        self.shuffled
    }

    pub fn len(&self) -> usize {
        self.cards.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }
}

impl Default for Deck {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::Deck;
    use std::collections::HashSet;

    #[test]
    fn new_deck_has_52_unique_cards() {
        let deck = Deck::new();
        assert_eq!(deck.len(), 52);

        let unique_count = deck.cards.iter().collect::<HashSet<_>>().len();
        assert_eq!(unique_count, 52);
    }

    #[test]
    fn draw_n_caps_at_remaining_cards() {
        let mut deck = Deck::new();
        let drawn = deck.draw_n(60);
        assert_eq!(drawn.len(), 52);
        assert!(deck.is_empty());
    }
}
