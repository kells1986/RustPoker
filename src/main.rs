mod card;
mod deck;
mod hand;
use deck::Deck;
use hand::Hand;

fn main() {
    let mut deck = Deck::new();
    deck.do_shuffle();

    let mut hand1 = Hand::new();
    hand1.add_hole_card(deck.draw().unwrap());
    hand1.add_hole_card(deck.draw().unwrap());

    let mut hand2 = Hand::new();
    hand2.add_hole_card(deck.draw().unwrap());
    hand2.add_hole_card(deck.draw().unwrap());

    let flop = deck.draw_n(3);
    hand1.add_community_card(flop[0]);
    hand1.add_community_card(flop[1]);
    hand1.add_community_card(flop[2]);

    hand2.add_community_card(flop[0]);
    hand2.add_community_card(flop[1]);
    hand2.add_community_card(flop[2]);

    let turn = deck.draw().unwrap();
    hand1.add_community_card(turn);
    hand2.add_community_card(turn);

    let river = deck.draw().unwrap();
    hand1.add_community_card(river);
    hand2.add_community_card(river);
    println!("Hand 1: {:?}", hand1.hole_cards());
    println!("--");
    println!("Hand 2: {:?}", hand2.hole_cards());
    println!("--");
    println!("Community cards: {:?}", flop);
    println!("--");
    println!("Turn: {:?}", turn);
    println!("--");
    println!("River: {:?}", river);
    println!("--");
    println!("Hand 1 best hand: {:?}", hand1.best_hand());
    println!("--");
    println!("Hand 2 best hand: {:?}", hand2.best_hand());
    println!("--");

    if hand1.best_hand() > hand2.best_hand() {
        println!("Hand 1 wins");
    } else if hand1.best_hand() < hand2.best_hand() {
        println!("Hand 2 wins");
    } else {
        println!("It's a tie");
    }
}
