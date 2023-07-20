use std::io::Error;
use getrandom;

extern crate rand;

use rand::seq::SliceRandom;
use rand_core::SeedableRng;
use rand_xoshiro::Xoshiro256PlusPlus;

use strum::IntoEnumIterator;

use super::{Card, Value, Suit};

/// A deck of cards.
///
/// This deck will contain 52 distinct cards upon initalization. To ensure uniform randomness,
/// mersenne twisters are used when the deck is intialized and everytime when the muck is
/// reshuffled back in.
///
/// Example
/// ```rust
/// use playing_cards::core::CardDeck;
///
/// let mut deck = CardDeck::new();
/// deck.shuffle(None);
///
/// let hand = deck.deal_cards(2, false);
///
/// println!("{:?}", hand.unwrap()); // Two random cards from the deck
/// ```
pub struct CardDeck {
    deck: Vec<Card>,
    seed: Option<[u8; 32]>,
    muck: Vec<Card>,
}

impl CardDeck {
    /// Creates a new randomized CardDeck.
    ///
    /// The seed of the mersenne twister is set based on the entropy of the system.
    ///
    /// Example
    /// ```rust
    /// use playing_cards::core::CardDeck;
    ///
    /// for _ in 0..10 {
    ///     let mut deck = CardDeck::new();
    ///
    ///     // Since we did not shuffle the deck of cards, we should see cards in descending order. 
    ///     for (i, card) in (52..0).zip(deck) {
    ///         assert_eq!(i, Into::<i32>::into(card));
    ///     }
    /// }
    /// ```
    pub fn new() -> CardDeck {
        Self::create_unshuffled_deck()
    }

    /// Creates a new CardDeck from the given seed.
    ///
    /// This method is a way to create deterministic deck for random but predictiable outcomes.
    /// Please note that this method will attempt to shuffle the deck, but if shuffling fails,
    /// `new_with_seed()` will return an error.
    ///
    /// Examples
    /// ```rust
    /// use playing_cards::core::CardDeck;
    ///
    /// for _ in 0..10 {
    ///     let mut seed_bytes = Vec::from(1337_u32.to_ne_bytes());
    ///     seed_bytes.extend_from_slice(&[0u8; 28]);
    ///     let mut deck = CardDeck::new_with_seed(Some(seed_bytes.as_slice().try_into().unwrap())).unwrap();
    ///
    ///     // Every single line should produce the same 5 cards in the same exact order because
    ///     // we gave each deck the same seed.
    ///     let hand = deck.deal_cards(5, false);
    ///     println!("{:?}", hand.unwrap());
    /// }
    /// ```
    ///
    /// ```rust
    /// use playing_cards::core::CardDeck;
    ///
    /// for i in 0..10 {
    ///     let mut seed_bytes = Vec::from((i as u32).to_ne_bytes());
    ///     seed_bytes.extend_from_slice(&[0u8; 28]);
    ///     let mut deck = CardDeck::new_with_seed(Some(seed_bytes.as_slice().try_into().unwrap())).unwrap();
    ///
    ///     // Each line should be different from one another, but if you rerun this code again,
    ///     // it will print out the exact 10 lines again.
    ///     let hand = deck.deal_cards(5, false);
    ///     println!("{:?}", hand.unwrap());
    /// }
    /// ```
    ///
    /// If you do use `new_with_seed()` and are using it inputting random seeds, the cards within
    /// the deck can be predicted if the seed generation is predictable (e.g. incrementing the seed
    /// by one, using unix time). It is better to use `new()` in these cases since the entropy from
    /// the system cannot be replicated across systems easily unless the seed generated is shared.
    pub fn new_with_seed(seed: Option<[u8; 32]>) -> Result<CardDeck, Error> {
        let mut deck = Self::create_unshuffled_deck();

        if let Some(_) = seed {
            if let Err(err) = deck.shuffle(seed) {
                return Err(err);
            }
        }

        Ok(deck)
    }

    fn create_unshuffled_deck() -> CardDeck {
        let mut d: Vec<Card> = Vec::with_capacity(52);

        for s in Suit::iter() {
            for v in Value::iter() {
                d.push(Card{
                    value: v,
                    suit: s,
                });
            }
        }

        CardDeck{
            deck: d,
            seed: None,
            muck: Vec::new(),
        }
    }

