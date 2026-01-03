mod card;
mod deck;

use deck::Deck;

fn main() {
    let mut deck = Deck::new();
    deck.do_shuffle();
    println!("{:?}", deck.draw());
    println!("{:?}", deck.draw());
}
