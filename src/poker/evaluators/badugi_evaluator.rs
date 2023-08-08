use std::collections::BinaryHeap;

use itertools::Itertools;

use crate::{core::Card, poker::rank::Rank};

use super::EvaluatorError;

fn choose(n: u64, k: u64) -> u64 {
    if k == 0 {
        return 1
    }
    n * choose(n - 1, k - 1) / k
}

pub fn evaluate_hand(player_hand: &Vec<Card>, board: &Vec<Card>) -> Result<Rank, EvaluatorError> {
    if player_hand.len() > 4 {
        return Err(EvaluatorError::TooManyCards("The player hand had too many cards".to_string(), 4));
    }
    if player_hand.len() < 4 {
        return Err(EvaluatorError::NotEnoughCards("The player hand did not have enough cards".to_string(), 4));
    }
    let mut suit_bits = 0;
    let mut rank_bits = 0;
    for c in player_hand {
        suit_bits |= (c.calculate_bit_pattern() >> 12) & 0xf;
        rank_bits |= (c.calculate_bit_pattern() >> 16) & 0x1fff;
    }
    let mut best_hand_card_count = 0;

    while suit_bits != 0 && rank_bits != 0 {
        suit_bits &= suit_bits - 1;
        rank_bits &= rank_bits - 1;

        best_hand_card_count += 1;
    }
    
    player_hand.iter().combinations(best_hand_card_count)
        .filter(|canidate_hand| {
            println!("{:?}", canidate_hand);
            let mut suit_bits = 0;
            let mut rank_bits = 0;
            for c in canidate_hand {
                suit_bits |= (c.calculate_bit_pattern() >> 12) & 0xf;
                rank_bits |= (c.calculate_bit_pattern() >> 16) & 0x1fff;
            }
            let mut distinct_rank_suit_cards = 0;

            while suit_bits != 0 && rank_bits != 0 {
                suit_bits &= suit_bits - 1;
                rank_bits &= rank_bits - 1;

                distinct_rank_suit_cards += 1;
            }

            distinct_rank_suit_cards == best_hand_card_count
        })
        .map(|canidate_hand| {
            let card_ranks = canidate_hand.iter()
                .map(|&card| {
                    (card.value.clone() as u8 + 1) % 13
                })
                .sorted_by(|a,b | {
                    b.cmp(a)
                })
                .collect::<Vec<_>>();

            println!("{:?}", card_ranks);
            let mut base_strength = 1;
            let card_count = card_ranks.len();

            for i in 1..card_count {
                base_strength += choose(13, i as u64);
            }

            let (_, rank) = card_ranks.iter()
                .enumerate()
                .fold((13, Rank{strength: base_strength as u32, hand_rank: card_count as u16, sub_rank: 0, description: None}), 
                    |(prev_rank_strength, mut acc), (i, rank_strength)| {
                        for s in (rank_strength + 1)..prev_rank_strength {
                            let strength_inc = choose((s - 1) as u64, (card_count - i - 1) as u64);
                            acc.strength += strength_inc as u32;
                            acc.sub_rank += strength_inc as u16;
                        }

                        (*rank_strength, acc)
                    });

            return rank;
            let (_, strength) = card_ranks.iter()
                .enumerate()
                .fold((13, 0), |(prev_rank_strength, mut acc), (i, rank_strength)| {
                    for s in (rank_strength + 1)..prev_rank_strength {
                        acc += choose((s - 1) as u64, (card_count - i - 1) as u64);
                    }

                    (*rank_strength, acc)
                });

            // let mut strength = 0;
            // let mut prev_rank_strength = 13;

            // for (i, rank_strength) in card_ranks.into_iter().enumerate() {
            //     println!("({}, {})", i, rank_strength);
            //     for s in (rank_strength + 1)..prev_rank_strength {
            //         strength += choose((s - 1) as u64, (card_count - i - 1) as u64);
            //     }

            //     println!("card {}: {}", rank_strength, strength);
            //     prev_rank_strength = rank_strength;
            // }
            println!("Stength: {} ({})\n", strength, base_strength);

            Rank {
                strength: (strength + base_strength) as u32,
                hand_rank: 0, // TODO
                sub_rank: 0, // TODO
                description: None,
            }
        })
        .fold(Err(EvaluatorError::UnknownError("No valid rank was generated".to_string())), |acc, rank| {
            if let Ok(acc) = acc {
                if rank > acc {
                    Ok(rank)
                } else {
                    Ok(acc)
                }
            } else {
                Ok(rank)
            }
         })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hand_all_same_suit() {
        let hand = Card::vec_from_str("2h4hThQh").expect("Cards did not parse correctly");
        let rank = evaluate_hand(&hand, &vec![]).expect("Hand did not evaluate correctly");

        assert_eq!(rank.strength, 12);

    }

    #[test]
    fn hand_all_same_rank() {
        let hand = Card::vec_from_str("QhQsQdQc").expect("Cards did not parse correctly");
        let rank = evaluate_hand(&hand, &vec![]).expect("Hand did not evaluate correctly");

        assert_eq!(rank.strength, 2);

    }

    #[test]
    fn card_hand_size_2() {
        let hand = Card::vec_from_str("2h4hTd2d").expect("Cards did not parse correctly");
        let rank = evaluate_hand(&hand, &vec![]).expect("Hand did not evaluate correctly");

        // + 1 since all ranks start at strength of 1
        // +13 to account for all hand combos with only 1 card
        // +63 for Σ nCr(n - 1, 1) for all n ∈ [4, 13)
        // + 1 for Σ nCr(n - 1, 0) for all n ∈ [2, 3)
        assert_eq!(rank.strength, 1 + 13 + 63 + 1);

    }

    #[test]
    fn card_hand_size_3() {
        let hand = Card::vec_from_str("3d7h6s7c").expect("Cards did not parse correctly");
        let rank = evaluate_hand(&hand, &vec![]).expect("Hand did not evaluate correctly");

        // +  1 since all ranks start at strength of 1
        // + 91 to account for all hand combos with only 1 or 2 cards
        // +200 for Σ nCr(n - 1, 2) for all n ∈ [7, 13)
        // +  0 for Σ nCr(n - 1, 1) for all n ∈ [6, 6) but |n| = 0
        // +  2 for Σ nCr(n - 1, 0) for all n ∈ [3, 5)
        assert_eq!(rank.strength, 1 + 91 + 200 + 0 + 2);

    }

    #[test]
    fn badugi_hand() {
        let hand = Card::vec_from_str("As3dKc5h").expect("Cards did not parse correctly");
        let rank = evaluate_hand(&hand, &vec![]).expect("Hand did not evaluate correctly");

        // +  1 since all ranks start at strength of 1
        // +377 to account for all hand combos with only 1-3 cards
        // +  0 for Σ nCr(n - 1, 3) for all n ∈ [13, 13) but |n| = 0
        // +161 for Σ nCr(n - 1, 2) for all n ∈ [5, 12)
        // +  2 for Σ nCr(n - 1, 1) for all n ∈ [3, 4)
        // +  1 for Σ nCr(n - 1, 0) for all n ∈ [1, 2)
        assert_eq!(rank.strength, 1 + 377 + 0 + 161 + 2 + 1);
    }

    #[test]
    fn budugi_hand_10th_best() {
        let hand = Card::vec_from_str("As2d5c6h").expect("Cards did not parse correctly");
        let rank = evaluate_hand(&hand, &vec![]).expect("Hand did not evaluate correctly");

        // +  1 since all ranks start at strength of 1
        // +377 to account for all hand combos with only 1-3 cards
        // +490 for Σ nCr(n - 1, 3) for all n ∈ [6, 13)
        // +  0 for Σ nCr(n - 1, 2) for all n ∈ [5, 5) but |n| = 0
        // +  3 for Σ nCr(n - 1, 1) for all n ∈ [2, 4)
        // +  0 for Σ nCr(n - 1, 0) for all n ∈ [1, 1) but |n| = 0
        assert_eq!(rank.strength, 1 + 377 + 490 + 0 + 3 + 0);
    }

    #[test]
    fn best_badugi_hand() {
        let hand = Card::vec_from_str("As2d3c4h").expect("Cards did not parse correctly");
        let rank = evaluate_hand(&hand, &vec![]).expect("Hand did not evaluate correctly");

        // +  1 since all ranks start at strength of 1
        // +377 to account for all hand combos with only 1-3 cards
        // +495 for Σ nCr(n - 1, 3) for all n ∈ [4, 13)
        // +  0 for Σ nCr(n - 1, 2) for all n ∈ [5, 5) but |n| = 0
        // +  0 for Σ nCr(n - 1, 1) for all n ∈ [2, 4) but |n| = 0
        // +  0 for Σ nCr(n - 1, 0) for all n ∈ [1, 1) but |n| = 0
        assert_eq!(rank.strength, 1 + 377 + 495 + 0 + 0 + 0);
    }
}