    /// Shuffles the deck.
    ///
    /// An optional seed can be provided if the deck should be shuffled with a specific seed. If no
    /// seed is provided, then system entropy is sampled for a random seed.
    pub fn shuffle(&mut self, seed: Option<[u8; 32]>) -> Result<(), Error> {
        match Self::shuffle_cards(&mut self.deck, seed) {
            Ok(seed) => {
                self.seed = Some(seed);
                Ok(())
            },
            Err(err) => Err(err)
        }
    }

    fn shuffle_cards(cards: &mut Vec<Card>, seed: Option<[u8; 32]>) -> Result<[u8; 32], Error> {
        let mut rng;
        let mut seed_used;
        match seed {
            Some(seed) => {
                seed_used = seed
            },
            None => {
                seed_used = [0u8; 32];
                let res = getrandom::getrandom(&mut seed_used);

                if let Err(e) = res {
                    return Err(From::<getrandom::Error>::from(e));
                }
            },
        }
        rng = Xoshiro256PlusPlus::from_seed(seed_used);
        cards.shuffle(&mut rng);
        Ok(seed_used)
    }

    /// Gets the mersenne twister seed of the CardDeck.
    pub fn get_seed(& self) -> Option<[u8; 32]> {
        self.seed
    }

    /// Adds the inputted cards into the muck.
    ///
    /// This is primarily important if reshuffling the muck can occur.
    pub fn muck_cards(&mut self, mut cards: Vec<Card>) {
        self.muck.append(&mut cards);
    }

    /// Checks to see if there are enough cards in the deck to deal
    ///
    /// Returns true if there are enough cards, false otherwise.
    pub fn check_deal_cards(& self, cards_to_deal: usize, include_muck: bool) -> bool {
        let mut total_cards = self.deck.len();
        if include_muck {
            total_cards = self.muck.len();
        }
        total_cards >= cards_to_deal
    }

    /// Deals `n` cards out from the CardDeck.
    ///
    /// If there is not enough cards remaining in the deck, it will reshuffle the mucked card back
    /// into the deck and redeal them out. If there are no more cards left, this method will return
    /// None. The method also returns 
    ///
    /// Examples
    /// ```rust
    /// use playing_cards::core::{Card, CardDeck};
    ///
    /// let mut player_hands: Vec<Vec<Card>> = Vec::new();
    ///
    /// let mut deck = CardDeck::new();
    /// deck.shuffle(None);
    ///
    /// for i in 0..10 {
    ///     if let Some(hand) = deck.deal_cards(2, false) { // 2 cards per player would require 20 cards
    ///         player_hands.push(hand);
    ///     } else {
    ///         unreachable!("Ran out of cards!");
    ///     }
    /// }
    ///
    /// println!("{:?}", player_hands);
    /// ```
    ///
    /// ```rust should_panic
    ///  use playing_cards::core::{Card, CardDeck};
    ///
    /// let mut player_hands: Vec<Vec<Card>> = Vec::new();
    ///
    /// let mut deck = CardDeck::new();
    /// deck.shuffle(None);
    ///
    /// for i in 0..10 {
    ///     if let Some(hand) = deck.deal_cards(6, false) { // 6 cards per player would require 60 cards, but there's only 52
    ///         player_hands.push(hand);
    ///     } else {
    ///         panic!("Ran out of cards!");
    ///     }
    /// }
    ///
    /// unreachable!();
    /// ```
    pub fn deal_cards(&mut self, cards_to_deal: usize, include_muck: bool) -> Option<Vec<Card>> {
        if !self.check_deal_cards(cards_to_deal, include_muck) {
            return None
        }
        let mut cards_dealt: Vec<Card> = Vec::new();
        for _ in 0..cards_to_deal {
            if let Some(s) = self.next() {
                cards_dealt.push(s);
            }
        }

        Some(cards_dealt)
    }

