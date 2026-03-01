use crate::card::Card;
use crate::hand::{EvaluatedHand, evaluate_best};
use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerInHand {
    pub committed: u32,
    pub folded: bool,
    pub hole_cards: [Card; 2],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SidePotResult {
    pub amount: u32,
    pub contributors: Vec<usize>,
    pub eligible: Vec<usize>,
    pub winners: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShowdownResult {
    pub payouts: Vec<u32>,
    pub side_pots: Vec<SidePotResult>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShowdownError {
    NoPlayers,
    NoEligiblePlayersForPot { pot_index: usize },
}

pub fn settle_showdown(
    players: &[PlayerInHand],
    community_cards: [Card; 5],
) -> Result<ShowdownResult, ShowdownError> {
    if players.is_empty() {
        return Err(ShowdownError::NoPlayers);
    }

    let evaluated_hands = evaluate_all_hands(players, community_cards);
    let mut payouts = vec![0_u32; players.len()];
    let mut side_pots = Vec::new();

    let mut levels: Vec<u32> = players
        .iter()
        .map(|player| player.committed)
        .filter(|&committed| committed > 0)
        .collect();
    levels.sort_unstable();
    levels.dedup();

    let mut previous_level = 0_u32;
    for level in levels {
        let contributors: Vec<usize> = players
            .iter()
            .enumerate()
            .filter_map(|(index, player)| (player.committed >= level).then_some(index))
            .collect();

        let amount = (level - previous_level) * contributors.len() as u32;
        previous_level = level;
        if amount == 0 {
            continue;
        }

        let eligible: Vec<usize> = contributors
            .iter()
            .copied()
            .filter(|&index| !players[index].folded)
            .collect();

        if eligible.is_empty() {
            return Err(ShowdownError::NoEligiblePlayersForPot {
                pot_index: side_pots.len(),
            });
        }

        let winners = winner_indexes(&eligible, &evaluated_hands);
        payout_side_pot(amount, &winners, &mut payouts);

        side_pots.push(SidePotResult {
            amount,
            contributors,
            eligible,
            winners,
        });
    }

    Ok(ShowdownResult { payouts, side_pots })
}

fn evaluate_all_hands(
    players: &[PlayerInHand],
    community_cards: [Card; 5],
) -> Vec<Option<EvaluatedHand>> {
    players
        .iter()
        .map(|player| {
            if player.folded {
                return None;
            }

            let cards = [
                player.hole_cards[0],
                player.hole_cards[1],
                community_cards[0],
                community_cards[1],
                community_cards[2],
                community_cards[3],
                community_cards[4],
            ];
            Some(evaluate_best(&cards))
        })
        .collect()
}

fn winner_indexes(eligible: &[usize], evaluated_hands: &[Option<EvaluatedHand>]) -> Vec<usize> {
    let mut best = eligible[0];
    for &index in eligible.iter().skip(1) {
        if evaluated_hands[index]
            .as_ref()
            .expect("Eligible player must have a hand value")
            > evaluated_hands[best]
                .as_ref()
                .expect("Eligible player must have a hand value")
        {
            best = index;
        }
    }

    let best_hand = evaluated_hands[best]
        .as_ref()
        .expect("Best player must have a hand value");

    eligible
        .iter()
        .copied()
        .filter(|&index| {
            evaluated_hands[index]
                .as_ref()
                .expect("Eligible player must have a hand value")
                .cmp(best_hand)
                == Ordering::Equal
        })
        .collect()
}

fn payout_side_pot(amount: u32, winners: &[usize], payouts: &mut [u32]) {
    let winner_count = winners.len() as u32;
    let base_share = amount / winner_count;
    let odd_chips = amount % winner_count;

    for &winner in winners {
        payouts[winner] += base_share;
    }

    // Deterministic odd-chip assignment: earliest seat(s) in winner order.
    for &winner in winners.iter().take(odd_chips as usize) {
        payouts[winner] += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::{PlayerInHand, ShowdownError, settle_showdown};
    use crate::card::{Card, Rank, Suit};

    fn c(rank: Rank, suit: Suit) -> Card {
        Card::new(rank, suit)
    }

    fn player(committed: u32, folded: bool, hole_cards: [Card; 2]) -> PlayerInHand {
        PlayerInHand {
            committed,
            folded,
            hole_cards,
        }
    }

    #[test]
    fn heads_up_board_play_splits_pot_evenly() {
        let community = [
            c(Rank::Ace, Suit::Spades),
            c(Rank::King, Suit::Hearts),
            c(Rank::Queen, Suit::Diamonds),
            c(Rank::Jack, Suit::Clubs),
            c(Rank::Ten, Suit::Spades),
        ];
        let players = [
            player(
                100,
                false,
                [c(Rank::Two, Suit::Clubs), c(Rank::Three, Suit::Clubs)],
            ),
            player(
                100,
                false,
                [c(Rank::Four, Suit::Hearts), c(Rank::Five, Suit::Hearts)],
            ),
        ];

        let result = settle_showdown(&players, community).expect("showdown should resolve");
        assert_eq!(result.payouts, vec![100, 100]);
        assert_eq!(result.side_pots.len(), 1);
        assert_eq!(result.side_pots[0].winners, vec![0, 1]);
    }

    #[test]
    fn all_in_side_pots_are_distributed_to_eligible_winners() {
        let community = [
            c(Rank::Two, Suit::Hearts),
            c(Rank::Seven, Suit::Diamonds),
            c(Rank::Nine, Suit::Clubs),
            c(Rank::Jack, Suit::Spades),
            c(Rank::Queen, Suit::Hearts),
        ];
        let players = [
            player(
                100,
                false,
                [c(Rank::Ace, Suit::Spades), c(Rank::Ace, Suit::Diamonds)],
            ),
            player(
                50,
                false,
                [c(Rank::King, Suit::Spades), c(Rank::King, Suit::Diamonds)],
            ),
            player(
                20,
                false,
                [c(Rank::Queen, Suit::Clubs), c(Rank::Queen, Suit::Diamonds)],
            ),
        ];

        let result = settle_showdown(&players, community).expect("showdown should resolve");
        assert_eq!(result.payouts, vec![110, 0, 60]);
        assert_eq!(result.side_pots.len(), 3);
        assert_eq!(result.side_pots[0].amount, 60);
        assert_eq!(result.side_pots[0].winners, vec![2]);
        assert_eq!(result.side_pots[1].amount, 60);
        assert_eq!(result.side_pots[1].winners, vec![0]);
        assert_eq!(result.side_pots[2].amount, 50);
        assert_eq!(result.side_pots[2].winners, vec![0]);
    }

    #[test]
    fn folded_dead_money_stays_in_pot_for_remaining_players() {
        let community = [
            c(Rank::Two, Suit::Hearts),
            c(Rank::Three, Suit::Diamonds),
            c(Rank::Nine, Suit::Clubs),
            c(Rank::Jack, Suit::Spades),
            c(Rank::Queen, Suit::Hearts),
        ];
        let players = [
            player(
                100,
                false,
                [c(Rank::Ace, Suit::Spades), c(Rank::Ace, Suit::Diamonds)],
            ),
            player(
                100,
                false,
                [c(Rank::King, Suit::Spades), c(Rank::King, Suit::Diamonds)],
            ),
            player(
                100,
                true,
                [c(Rank::Queen, Suit::Spades), c(Rank::Queen, Suit::Diamonds)],
            ),
        ];

        let result = settle_showdown(&players, community).expect("showdown should resolve");
        assert_eq!(result.payouts, vec![300, 0, 0]);
    }

    #[test]
    fn odd_chip_goes_to_earliest_winner_seat() {
        let community = [
            c(Rank::Ace, Suit::Hearts),
            c(Rank::King, Suit::Diamonds),
            c(Rank::Nine, Suit::Spades),
            c(Rank::Four, Suit::Clubs),
            c(Rank::Two, Suit::Diamonds),
        ];
        let players = [
            player(
                5,
                false,
                [c(Rank::Queen, Suit::Hearts), c(Rank::Queen, Suit::Diamonds)],
            ),
            player(
                5,
                false,
                [c(Rank::Queen, Suit::Clubs), c(Rank::Queen, Suit::Spades)],
            ),
            player(
                5,
                false,
                [c(Rank::Jack, Suit::Hearts), c(Rank::Jack, Suit::Clubs)],
            ),
        ];

        let result = settle_showdown(&players, community).expect("showdown should resolve");
        assert_eq!(result.payouts, vec![8, 7, 0]);
    }

    #[test]
    fn showdown_requires_players() {
        let community = [
            c(Rank::Ace, Suit::Hearts),
            c(Rank::King, Suit::Diamonds),
            c(Rank::Queen, Suit::Spades),
            c(Rank::Jack, Suit::Clubs),
            c(Rank::Ten, Suit::Hearts),
        ];

        let result = settle_showdown(&[], community);
        assert_eq!(result, Err(ShowdownError::NoPlayers));
    }
}
