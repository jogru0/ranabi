use std::{fmt::Debug, iter::Copied};

use indexmap::{set::Iter, IndexSet};

use crate::{player::Property, state::Rules};

use super::{Card, Number};

#[derive(Clone)]
pub struct CardSet {
    hashed: IndexSet<Card>,
}

impl Debug for CardSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for card in &self.hashed {
            write!(f, "{card}")?;
        }
        Ok(())
    }
}

impl CardSet {
    pub fn none() -> Self {
        Self {
            hashed: IndexSet::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.hashed.is_empty()
    }

    pub fn all(rules: &Rules) -> Self {
        let mut hashed = IndexSet::with_capacity(25);
        for color in rules.used_colors() {
            hashed.insert(Card {
                number: Number::One,
                color,
            });
            hashed.insert(Card {
                number: Number::Two,
                color,
            });
            hashed.insert(Card {
                number: Number::Three,
                color,
            });
            hashed.insert(Card {
                number: Number::Four,
                color,
            });
            hashed.insert(Card {
                number: Number::Five,
                color,
            });
        }
        Self { hashed }
    }

    pub fn apply(&mut self, hinted_property: Property) {
        self.hashed.retain(|card| card.satisfies(hinted_property));
    }

    pub(crate) fn empty() -> Self {
        Self {
            hashed: IndexSet::new(),
        }
    }

    pub(crate) fn extend(&mut self, currently_playable: Self) {
        self.hashed.extend(currently_playable.hashed)
    }

    pub(crate) fn add(&mut self, card: Card) -> bool {
        self.hashed.insert(card)
    }

    pub(crate) fn with_property(rules: &Rules, hinted_property: Property) -> Self {
        let mut result = Self::all(rules);
        result.apply(hinted_property);
        result
    }

    pub(crate) fn intersect(&mut self, possibilities: &CardSet) {
        self.hashed.retain(|card| possibilities.contains(card));
    }

    pub(crate) fn exclude(&mut self, possibilities: &CardSet) {
        self.hashed.retain(|card| !possibilities.contains(card));
    }

    pub fn contains(&self, card: &Card) -> bool {
        self.hashed.contains(card)
    }

    pub(crate) fn apply_not(&mut self, hinted_property: Property) {
        self.hashed.retain(|card| !card.satisfies(hinted_property));
    }

    pub(crate) fn remove(&mut self, card: &Card) -> bool {
        self.hashed.remove(card)
    }

    pub(crate) fn merge(&mut self, possibilities: &CardSet) {
        self.hashed.extend(&possibilities.hashed)
    }

    pub(crate) fn intersects(&self, other: &CardSet) -> bool {
        self.hashed.iter().any(|card| other.contains(card))
    }

    pub(crate) fn retain(&mut self, keep: impl FnMut(&Card) -> bool) {
        self.hashed.retain(keep);
    }

    pub(crate) fn in_play_order(&self) -> Vec<Card> {
        let mut result: Vec<Card> = self.hashed.iter().copied().collect();

        result.sort_unstable_by_key(|card| card.number);

        result
    }

    pub(crate) fn unique(&self) -> Option<Card> {
        if self.hashed.len() == 1 {
            Some(self.hashed[0])
        } else {
            None
        }
    }

    pub(crate) fn iter(&self) -> Copied<Iter<'_, Card>> {
        self.hashed.iter().copied()
    }
}

impl<'a> IntoIterator for &'a CardSet {
    type Item = Card;

    type IntoIter = Copied<Iter<'a, Card>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