    /// Draws `n` cards out from the CardDeck.
    ///
    /// The definition of drawing in this case means to discard and replace cards. This function
    /// can take any number of discard cards with the help of `muck_cards()` and then simply
    /// invokes `deal_cards()` to deal `n` cards out of the deck.
    pub fn draw_cards(&mut self, cards_to_deal: usize, discard_cards: Option<Vec<Card>>, include_muck: bool) -> Option<Vec<Card>> {
        if !self.check_deal_cards(cards_to_deal - discard_cards.clone().map_or(0, |v| if include_muck { v.len() } else { 0 } ), include_muck) {
            return None
        }
        if let Some(c) = discard_cards {
            self.muck_cards(c);
        }

        self.deal_cards(cards_to_deal, include_muck)
    }

    /// Reshuffles the muck and inserts those cards into the deck.
    ///
    /// The muck will be placed behind the remaining cards in the deck.
    ///
    /// Similar to `shuffle()` this funtion takes in an optional seed if a specific seed is
    /// desired. If no seed is provided, a seed will be sampled from entropy.
    pub fn reshuffle_muck(&mut self, seed: Option<[u8; 32]>) -> Result<(), Error> {
        if let Err(err) = Self::shuffle_cards(&mut self.muck, seed) {
            return Err(err);
        }

        self.muck.append(&mut self.deck);
        self.deck = self.muck.to_owned();
        self.muck = Vec::new();

        Ok(())
    }
}

impl Iterator for CardDeck {
    type Item = Card;

    fn next(&mut self) -> Option<Self::Item> {
        self.deck.pop()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rayon::prelude::*;
    use std::iter::Iterator;
    use super::super::Value;

    #[test]
    fn test_deck_same_seed() {
        let mut seed_bytes = Vec::from(233_i32.to_le_bytes());
        seed_bytes.extend_from_slice(&[0u8; 28]);
        let mut d1 = CardDeck::new_with_seed(Some(seed_bytes.as_slice().try_into().unwrap())).unwrap();
        let mut d2 = CardDeck::new_with_seed(Some(seed_bytes.as_slice().try_into().unwrap())).unwrap();

        are_decks_equal(&mut d1,&mut d2);
    }

    fn are_decks_equal(d1: &mut CardDeck, d2: &mut CardDeck) {
        assert_eq!(d1.seed, d2.seed);
        let mut both_decks = Iterator::zip(d1, d2);
        for i in 0..52 { // checks all cards
            let both_cards = both_decks.next();

            assert_ne!(both_cards, None);

            if let Some((c1,c2)) = both_cards {
                assert_eq!(c1, c2, "Cards at index {} are not equal ({} != {})", i, c1, c2);
            }
        }

        // then check if there is any extra cards over 52
        assert_eq!(both_decks.next(), None);
    }

    #[test]
    fn test_get_seed() {
        let mut expected_seed = Vec::from(233_i32.to_le_bytes());
        expected_seed.extend_from_slice(&[0u8; 28]);
        let d = CardDeck::new_with_seed(Some(expected_seed.as_slice().try_into().unwrap())).unwrap();

        assert_eq!(Vec::from(d.get_seed().unwrap()), expected_seed);
    }

    // This test relies on random entropy seeding. By the very nature of random numbers and normal
    // curves, there will be a subset of runs that will fail since the actual percentage lands
    // outside if the bounds of the expected percentage (+/- 0.2%).
    #[test]
    #[ignore]
    fn test_monte_carlo_2kings_adjacent() {
        let iters = 150000;

        let count : i32 = (0..iters).into_par_iter().map(|_| {
            let mut deck = CardDeck::new();

            deck.shuffle(None).expect("Problem occured when shuffling the deck");

            if are_2kings_adjacent(&mut deck) {
                1
            } else {
                0
            }
        })
        .sum();

        let expected_prob = 1201.0/5525.0; // 1 - ((49! / (49-4)! * 48!) / 52!)
        let actual_prob = (count as f64) / (iters as f64);
        let epsilon = 0.002; // within a percentage of error of the actual
        assert!((actual_prob - expected_prob).abs() < epsilon, "Probability did not fall within {} of expected probability with {} iterations. Expected: {} (Actual: {})", epsilon, iters, expected_prob, actual_prob);
    }

    fn are_2kings_adjacent(deck: &mut CardDeck) -> bool {
        let mut was_previous_king = false;
        while let Some(c) = deck.next() {
            if c.value == Value::King {
                if was_previous_king {
                    return true;
                }
                was_previous_king = true;
            } else {
                was_previous_king = false;
            }
        }
        
        false
    }
}
