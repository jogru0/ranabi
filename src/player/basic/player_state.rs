use indexmap::{IndexMap, IndexSet};

use crate::{
    card::{card_set::CardSet, Number},
    player::{
        basic::inter::{Interpretation, Interpretations},
        PositionSet, Property,
    },
    state::{Firework, PublicState},
};

use super::HandCards;

#[derive(Clone)]
pub struct PlayerState {
    pub cards: HandCards,
    //I think this currently is strictly just what follows explicitely from hints.
    //Maybe this information should live in the public information?
    pub objectively_possible_cards_according_to_hints3: IndexMap<usize, CardSet>,
    pub touched: IndexSet<usize>,
    interpretations_some_of_which_self_should_entertain: Vec<Interpretations>,
}

impl PlayerState {
    pub fn new() -> Self {
        Self {
            cards: HandCards::new(),
            objectively_possible_cards_according_to_hints3: IndexMap::new(),
            touched: IndexSet::new(),
            interpretations_some_of_which_self_should_entertain: Vec::new(),
        }
    }

    pub fn add_card(&mut self, id: usize) {
        self.cards.add_card(id);
        let previous = self
            .objectively_possible_cards_according_to_hints3
            .insert(id, CardSet::all());
        assert!(previous.is_none());
    }

    pub fn play_or_discard_card(&mut self, position: usize) {
        let id = self.cards.play_or_discard_card(position);
        let removed = self
            .objectively_possible_cards_according_to_hints3
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
        giver_stall_severity: usize,
        all_surely_known_touched_cards_in_hands: &CardSet,
    ) -> Option<Interpretations> {
        assert_eq!(self.cards.current_hand_size, positions.hand_size);

        let touched_positions = self.touched_positions();
        let focus_position = positions.focus_position(touched_positions);
        let touches_no_new_cards = touched_positions.contains(focus_position);

        let potential_burned_clue = 2 <= giver_stall_severity && touches_no_new_cards;

        if potential_burned_clue {
            return Interpretations::new(vec![Interpretation::no_additional_info()]);
        }

        let focus_card_id = self.cards.cards[focus_position].unwrap();

        let is_chop_focused = self.chop_position() == Some(focus_position);

        assert!(!(is_chop_focused && touches_no_new_cards));

        let mut direct_interpretation_focus_possibilities = CardSet::none();

        let delayed_playable = state
            .firework
            .delayed_playable(all_surely_known_touched_cards_in_hands);
        direct_interpretation_focus_possibilities.extend(delayed_playable);

        if is_chop_focused {
            let critical_saves = state.critical_saves();
            direct_interpretation_focus_possibilities.extend(critical_saves);

            if hinted_property == Property::Number(Number::Two)
                || hinted_property == Property::Number(Number::Five)
            {
                let special_saves = CardSet::with_property(hinted_property);
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
        giver_stall_severity: usize,
        all_surely_known_touched_cards_in_hands: &CardSet,
    ) {
        assert_eq!(self.cards.current_hand_size, positions.hand_size);

        self.interpretations_some_of_which_self_should_entertain
            .push(
                self.get_hint_interpretations(
                    hinted_property,
                    positions,
                    state,
                    giver_stall_severity,
                    all_surely_known_touched_cards_in_hands,
                )
                .unwrap(),
            );

        //Ugh. This influences hint interpretation, which I don't like at all. For now, we just do this after the interpretation thing.
        for pos in 1..=self.cards.current_hand_size {
            let card_id = self.cards.cards[pos].unwrap();
            let possible = &mut self.objectively_possible_cards_according_to_hints3[&card_id];
            if positions.contains(pos) {
                possible.apply(hinted_property);
                self.touched.insert(card_id);
            } else {
                possible.apply_not(hinted_property);
            }
        }
    }

    pub fn touched_positions(&self) -> PositionSet {
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
        potentially_entertained_candidates_for_touched_in_own_hand: &CardSet,
        cards_self_definitely_sees_all_copies_of: &CardSet,
    ) -> CardSet {
        let card_id = self.cards.cards[position].unwrap();

        //TODO: Access maybe only combined with the sees all copies of thing?
        let mut possible = self
            .objectively_possible_cards_according_to_hints_minus_visible_full_sets(
                card_id,
                cards_self_definitely_sees_all_copies_of,
            );

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

    pub fn is_definitely_aware_that_these_are_all_playable_right_now(
        &self,
        possibilities: &CardSet,
        is_already_touched_card: bool,
        firework: &Firework,
        touched_in_other_hands_or_more: &CardSet,
    ) -> bool {
        // Maybe for locked, we want to violate good touch here?
        // if !self.touched_positions().contains(position) {
        //     if possibilities.
        // }

        if !is_already_touched_card && possibilities.intersects(touched_in_other_hands_or_more) {
            false
        } else {
            firework.are_all_playable(possibilities)
        }
    }

    fn is_definitely_aware_that_this_position_is_playable(
        &self,
        position: usize,
        firework: &Firework,
        potentially_entertained_candidates_for_touched_in_own_hand: &CardSet,
        touched_in_other_hands_or_more: &CardSet,
        cards_self_definitely_sees_all_copies_of: &CardSet,
    ) -> bool {
        let possibilities = self.possibilities_self_might_entertain(
            position,
            potentially_entertained_candidates_for_touched_in_own_hand,
            cards_self_definitely_sees_all_copies_of,
        );
        let is_already_touched = self.touched_positions().contains(position);

        self.is_definitely_aware_that_these_are_all_playable_right_now(
            &possibilities,
            is_already_touched,
            firework,
            touched_in_other_hands_or_more,
        )
    }

    fn is_definitely_aware_about_a_playable_card(
        &self,
        firework: &Firework,
        potentially_entertained_candidates_for_touched_in_own_hand: &CardSet,
        touched_in_other_hands_or_more: &CardSet,
        cards_self_definitely_sees_all_copies_of: &CardSet,
    ) -> bool {
        (1..=self.cards.current_hand_size).any(|position| {
            self.is_definitely_aware_that_this_position_is_playable(
                position,
                firework,
                potentially_entertained_candidates_for_touched_in_own_hand,
                touched_in_other_hands_or_more,
                cards_self_definitely_sees_all_copies_of,
            )
        })
    }

    pub fn potentially_is_locked_with_no_known_playable_card(
        &self,
        firework: &Firework,
        potentially_entertained_candidates_for_touched_in_own_hand: &CardSet,
        touched_in_other_hands_or_more: &CardSet,
        cards_self_definitely_sees_all_copies_of: &CardSet,
    ) -> bool {
        self.touched_positions().is_full()
            && !self.is_definitely_aware_about_a_playable_card(
                firework,
                potentially_entertained_candidates_for_touched_in_own_hand,
                touched_in_other_hands_or_more,
                cards_self_definitely_sees_all_copies_of,
            )
    }

    pub(crate) fn objectively_possible_cards_according_to_hints_minus_visible_full_sets(
        &self,
        card_id: usize,
        visible_full_sets: &CardSet,
    ) -> CardSet {
        let mut result = self.objectively_possible_cards_according_to_hints3[&card_id].clone();
        result.exclude(visible_full_sets);
        result
    }

    pub(crate) fn stall_severity(
        &self,
        state: &PublicState,
        potentially_entertained_candidates_for_touched_in_own_hand: &CardSet,
        touched_in_other_hands_or_more: &CardSet,
        cards_self_definitely_sees_all_copies_of: &CardSet,
    ) -> usize {
        if state.clues == 8 {
            4
        } else if self.potentially_is_locked_with_no_known_playable_card(
            &state.firework,
            potentially_entertained_candidates_for_touched_in_own_hand,
            touched_in_other_hands_or_more,
            cards_self_definitely_sees_all_copies_of,
        ) {
            3
        } else {
            0
        }
    }
}
