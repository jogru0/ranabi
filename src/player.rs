use std::{fmt::Display, ops::RangeInclusive};

use crate::{
    card::{Card, Color, Number},
    state::Rules,
};

use self::action::Action;

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
            } => write!(f, "Play {} from position {}", card, position),
            Action::Play {
                card: None,
                position,
            } => write!(f, "Play from position {}", position),
            Action::Discard {
                card: Some(card),
                position,
            } => write!(f, "Discard {} from position {}", card, position),
            Action::Discard {
                card: None,
                position,
            } => write!(f, "Discard from position {}", position),
            Action::Hint {
                receiver,
                hinted_property,
                positions,
            } => write!(
                f,
                "Hint {} at {}: {}",
                hinted_property,
                player_name(*receiver),
                positions
            ),
        }
    }
}

fn player_name(player_id: usize) -> &'static str {
    match player_id {
        0 => "Alice",
        1 => "Bob",
        2 => "Cathy",
        3 => "Donald",
        _ => todo!(),
    }
}

pub mod action;

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

#[derive(Clone, Copy, Debug, PartialEq)]
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

        for &b in &self.positions[self.all_possible_positions()] {
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
        self.positions[self.all_possible_positions()]
            .iter()
            .rev()
            .position(|&b| b)
            .map(|backwards_index| self.hand_size - backwards_index)
    }

    fn all_possible_positions(&self) -> RangeInclusive<usize> {
        1..=self.hand_size
    }

    fn inverse(mut self) -> Self {
        let all_possible_positions = self.all_possible_positions();
        for b in &mut self.positions[all_possible_positions] {
            *b = !*b;
        }

        self
    }

    fn focus_position(&self, touched: PositionSet) -> usize {
        if self.is_subset_of(touched) {
            return self.smallest().unwrap();
        }

        let chop = touched.inverse().biggest().unwrap();
        if self.contains(chop) {
            chop
        } else {
            self.without(touched).smallest().unwrap()
        }
    }

    fn is_subset_of(&self, touched: PositionSet) -> bool {
        assert_eq!(self.hand_size, touched.hand_size);
        (self.all_possible_positions()).all(|id| touched.contains(id) || !self.contains(id))
    }

    pub fn contains(&self, id: usize) -> bool {
        self.positions[id]
    }

    fn without(mut self, other: PositionSet) -> Self {
        assert_eq!(self.hand_size, other.hand_size);
        for id in self.all_possible_positions() {
            self.positions[id] &= !other.positions[id];
        }
        self
    }

    pub(crate) fn is_empty(&self) -> bool {
        !self.positions.iter().any(|&b| b)
    }

    fn is_full(&self) -> bool {
        self.inverse().is_empty()
    }
}
