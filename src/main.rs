mod card;

use card::{Card, Rank, Suit};
use std::io;
fn main() {
    let card = Card::new(Rank::Ace, Suit::Hearts);
    println!("{:?}", card);
    println!("Enter a Suit: ");
    let mut suit = String::new();
    io::stdin().read_line(&mut suit).unwrap();
    let suit = suit.trim().to_string();
    println!("Enter a Rank: ");
    let mut rank = String::new();
    io::stdin().read_line(&mut rank).unwrap();
    let rank = rank.trim().to_string();
    let card: Result<Card, String> = Card::try_from((rank, suit));
    match card {
        Ok(card) => println!("{:?}", card),
        Err(e) => println!("Error: {}", e),
    }
}
