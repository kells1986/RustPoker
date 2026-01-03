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

    println!("{:?}", hand1.best_hand());
    println!("{:?}", hand2.best_hand());
}
