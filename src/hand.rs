use crate::card::{Card, Rank};
use std::cmp::Ordering;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hand {
    hole_cards: Vec<Card>,
    community_cards: Vec<Card>,
}

impl Hand {
    pub fn new() -> Self {
        Self {
            hole_cards: Vec::new(),
            community_cards: Vec::new(),
        }
    }

    pub fn add_hole_card(&mut self, card: Card) {
        assert!(
            self.hole_cards.len() < 2,
            "Texas Hold'em hand can only hold 2 hole cards"
        );
        self.hole_cards.push(card);
    }

    pub fn hole_cards(&self) -> &[Card] {
        &self.hole_cards
    }

    pub fn add_community_card(&mut self, card: Card) {
        assert!(
            self.community_cards.len() < 5,
            "Texas Hold'em hand can only hold 5 community cards"
        );
        self.community_cards.push(card);
    }

    pub fn best_hand(&self) -> EvaluatedHand {
        let mut all = Vec::with_capacity(self.hole_cards.len() + self.community_cards.len());
        all.extend_from_slice(&self.hole_cards);
        all.extend_from_slice(&self.community_cards);
        evaluate_best(&all)
    }
}

impl Default for Hand {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum HandCategory {
    HighCard,
    OnePair,
    TwoPair,
    Trips,
    Straight,
    Flush,
    FullHouse,
    Quads,
    StraightFlush,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvaluatedHand {
    pub category: HandCategory,
    pub ranks: [Rank; 5],
    pub cards: [Card; 5],
}

impl Ord for EvaluatedHand {
    fn cmp(&self, other: &Self) -> Ordering {
        self.category
            .cmp(&other.category)
            .then_with(|| self.ranks.cmp(&other.ranks))
    }
}

impl PartialOrd for EvaluatedHand {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub fn evaluate_best(cards: &[Card]) -> EvaluatedHand {
    assert!(
        cards.len() >= 5 && cards.len() <= 7,
        "Texas Hold'em best-of expects 5..=7 cards"
    );

    let mut best: Option<EvaluatedHand> = None;

    for combo in combinations_5(cards) {
        let eval = evaluate_five(combo);
        best = Some(match best {
            None => eval,
            Some(current_best) => current_best.max(eval),
        });
    }

    best.expect("At least one 5-card combination is guaranteed")
}

fn evaluate_five(mut cards: [Card; 5]) -> EvaluatedHand {
    cards.sort_unstable_by(|a, b| b.rank.cmp(&a.rank));

    let is_flush = cards.iter().all(|c| c.suit == cards[0].suit);
    let ranks = [
        cards[0].rank,
        cards[1].rank,
        cards[2].rank,
        cards[3].rank,
        cards[4].rank,
    ];

    let (is_straight, straight_high) = straight_high_rank(&ranks);

    let mut groups: Vec<(u8, Rank)> = Vec::with_capacity(5);
    for &rank in &ranks {
        if let Some(pos) = groups.iter().position(|&(_, r)| r == rank) {
            groups[pos].0 += 1;
        } else {
            groups.push((1, rank));
        }
    }
    groups.sort_unstable_by(|a, b| b.0.cmp(&a.0).then_with(|| b.1.cmp(&a.1)));

    let (category, key) = if is_straight && is_flush {
        (
            HandCategory::StraightFlush,
            [straight_high, Rank::Two, Rank::Two, Rank::Two, Rank::Two],
        )
    } else if groups[0].0 == 4 {
        (
            HandCategory::Quads,
            [groups[0].1, groups[1].1, Rank::Two, Rank::Two, Rank::Two],
        )
    } else if groups[0].0 == 3 && groups.len() == 2 {
        (
            HandCategory::FullHouse,
            [groups[0].1, groups[1].1, Rank::Two, Rank::Two, Rank::Two],
        )
    } else if is_flush {
        (HandCategory::Flush, ranks)
    } else if is_straight {
        (
            HandCategory::Straight,
            [straight_high, Rank::Two, Rank::Two, Rank::Two, Rank::Two],
        )
    } else if groups[0].0 == 3 {
        let mut kickers: Vec<Rank> = groups[1..].iter().map(|&(_, r)| r).collect();
        kickers.sort_unstable_by(|a, b| b.cmp(a));
        (
            HandCategory::Trips,
            [groups[0].1, kickers[0], kickers[1], Rank::Two, Rank::Two],
        )
    } else if groups[0].0 == 2 && groups[1].0 == 2 {
        let pair_high = groups[0].1.max(groups[1].1);
        let pair_low = groups[0].1.min(groups[1].1);
        (
            HandCategory::TwoPair,
            [pair_high, pair_low, groups[2].1, Rank::Two, Rank::Two],
        )
    } else if groups[0].0 == 2 {
        let mut kickers: Vec<Rank> = groups[1..].iter().map(|&(_, r)| r).collect();
        kickers.sort_unstable_by(|a, b| b.cmp(a));
        (
            HandCategory::OnePair,
            [groups[0].1, kickers[0], kickers[1], kickers[2], Rank::Two],
        )
    } else {
        (HandCategory::HighCard, ranks)
    };

    EvaluatedHand {
        category,
        ranks: key,
        cards,
    }
}

fn straight_high_rank(ranks_desc: &[Rank; 5]) -> (bool, Rank) {
    let mut unique_ranks: Vec<Rank> = Vec::with_capacity(5);
    for &rank in ranks_desc {
        if !unique_ranks.contains(&rank) {
            unique_ranks.push(rank);
        }
    }
    if unique_ranks.len() != 5 {
        return (false, Rank::Two);
    }

    let mut values: Vec<u8> = unique_ranks.iter().map(|&rank| rank.value()).collect();
    values.sort_unstable_by(|a, b| b.cmp(a));

    let first = values[0];
    let is_regular_straight = values
        .iter()
        .enumerate()
        .all(|(idx, &value)| value == first.saturating_sub(idx as u8));
    if is_regular_straight {
        return (
            true,
            Rank::try_from(first).expect("Straight high card must map to a valid rank"),
        );
    }

    if values.as_slice() == [14, 5, 4, 3, 2] {
        return (true, Rank::Five);
    }

    (false, Rank::Two)
}

fn combinations_5(cards: &[Card]) -> impl Iterator<Item = [Card; 5]> + '_ {
    let n = cards.len();
    (0..n).flat_map(move |a| {
        (a + 1..n).flat_map(move |b| {
            (b + 1..n).flat_map(move |c| {
                (c + 1..n).flat_map(move |d| {
                    (d + 1..n).map(move |e| [cards[a], cards[b], cards[c], cards[d], cards[e]])
                })
            })
        })
    })
}

#[cfg(test)]
mod tests {
    use super::{Hand, HandCategory, evaluate_best};
    use crate::card::{Card, Rank, Suit};

