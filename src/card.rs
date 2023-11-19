use std::fmt::{Debug, Display};

use colored::{ColoredString, Colorize};

use crate::player::Property;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Card {
    pub number: Number,
    pub color: Color,
}

impl Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.color
                .tint_str(&format!("{}{}", self.color, self.number))
        )
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
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
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

impl Color {
    fn tint_str(&self, str: &str) -> ColoredString {
        match self {
            Color::White => str.white(),
            Color::Green => str.green(),
            Color::Yellow => str.yellow(),
            Color::Red => str.red(),
            Color::Blue => str.blue(),
        }
    }
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

pub mod card_set;
