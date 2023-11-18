use indexmap::{IndexMap, IndexSet};

use crate::{
    card::{Number, PossibleCards},
    player::{
        action::Action,
        basic::inter::{Interpretation, Interpretations},
        PositionSet, Property,
    },
    state::{Firework, PublicState, Rules},
};

use super::{
    action_assessment::{ActionAssessment, ActionType},
    HandCards,
};

#[derive(Clone)]
pub struct PlayerState {
    pub cards: HandCards,
    //I think this currently is strictly just what follows explicitely from hints.
    //Maybe this information should live in the public information?
    pub objectively_possible_cards_according_to_hints: IndexMap<usize, PossibleCards>,
    pub touched: IndexSet<usize>,
    interpretations_some_of_which_self_should_entertain: Vec<Interpretations>,
}

impl PlayerState {
    pub fn new() -> Self {
        Self {
            cards: HandCards::new(),
            objectively_possible_cards_according_to_hints: IndexMap::new(),
            touched: IndexSet::new(),
            interpretations_some_of_which_self_should_entertain: Vec::new(),
        }
    }

    pub fn add_card(&mut self, id: usize, rules: &Rules) {
        self.cards.add_card(id);
        let previous = self
            .objectively_possible_cards_according_to_hints
            .insert(id, PossibleCards::all(rules));
        assert!(previous.is_none());
    }

    pub fn play_or_discard_card(&mut self, position: usize) {
        let id = self.cards.play_or_discard_card(position);
        let removed = self
            .objectively_possible_cards_according_to_hints
            .remove(&id);
        assert!(removed.is_some());
        self.touched.remove(&id);
    }

    pub fn chop_position(&self) -> Option<usize> {
        for pos in (1..=self.cards.current_hand_size).rev() {
            let id = self.cards.cards[pos].unwrap();
            if !self.touched.contains(&id) {
                return Some(pos);
            }
        }
        None
    }

    pub fn get_hint_interpretations(
        &self,
        hinted_property: Property,
        positions: PositionSet,
        state: &PublicState,
    ) -> Option<Interpretations> {
        assert_eq!(self.cards.current_hand_size, positions.hand_size);

        let touched_positions = self.touched_positions();
        let focus_position = positions.focus_position(touched_positions);
        let touches_no_new_cards = touched_positions.contains(focus_position);

        if touches_no_new_cards {
            //TODO: This implies more.
        }

        let focus_card_id = self.cards.cards[focus_position].unwrap();

        let is_chop_focused = self.chop_position() == Some(focus_position);

        assert!(!(is_chop_focused && touches_no_new_cards));

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
            card_id_to_possibilities: [(focus_card_id, direct_interpretation_focus_possibilities)]
                .into(),
        };

        Interpretations::new(vec![direct_interpretation])
    }

    pub fn fr_apply_hint(
        &mut self,
        hinted_property: Property,
        positions: PositionSet,
        state: &PublicState,
    ) {
        assert_eq!(self.cards.current_hand_size, positions.hand_size);

        self.interpretations_some_of_which_self_should_entertain
            .push(
                self.get_hint_interpretations(hinted_property, positions, state)
                    .unwrap(),
            );

        //Ugh. This influences hint interpretation, which I don't like at all. For now, we just do this after the interpretation thing.
        for pos in 1..=self.cards.current_hand_size {
            let card_id = self.cards.cards[pos].unwrap();
            let possible = &mut self.objectively_possible_cards_according_to_hints[&card_id];
            if positions.contains(pos) {
                possible.apply(hinted_property);
                self.touched.insert(card_id);
            } else {
                possible.apply_not(hinted_property);
            }
        }
    }

    fn touched_positions(&self) -> PositionSet {
        let positions = self
            .cards
            .cards
            .map(|maybe_id| maybe_id.is_some_and(|id| self.touched.contains(&id)));

        PositionSet {
            positions,
            hand_size: self.cards.current_hand_size,
        }
    }

    pub fn possibilities_self_might_entertain(
        &self,
        position: usize,
        potentially_entertained_candidates_for_touched_in_own_hand: &PossibleCards,
    ) -> PossibleCards {
        let card_id = self.cards.cards[position].unwrap();
        let mut possible = self.objectively_possible_cards_according_to_hints[&card_id].clone();

        for inter in &self.interpretations_some_of_which_self_should_entertain {
            if let Some(inter) = inter.unique_interpretation() {
                if let Some(ps) = inter.card_id_to_possibilities.get(&card_id) {
                    possible.intersect(ps);
                }
            }
        }

        if self.touched.contains(&card_id) {
            possible.intersect(potentially_entertained_candidates_for_touched_in_own_hand);
        }

        possible
    }

    fn is_definitely_aware_that_this_position_is_playable(
        &self,
        position: usize,
        firework: &Firework,
        potentially_entertained_candidates_for_touched_in_own_hand: &PossibleCards,
    ) -> bool {
        let possibilities = self.possibilities_self_might_entertain(
            position,
            potentially_entertained_candidates_for_touched_in_own_hand,
        );
        firework.are_all_playable(&possibilities)
    }

    fn is_definitely_aware_about_a_playable_card(
        &self,
        firework: &Firework,
        potentially_entertained_candidates_for_touched_in_own_hand: &PossibleCards,
    ) -> bool {
        (1..=self.cards.current_hand_size).any(|position| {
            self.is_definitely_aware_that_this_position_is_playable(
                position,
                firework,
                potentially_entertained_candidates_for_touched_in_own_hand,
            )
        })
    }

    pub fn potentially_is_locked_with_no_known_playable_card(
        &self,
        firework: &Firework,
        potentially_entertained_candidates_for_touched_in_own_hand: &PossibleCards,
    ) -> bool {
        self.touched_positions().is_full()
            && !self.is_definitely_aware_about_a_playable_card(
                firework,
                potentially_entertained_candidates_for_touched_in_own_hand,
            )
    }

    pub fn suggest_discards(&self) -> Vec<(ActionAssessment, Action)> {
        (1..=self.cards.current_hand_size)
            .map(|position| {
                (
                    self.assess_discard(position),
                    Action::Discard {
                        card: None,
                        position,
                    },
                )
            })
            .collect()
    }

    pub fn assess_discard(&self, position: usize) -> ActionAssessment {
        let last_resort = if let Some(chop_position) = self.chop_position() {
            if position != chop_position {
                return ActionAssessment::unconvectional();
            }
            false
        } else {
            true
        };
        ActionAssessment {
            new_touches: 0,
            delay_until_relevant: 0,
            is_unconventional: false,
            action_type: ActionType::Discard,
            sure_influence_on_clue_count: 1,
            last_resort,
            next_player_might_be_locked_with_no_clue: false,
        }
    }
}
