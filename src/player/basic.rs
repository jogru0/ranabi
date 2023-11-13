use std::mem::swap;

use indexmap::{IndexMap, IndexSet};

use crate::{
    card::{Card, Number, PossibleCards},
    state::{PublicState, Rules},
};

use super::{Action, Player, PositionSet, Property};

pub struct BasicPlayer {
    player_states: Vec<PlayerState>,
    witnessed_cards: Vec<Option<Card>>,
    player_id: usize,
    public_state: PublicState,
}

impl BasicPlayer {
    pub fn new(rules: Rules, player_id: usize) -> Self {
        let player_states = (0..rules.number_of_players)
            .map(|_| PlayerState::new())
            .collect();

        BasicPlayer {
            player_states,
            witnessed_cards: Vec::new(),
            player_id,
            public_state: PublicState::new(rules),
        }
    }

    fn play_or_discard_card(&mut self, seen_card: Card, player: usize, position: usize) {
        let card_id = self.player_states[player].cards.get(position);
        let info = &mut self.witnessed_cards[card_id];

        if player == self.player_id {
            assert!(info.is_none());
            *info = Some(seen_card);
        } else {
            assert_eq!(*info, Some(seen_card));
        }

        self.player_states[player].play_or_discard_card(position)
    }

    fn get_positions(&self, hinted_property: Property, receiver: usize) -> PositionSet {
        assert_ne!(receiver, self.player_id);

        let card_ids = self.player_states[receiver].cards.cards;

        let mut positions = [false; 6];

        for pos in 0..6 {
            let Some(card_id) = card_ids[pos] else {continue;};
            let card = self.witnessed_cards[card_id].unwrap();
            if card.satisfies(hinted_property) {
                positions[pos] = true;
            }
        }

        PositionSet { positions }
    }

    fn suggest_hint(&self) -> Option<(usize, Property, PositionSet)> {
        for receiver in 0..self.public_state.rules.number_of_players {
            if receiver == self.player_id {
                continue;
            }

            for property in Property::all() {
                let positions = self.get_positions(hinted_property, receiver);
                if positions.is_empty() {
                    continue;
                }

                let interpretations = self.player_states[receiver].get_hint_interpretations(
                    hinted_property,
                    positions,
                    &self.public_state,
                );
                if interpretations.are_the_truth(&self.witnessed_cards) {
                    return Some((receiver, property, positions));
                }
            }
        }

        None
    }
}

impl Player for BasicPlayer {
    fn witness_action(&mut self, action: Action, player: usize) {
        match action {
            Action::Play {
                card: Some(card),
                position,
            } => {
                self.play_or_discard_card(card, player, position);
                self.public_state.play(card);
            }
            Action::Discard {
                card: Some(card),
                position,
            } => {
                self.play_or_discard_card(card, player, position);
                self.public_state.discard();
            }
            Action::Hint {
                receiver,
                hinted_property,
                positions,
            } => {
                self.player_states[receiver].fr_apply_hint(
                    hinted_property,
                    positions,
                    &self.public_state,
                );
                self.public_state.hint();
            }
            _ => unreachable!(),
        }
    }

    fn witness_draw(&mut self, player: usize, card: Option<Card>) {
        let id = self.witnessed_cards.len();
        self.witnessed_cards.push(card);
        self.player_states[player].add_card(id, &self.public_state.rules)
    }

    fn request_action(&self) -> Action {
        if let Some(index) = self.player_states[self.player_id].suggest_play(&self.public_state) {
            return Action::Play {
                card: None,
                position: index,
            };
        }

        if self.public_state.clues != 0 {
            if let Some((receiver, hinted_property, positions)) = self.suggest_hint() {
                return Action::Hint {
                    receiver,
                    hinted_property,
                    positions,
                };
            }
        }

        if let Some(chop) = self.player_states[self.player_id].chop_position() {
            if self.public_state.clues != self.public_state.rules.max_clues {
                return Action::Discard {
                    card: None,
                    position: chop,
                };
            }
        }

        todo!()
    }
}

struct PlayerState {
    cards: HandCards,
    possible_cards: IndexMap<usize, PossibleCards>,
    touched: IndexSet<usize>,
    interpretations: Vec<Interpretations>,
}

impl PlayerState {
    fn new() -> Self {
        Self {
            cards: HandCards::new(),
            possible_cards: IndexMap::new(),
            touched: IndexSet::new(),
            interpretations: Vec::new(),
        }
    }

