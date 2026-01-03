use std::str::FromStr;
use strum_macros::EnumIter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter)]
pub enum Suit {
    Hearts,
    Diamonds,
    Clubs,
    Spades,
}

impl FromStr for Suit {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "hearts" => Ok(Suit::Hearts),
            "h" => Ok(Suit::Hearts),
            "diamonds" => Ok(Suit::Diamonds),
            "d" => Ok(Suit::Diamonds),
            "clubs" => Ok(Suit::Clubs),
            "c" => Ok(Suit::Clubs),
            "spades" => Ok(Suit::Spades),
            "s" => Ok(Suit::Spades),
            _ => Err(format!("Invalid suit: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, EnumIter)]
pub enum Rank {
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}

impl FromStr for Rank {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ace" => Ok(Rank::Ace),
            "1" => Ok(Rank::Ace),
            "a" => Ok(Rank::Ace),

            "two" => Ok(Rank::Two),
            "2" => Ok(Rank::Two),

            "three" => Ok(Rank::Three),
            "3" => Ok(Rank::Three),

            "four" => Ok(Rank::Four),
            "4" => Ok(Rank::Four),

            "five" => Ok(Rank::Five),
            "5" => Ok(Rank::Five),

            "six" => Ok(Rank::Six),
            "6" => Ok(Rank::Six),

            "seven" => Ok(Rank::Seven),
            "7" => Ok(Rank::Seven),

            "eight" => Ok(Rank::Eight),
            "8" => Ok(Rank::Eight),

            "nine" => Ok(Rank::Nine),
            "9" => Ok(Rank::Nine),

            "ten" => Ok(Rank::Ten),
            "10" => Ok(Rank::Ten),

            "jack" => Ok(Rank::Jack),
            "j" => Ok(Rank::Jack),

            "queen" => Ok(Rank::Queen),
            "q" => Ok(Rank::Queen),

            "king" => Ok(Rank::King),
            "k" => Ok(Rank::King),
            _ => Err(format!("Invalid rank: {}", s)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Card {
    pub rank: Rank,
    pub suit: Suit,
}

impl Card {
    pub fn new(rank: Rank, suit: Suit) -> Self {
        Self { rank, suit }
    }
}

impl TryFrom<(String, String)> for Card {
    type Error = String;

    fn try_from(value: (String, String)) -> Result<Self, Self::Error> {
        let rank: Rank = value.0.parse()?;
        let suit: Suit = value.1.parse()?;
        Ok(Self { rank, suit })
    }
}
