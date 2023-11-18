use crate::card::Card;

use super::{PositionSet, Property};

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
