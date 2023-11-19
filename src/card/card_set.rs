use std::fmt::Debug;

use bitvec::{array::BitArray, BitArr};

use crate::player::Property;

use super::{Card, Color, Number};

const NUMBER_OF_COLORS: usize = 5;
const NUMBER_OF_NUMBERS: usize = 5;
const NUMBER_OF_CARDS: usize = NUMBER_OF_NUMBERS * NUMBER_OF_COLORS;

#[derive(Clone, Copy)]
pub struct CardSet {
    bitset: BitArr!(for NUMBER_OF_CARDS, in u32),
}

impl Debug for CardSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for card in self.iter() {
            write!(f, "{card}")?;
        }
        Ok(())
    }
}

fn card_to_index(card: Card) -> usize {
    let color_index = match card.color {
        Color::White => 0,
        Color::Green => 1,
        Color::Yellow => 2,
        Color::Red => 3,
        Color::Blue => 4,
    };

    let number_index = match card.number {
        Number::One => 0,
        Number::Two => 1,
        Number::Three => 2,
        Number::Four => 3,
        Number::Five => 4,
    };

    let index = NUMBER_OF_NUMBERS * color_index + number_index;
    assert!(index < NUMBER_OF_CARDS);
    index
}

fn index_to_card(index: usize) -> Card {
    assert!(index < NUMBER_OF_CARDS);
    let color = match index / NUMBER_OF_NUMBERS {
        0 => Color::White,
        1 => Color::Green,
        2 => Color::Yellow,
        3 => Color::Red,
        4 => Color::Blue,
        _ => unreachable!(),
    };

    let number = match index % NUMBER_OF_NUMBERS {
        0 => Number::One,
        1 => Number::Two,
        2 => Number::Three,
        3 => Number::Four,
        4 => Number::Five,
        _ => unreachable!(),
    };

    Card { number, color }
}

#[allow(clippy::unusual_byte_groupings)]
impl CardSet {
    pub fn none() -> Self {
        Self {
            bitset: BitArray::ZERO,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.bitset.not_any()
    }

    pub fn all() -> Self {
        Self {
            bitset: BitArray::new([0b00_00000_11111_11111_11111_11111_11111]),
        }
    }

    pub fn len(&self) -> usize {
        self.bitset.count_ones()
    }

    pub fn apply(&mut self, hinted_property: Property) {
        self.intersect(&Self::with_property(hinted_property));
    }

    pub(crate) fn extend(&mut self, other: Self) {
        self.bitset |= other.bitset;
    }

    pub(crate) fn add(&mut self, card: Card) -> bool {
        !self.bitset.replace(card_to_index(card), true)
    }

    pub(crate) fn with_property(hinted_property: Property) -> Self {
        let data = match hinted_property {
            Property::Color(Color::White) => [0b00_00000_00000_00000_00000_00000_11111],
            Property::Color(Color::Green) => [0b00_00000_00000_00000_00000_11111_00000],
            Property::Color(Color::Yellow) => [0b00_00000_00000_00000_11111_00000_00000],
            Property::Color(Color::Red) => [0b00_00000_00000_11111_00000_00000_00000],
            Property::Color(Color::Blue) => [0b00_00000_11111_00000_00000_00000_00000],
            Property::Number(Number::One) => [0b00_00000_00001_00001_00001_00001_00001],
            Property::Number(Number::Two) => [0b00_00000_00010_00010_00010_00010_00010],
            Property::Number(Number::Three) => [0b00_00000_00100_00100_00100_00100_00100],
            Property::Number(Number::Four) => [0b00_00000_01000_01000_01000_01000_01000],
            Property::Number(Number::Five) => [0b00_00000_10000_10000_10000_10000_10000],
        };
        Self {
            bitset: BitArray::new(data),
        }
    }

    pub(crate) fn intersect(&mut self, other: &CardSet) {
        self.bitset &= other.bitset;
    }

    pub(crate) fn exclude(&mut self, other: &CardSet) {
        self.bitset &= !other.bitset;
    }

    pub fn contains(&self, card: &Card) -> bool {
        self.bitset[card_to_index(*card)]
    }

    pub(crate) fn apply_not(&mut self, hinted_property: Property) {
        self.exclude(&Self::with_property(hinted_property));
    }

    pub(crate) fn remove(&mut self, card: &Card) -> bool {
        self.bitset.replace(card_to_index(*card), false)
    }

    pub(crate) fn merge(&mut self, other: &CardSet) {
        self.bitset |= other.bitset
    }

    pub(crate) fn intersects(&self, other: &CardSet) -> bool {
        (self.bitset & other.bitset).any()
    }

    pub(crate) fn in_play_order(&self) -> Vec<Card> {
        self.iter().collect()
    }

    pub(crate) fn unique(&self) -> Option<Card> {
        if self.bitset.count_ones() == 1 {
            Some(index_to_card(self.bitset.first_one().unwrap()))
        } else {
            None
        }
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = Card> + '_ {
        self.bitset.iter_ones().map(index_to_card)
    }
}

#[cfg(test)]
mod tests {
    use crate::{player::Property, state::Rules};

    use super::CardSet;

    #[test]
    fn properties_work_correctly() {
        for property in Property::all(&Rules::new()) {
            let pos = CardSet::with_property(property);
            let mut neg = CardSet::all();
            neg.apply_not(property);

            for card in pos.iter() {
                assert!(card.satisfies(property));
            }
            for card in neg.iter() {
                assert!(!card.satisfies(property));
            }

            assert_eq!(pos.len(), 5);
            assert_eq!(neg.len(), 20);
            assert!(!pos.intersects(&neg));
        }
    }
}
