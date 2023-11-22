use std::fmt::Display;

use crate::{
    card::{card_set::CardSet, Card, Number},
    player::Property,
};

use super::Rules;

#[derive(Clone)]
pub struct CardPile {
    shifted_multiplicity_to_cards: [CardSet; 3],
}

impl Display for CardPile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for cards in self.shifted_multiplicity_to_cards {
            for card in cards.iter() {
                write!(f, "{} ", card)?;
            }
        }

        Ok(())
    }
}

impl CardPile {
    pub fn contains(&self, card: Card) -> bool {
        self.shifted_multiplicity_to_cards[0].contains(card)
    }

    pub fn new() -> Self {
        CardPile {
            shifted_multiplicity_to_cards: [CardSet::none(); 3],
        }
    }

    pub fn unreachable(&self, rules: &Rules) -> CardSet {
        let mut result = CardSet::all();
        for color in rules.used_colors() {
            let one = Card {
                color,
                number: Number::One,
            };
            if self.shifted_multiplicity_to_cards[2].contains(one) {
                continue;
            }
            result.remove(one);

            let two = Card {
                color,
                number: Number::Two,
            };
            if self.shifted_multiplicity_to_cards[1].contains(two) {
                continue;
            }
            result.remove(two);

            let three = Card {
                color,
                number: Number::Three,
            };
            if self.shifted_multiplicity_to_cards[1].contains(three) {
                continue;
            }
            result.remove(three);

            let four = Card {
                color,
                number: Number::Four,
            };
            if self.shifted_multiplicity_to_cards[1].contains(four) {
                continue;
            }
            result.remove(four);

            let five = Card {
                color,
                number: Number::Five,
            };
            if self.shifted_multiplicity_to_cards[0].contains(five) {
                continue;
            }
            result.remove(five);
        }

        result
    }

    pub fn add(&mut self, card: &Card) {
        for cards in &mut self.shifted_multiplicity_to_cards {
            if cards.add(*card) {
                return;
            }
        }

        panic!()
    }

    pub(crate) fn full_sets(&self) -> CardSet {
        let mut result = CardSet::none();

        let mut ones = CardSet::with_property(Property::Number(Number::One));
        let mut twos = CardSet::with_property(Property::Number(Number::Two));
        let mut threes = CardSet::with_property(Property::Number(Number::Three));
        let mut fours = CardSet::with_property(Property::Number(Number::Four));
        let mut fives = CardSet::with_property(Property::Number(Number::Five));

        ones.intersect(&self.shifted_multiplicity_to_cards[2]);
        twos.intersect(&self.shifted_multiplicity_to_cards[1]);
        threes.intersect(&self.shifted_multiplicity_to_cards[1]);
        fours.intersect(&self.shifted_multiplicity_to_cards[1]);
        fives.intersect(&self.shifted_multiplicity_to_cards[0]);

        result.merge(&ones);
        result.merge(&twos);
        result.merge(&threes);
        result.merge(&fours);
        result.merge(&fives);

        result
    }
}

impl Default for CardPile {
    fn default() -> Self {
        Self::new()
    }
}
