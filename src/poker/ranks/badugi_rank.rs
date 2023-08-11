use std::ops::Deref;

use super::{BasicRank, IntoRankStrengthIterator, RankStrengthIterator};

/// A rank of a Badugi hand
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd)]
pub struct BadugiRank(pub BasicRank);

impl Ord for BadugiRank {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.strength.cmp(&other.0.strength)
    }
}

impl Deref for BadugiRank {
    type Target = BasicRank;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl IntoRankStrengthIterator for BadugiRank {
    fn into_strength_iter(self) -> RankStrengthIterator {
        RankStrengthIterator::from(self.0)
    }
}
