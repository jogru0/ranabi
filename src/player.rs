use std::fmt::Display;

use crate::{
    card::{Card, Color, Number},
    state::Rules,
};

pub trait Player {
    fn witness_action(&mut self, action: Action, player: usize);
    fn witness_draw(&mut self, player: usize, card: Option<Card>);
    fn request_action(&self) -> Action;
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::Play {
                card: Some(card),
                position,
            } => write!(f, "play {} from position {}", card, position),
            Action::Play {
                card: None,
                position,
            } => write!(f, "play from position {}", position),
            Action::Discard {
                card: Some(card),
                position,
            } => write!(f, "discard {} from position {}", card, position),
            Action::Discard {
                card: None,
                position,
            } => write!(f, "discard from position {}", position),
            Action::Hint {
                receiver,
                hinted_property,
                positions,
            } => write!(f, "Hint {} at {}: {}", hinted_property, receiver, positions),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Action {
    Play {
        card: Option<Card>,
        position: usize,
    },
    Discard {
        card: Option<Card>,
        position: usize,
    },
    Hint {
        receiver: usize,
        hinted_property: Property,
        positions: PositionSet,
    },
}
impl Action {
    pub(crate) fn add_card_information(&mut self, old: Card) {
        match self {
            Action::Play { card, .. } => {
                assert!(card.is_none());
                *card = Some(old);
            }
            Action::Discard { card, .. } => {
                assert!(card.is_none());
                *card = Some(old);
            }
            Action::Hint { .. } => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Property {
    Color(Color),
    Number(Number),
}

impl Display for Property {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Property::Color(color) => write!(f, "{color}"),
            Property::Number(number) => write!(f, "{number}"),
        }
    }
}

impl Property {
    fn all(rules: &Rules) -> Vec<Self> {
        let mut result = Vec::with_capacity(5 + rules.used_colors().len());
        result.push(Property::Number(Number::One));
        result.push(Property::Number(Number::Two));
        result.push(Property::Number(Number::Three));
        result.push(Property::Number(Number::Four));
        result.push(Property::Number(Number::Five));
        for color in rules.used_colors() {
            result.push(Property::Color(color));
        }
        result
    }
}

pub mod basic;

#[derive(Clone, Copy, Debug)]
pub struct PositionSet {
    positions: [bool; 6],
    hand_size: usize,
}

impl Display for PositionSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn to_char(b: bool) -> char {
            if b {
                'X'
            } else {
                '-'
            }
        }

        for &b in &self.positions[1..=self.hand_size] {
            write!(f, "{}", to_char(b))?;
        }
        Ok(())
    }
}

impl PositionSet {
    fn smallest(&self) -> Option<usize> {
        self.positions.iter().position(|&b| b)
    }

    fn biggest(&self) -> Option<usize> {
        self.positions.iter().rev().position(|&b| b)
    }

    fn inverse(mut self, hand_size: usize) -> Self {
        for b in &mut self.positions[1..=hand_size] {
            *b = !*b;
        }

        self
    }

    fn focus_position(&self, touched: PositionSet, hand_size: usize) -> usize {
        if self.is_subset_of(touched, hand_size) {
            return self.smallest().unwrap();
        }

        let chop = touched.inverse(hand_size).biggest().unwrap();
        if self.contains(chop) {
            chop
        } else {
            self.without(touched, hand_size).smallest().unwrap()
        }
    }

    fn is_subset_of(&self, touched: PositionSet, hand_size: usize) -> bool {
        (1..=hand_size).all(|id| touched.contains(id) || !self.contains(id))
    }

    pub fn contains(&self, id: usize) -> bool {
        self.positions[id]
    }

    fn without(mut self, touched: PositionSet, hand_size: usize) -> Self {
        for id in 1..=hand_size {
            self.positions[id] &= !touched.positions[id];
        }
        self
    }

    pub(crate) fn is_empty(&self) -> bool {
        !self.positions.iter().any(|&b| b)
    }
}