    fn card(rank: Rank, suit: Suit) -> Card {
        Card::new(rank, suit)
    }

    #[test]
    fn wheel_straight_uses_five_as_high_card() {
        let cards = [
            card(Rank::Ace, Suit::Spades),
            card(Rank::Five, Suit::Hearts),
            card(Rank::Four, Suit::Clubs),
            card(Rank::Three, Suit::Diamonds),
            card(Rank::Two, Suit::Spades),
        ];
        let evaluated = evaluate_best(&cards);

        assert_eq!(evaluated.category, HandCategory::Straight);
        assert_eq!(evaluated.ranks[0], Rank::Five);
    }

    #[test]
    fn one_pair_tie_breaker_uses_kickers() {
        let stronger = evaluate_best(&[
            card(Rank::Ace, Suit::Spades),
            card(Rank::Ace, Suit::Hearts),
            card(Rank::King, Suit::Clubs),
            card(Rank::Queen, Suit::Diamonds),
            card(Rank::Jack, Suit::Spades),
        ]);
        let weaker = evaluate_best(&[
            card(Rank::Ace, Suit::Clubs),
            card(Rank::Ace, Suit::Diamonds),
            card(Rank::King, Suit::Hearts),
            card(Rank::Queen, Suit::Clubs),
            card(Rank::Ten, Suit::Spades),
        ]);

        assert!(stronger > weaker);
    }

    #[test]
    fn evaluate_best_selects_best_five_from_seven_cards() {
        let evaluated = evaluate_best(&[
            card(Rank::Ace, Suit::Hearts),
            card(Rank::King, Suit::Hearts),
            card(Rank::Queen, Suit::Hearts),
            card(Rank::Jack, Suit::Hearts),
            card(Rank::Two, Suit::Hearts),
            card(Rank::Nine, Suit::Clubs),
            card(Rank::Eight, Suit::Diamonds),
        ]);

        assert_eq!(evaluated.category, HandCategory::Flush);
        assert_eq!(evaluated.ranks[0], Rank::Ace);
    }

    #[test]
    fn board_play_results_in_tie() {
        let board = [
            card(Rank::Ace, Suit::Spades),
            card(Rank::King, Suit::Hearts),
            card(Rank::Queen, Suit::Diamonds),
            card(Rank::Jack, Suit::Clubs),
            card(Rank::Ten, Suit::Spades),
        ];

        let player_one = evaluate_best(&[
            card(Rank::Two, Suit::Hearts),
            card(Rank::Three, Suit::Hearts),
            board[0],
            board[1],
            board[2],
            board[3],
            board[4],
        ]);
        let player_two = evaluate_best(&[
            card(Rank::Four, Suit::Hearts),
            card(Rank::Five, Suit::Hearts),
            board[0],
            board[1],
            board[2],
            board[3],
            board[4],
        ]);

        assert_eq!(player_one, player_two);
    }

    #[test]
    fn hand_limits_hole_and_community_cards() {
        let mut hand = Hand::new();
        hand.add_hole_card(card(Rank::Ace, Suit::Spades));
        hand.add_hole_card(card(Rank::King, Suit::Spades));
        hand.add_community_card(card(Rank::Two, Suit::Spades));
        hand.add_community_card(card(Rank::Three, Suit::Spades));
        hand.add_community_card(card(Rank::Four, Suit::Spades));
        hand.add_community_card(card(Rank::Five, Suit::Spades));
        hand.add_community_card(card(Rank::Six, Suit::Spades));

        assert_eq!(hand.best_hand().category, HandCategory::StraightFlush);
    }
}
