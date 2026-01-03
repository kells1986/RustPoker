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
        self.hole_cards.push(card);
    }

    pub fn hole_cards(&self) -> &[Card] {
        &self.hole_cards
    }

    pub fn add_community_card(&mut self, card: Card) {
        self.community_cards.push(card);
    }
}

impl Hand {
    pub fn best_hand(&self) -> EvaluatedHand {
        let mut all = Vec::with_capacity(self.hole_cards.len() + self.community_cards.len());
        all.extend_from_slice(&self.hole_cards);
        all.extend_from_slice(&self.community_cards);
        evaluate_best(&all)
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
        // Primary: category; Secondary: tie-break ranks
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
            Some(b) => b.max(eval),
        });
    }

    best.unwrap()
}

// ---------- 5-card evaluator (the core) ----------

fn evaluate_five(mut cards: [Card; 5]) -> EvaluatedHand {
    // Sort cards by rank descending (helps with kickers / output cards).
    cards.sort_by(|a, b| b.rank.cmp(&a.rank));

    let is_flush = cards.iter().all(|c| c.suit == cards[0].suit);

    // Collect ranks sorted descending
    let ranks = [
        cards[0].rank,
        cards[1].rank,
        cards[2].rank,
        cards[3].rank,
        cards[4].rank,
    ];

    // Straight detection should use unique ranks.
    let (is_straight, straight_high) = straight_high_rank(&ranks);

    // Build frequency table for ranks (5 cards => tiny; just do a vec and sort)
    // We'll compute groups: Vec<(count, rank)> sorted by (count desc, rank desc)
    let mut groups: Vec<(u8, Rank)> = Vec::with_capacity(5);
    for &r in &ranks {
        if let Some(pos) = groups.iter().position(|&(_, rr)| rr == r) {
            groups[pos].0 += 1;
        } else {
            groups.push((1, r));
        }
    }
    groups.sort_by(|(ca, ra), (cb, rb)| cb.cmp(ca).then_with(|| rb.cmp(ra)));

    // Helper to expand groups into tie-break rank list:
    // e.g. for one-pair: [(2, K), (1, A), (1, 9), (1, 3)] => [K, A, 9, 3, ...]
    let mut tiebreak: Vec<Rank> = Vec::with_capacity(5);
    for (count, r) in &groups {
        for _ in 0..*count {
            tiebreak.push(*r);
        }
    }

    // Now assign category + exact tie-break ordering rules.
    let category;
    let mut key = [Rank::Two; 5];

    if is_straight && is_flush {
        category = HandCategory::StraightFlush;
        key = [straight_high, Rank::Two, Rank::Two, Rank::Two, Rank::Two];
    } else if groups[0].0 == 4 {
        // Quads: [quad_rank, kicker, ...]
        category = HandCategory::Quads;
        let quad = groups[0].1;
        let kicker = groups[1].1;
        key = [quad, kicker, Rank::Two, Rank::Two, Rank::Two];
    } else if groups[0].0 == 3 && groups.len() == 2 {
        // Full house: [trips_rank, pair_rank, ...]
        category = HandCategory::FullHouse;
        key = [groups[0].1, groups[1].1, Rank::Two, Rank::Two, Rank::Two];
    } else if is_flush {
        // Flush: ranks descending
        category = HandCategory::Flush;
        key = ranks;
    } else if is_straight {
        category = HandCategory::Straight;
        key = [straight_high, Rank::Two, Rank::Two, Rank::Two, Rank::Two];
    } else if groups[0].0 == 3 {
        // Trips: [trips, kicker1, kicker2, ...]
        category = HandCategory::Trips;
        // groups sorted by count desc then rank desc => trips first, then kickers
        let trips = groups[0].1;
        let mut kickers: Vec<Rank> = groups[1..].iter().map(|&(_, r)| r).collect();
        kickers.sort_by(|a, b| b.cmp(a));
        key = [trips, kickers[0], kickers[1], Rank::Two, Rank::Two];
    } else if groups[0].0 == 2 && groups[1].0 == 2 {
        // Two pair: [high_pair, low_pair, kicker, ...]
        category = HandCategory::TwoPair;
        let p1 = groups[0].1;
        let p2 = groups[1].1;
        let (hi, lo) = if p1 > p2 { (p1, p2) } else { (p2, p1) };
        let kicker = groups[2].1;
        key = [hi, lo, kicker, Rank::Two, Rank::Two];
    } else if groups[0].0 == 2 {
        // One pair: [pair, kicker1, kicker2, kicker3, ...]
        category = HandCategory::OnePair;
        let pair = groups[0].1;
        let mut kickers: Vec<Rank> = groups[1..].iter().map(|&(_, r)| r).collect();
        kickers.sort_by(|a, b| b.cmp(a));
        key = [pair, kickers[0], kickers[1], kickers[2], Rank::Two];
    } else {
        // High card
        category = HandCategory::HighCard;
        key = ranks;
    }

    // If you care about returning the 5 chosen cards in canonical order (e.g. for straights),
    // you can reorder here. For now we return them rank-desc sorted.
    EvaluatedHand {
        category,
        ranks: key,
        cards,
    }
}

/// Given 5 ranks in descending order (may contain duplicates),
/// detect straight and return (is_straight, high_rank).
///
/// Handles the wheel: A-2-3-4-5 => high_rank = Five.
fn straight_high_rank(ranks_desc: &[Rank; 5]) -> (bool, Rank) {
    // Make unique ranks descending
    let mut uniq: Vec<Rank> = Vec::with_capacity(5);
    for &r in ranks_desc.iter() {
        if !uniq.contains(&r) {
            uniq.push(r);
        }
    }
    if uniq.len() != 5 {
        return (false, Rank::Two);
    }

    // Convert to numeric values (2..14)
    let mut vals: Vec<u8> = uniq.iter().map(|&r| rank_value(r)).collect();
    vals.sort_by(|a, b| b.cmp(a));

    // Normal straight: v0, v0-1, v0-2, v0-3, v0-4
    let v0 = vals[0];
    let is_normal = vals
        .iter()
        .enumerate()
        .all(|(i, &v)| v == v0.saturating_sub(i as u8));

    if is_normal {
        return (true, value_to_rank(v0).unwrap());
    }

    // Wheel: A,5,4,3,2 => values are [14,5,4,3,2]
    let is_wheel = vals == vec![14, 5, 4, 3, 2];
    if is_wheel {
        return (true, Rank::Five);
    }

    (false, Rank::Two)
}

// Map Rank <-> numeric for straight logic and tie-breaks.
// Adapt these to your Rank enum ordering if needed.
fn rank_value(r: Rank) -> u8 {
    r.value()
}

fn value_to_rank(v: u8) -> Option<Rank> {
    Rank::try_from(v).ok()
}

// ---------- 5-card combination generator for up to 7 cards ----------

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
