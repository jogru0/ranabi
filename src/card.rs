use std::fmt::{Debug, Display};

use indexmap::IndexSet;

use crate::{player::Property, state::Rules};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Card {
    pub number: Number,
    pub color: Color,
}

impl Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.color, self.number)
    }
}

impl Card {
    pub(crate) fn satisfies(&self, hinted_property: Property) -> bool {
        match hinted_property {
            Property::Color(c) => self.color == c,
            Property::Number(n) => self.number == n,
        }
    }

    pub(crate) fn next(color: Color, number: Option<Number>) -> Option<Self> {
        match number {
            Some(Number::One) => Some(Card {
                number: Number::Two,
                color,
            }),
            Some(Number::Two) => Some(Card {
                number: Number::Three,
                color,
            }),
            Some(Number::Three) => Some(Card {
                number: Number::Four,
                color,
            }),
            Some(Number::Four) => Some(Card {
                number: Number::Five,
                color,
            }),
            Some(Number::Five) => None,
            None => Some(Card {
                number: Number::One,
                color,
            }),
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Number {
    One,
    Two,
    Three,
    Four,
    Five,
}
impl Number {
    pub(crate) fn score(&self) -> usize {
        match self {
            Number::One => 1,
            Number::Two => 2,
            Number::Three => 3,
            Number::Four => 4,
            Number::Five => 5,
        }
    }

    pub(crate) fn comes_after(&self, current: Option<Number>) -> bool {
        current
            == match self {
                Number::One => None,
                Number::Two => Some(Number::One),
                Number::Three => Some(Number::Two),
                Number::Four => Some(Number::Three),
                Number::Five => Some(Number::Four),
            }
    }

    pub(crate) fn decrease(&self) -> Option<Self> {
        match self {
            Number::One => None,
            Number::Two => Some(Number::One),
            Number::Three => Some(Number::Two),
            Number::Four => Some(Number::Three),
            Number::Five => Some(Number::Four),
        }
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub enum Color {
    White,
    Green,
    Yellow,
    Red,
    Blue,
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let char = match self {
            Color::White => 'w',
            Color::Green => 'g',
            Color::Yellow => 'y',
            Color::Red => 'r',
            Color::Blue => 'b',
        };

        write!(f, "{char}")
    }
}

impl Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let char = match self {
            Number::One => '1',
            Number::Two => '2',
            Number::Three => '3',
            Number::Four => '4',
            Number::Five => '5',
        };

        write!(f, "{char}")
    }
}

#[derive(Clone)]
pub struct PossibleCards {
    pub hashed: IndexSet<Card>,
}

impl Debug for PossibleCards {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for card in &self.hashed {
            write!(f, "{card}")?;
        }
        Ok(())
    }
}

impl PossibleCards {
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

    pub(crate) fn add(&mut self, card: Card) {
        self.hashed.insert(card);
    }

    pub(crate) fn with_property(rules: &Rules, hinted_property: Property) -> Self {
        let mut result = Self::all(rules);
        result.apply(hinted_property);
        result
    }

    pub(crate) fn intersect(&mut self, possibilities: &PossibleCards) {
        self.hashed.retain(|card| possibilities.contains(card));
    }

    pub(crate) fn exclude(&mut self, possibilities: &PossibleCards) {
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

    pub(crate) fn merge(&mut self, possibilities: &PossibleCards) {
        self.hashed.extend(&possibilities.hashed)
    }
}
