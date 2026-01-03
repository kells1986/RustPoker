use crate::card::{Card, Rank, Suit};
use rand::rngs::ThreadRng;
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
        self.cards.shuffle(&mut ThreadRng::default());
    }

    pub fn draw(&mut self) -> Option<Card> {
        self.cards.pop()
    }

    pub fn draw_n(&mut self, n: u8) -> Vec<Card> {
        self.cards.drain(0..n as usize).collect()
    }
}