    fn add_card(&mut self, id: usize, rules: &Rules) {
        self.cards.add_card(id);
        let previous = self.possible_cards.insert(id, PossibleCards::all(rules));
        assert!(previous.is_none());
    }

    fn play_or_discard_card(&mut self, position: usize) {
        let id = self.cards.play_card(position);
        self.possible_cards.remove(&id);
    }

    fn chop_position(&self) -> Option<usize> {
        for pos in self.cards.current_hand_size..1 {
            let id = self.cards.cards[pos].unwrap();
            if !self.touched.contains(&id) {
                return Some(id);
            }
        }
        None
    }

    fn get_hint_interpretations(
        &self,
        hinted_property: Property,
        positions: PositionSet,
        state: &PublicState,
    ) -> Interpretations {
        let focus = positions.focus(self.touched(), self.cards.current_hand_size);

        let card_id = self.cards.cards[focus].unwrap();

        let is_chop_focused = self.chop_position() == Some(focus);

        let mut direct_interpretation_focus_possibilities = PossibleCards::empty();

        let currently_playable = state.firework.currently_playable();
        direct_interpretation_focus_possibilities.extend(currently_playable);

        if is_chop_focused {
            let critical_saves = state.critical_saves();
            direct_interpretation_focus_possibilities.extend(critical_saves);

            if hinted_property == Property::Number(Number::Two)
                || hinted_property == Property::Number(Number::Five)
            {
                let special_saves = PossibleCards::with_property(&state.rules, hinted_property);
                direct_interpretation_focus_possibilities.extend(special_saves)
            }
        }

        let direct_interpretation = Interpretation {
            card_id_to_possibilities: [(card_id, direct_interpretation_focus_possibilities)].into(),
        };

        Interpretations {
            ors: vec![direct_interpretation],
        }
    }

    fn fr_apply_hint(
        &mut self,
        hinted_property: Property,
        positions: PositionSet,
        state: &PublicState,
    ) {
        for id in 1..=self.cards.current_hand_size {
            let possible = &mut self.possible_cards[&self.cards.cards[id].unwrap()];
            if positions.contains(id) {
                possible.apply(hinted_property);
            } else {
                possible.apply_not(hinted_property);
            }
        }

        self.interpretations
            .push(self.get_hint_interpretations(hinted_property, positions, state));
    }

    fn touched(&self) -> PositionSet {
        let positions = self
            .cards
            .cards
            .map(|maybe_id| maybe_id.is_some_and(|id| self.touched.contains(&id)));

        PositionSet { positions }
    }

    fn suggest_play(&self, state: &PublicState) -> Option<usize> {
        let mut possible = self.possible_cards.clone();
        for inter in &self.interpretations {
            if inter.ors.len() == 1 {
                let inter = &inter.ors[0];
                for (&card_id, ps) in &inter.card_id_to_possibilities {
                    //Currently, interpretations might contain restrictions about cards that are not on the hand anymore.
                    if let Some(my_ps) = possible.get_mut(&card_id) {
                        my_ps.intersect(ps);
                    }
                }
            }
        }

        for (card_id, p) in possible {
            assert!(!p.is_empty());
            if p.hashed.iter().all(|card| state.is_playable(card)) {
                let pos = self.cards.find(card_id).unwrap();
                return Some(pos);
            }
        }

        None
    }
}

struct HandCards {
    cards: [Option<usize>; 6],
    current_hand_size: usize,
}

impl HandCards {
    fn add_card(&mut self, id: usize) {
        let mut open = Some(id);
        let mut next_index = 1;
        while open.is_some() {
            swap(&mut open, &mut self.cards[next_index]);
            next_index += 1;
        }

        self.current_hand_size += 1;
        assert_eq!(next_index, self.current_hand_size + 1);
    }

    fn play_card(&mut self, mut position: usize) -> usize {
        while position < self.current_hand_size {
            self.cards.swap(position, position + 1);
            position += 1;
        }

        let id = self.cards[position].unwrap();
        self.cards[position] = None;
        self.current_hand_size -= 1;
        id
    }

    fn get(&self, position: usize) -> usize {
        self.cards[position].unwrap()
    }

    fn new() -> Self {
        Self {
            cards: [None; 6],
            current_hand_size: 0,
        }
    }

    fn find(&self, card_id: usize) -> Option<usize> {
        self.cards.iter().position(|&id| id == Some(card_id))
    }
}

struct Interpretations {
    ors: Vec<Interpretation>,
}

struct Interpretation {
    card_id_to_possibilities: IndexMap<usize, PossibleCards>,
}
