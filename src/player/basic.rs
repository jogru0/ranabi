use std::mem::swap;

use indexmap::{IndexMap, IndexSet};

use crate::{
    card::{Card, Number, PossibleCards},
    player::basic::action_assessment::ActionType,
    state::{PublicState, Rules},
};

use self::{
    action_assessment::ActionAssessment,
    inter::{Interpretation, Interpretations},
};

use super::{action::Action, Player, PositionSet, Property};

pub struct BasicPlayer {
    player_states: Vec<PlayerState>,
    witnessed_cards: Vec<Option<Card>>,
    player_id: usize,
    public_state: PublicState,
}

impl BasicPlayer {
    fn rules(&self) -> &Rules {
        &self.public_state.rules
    }

    // fn hand_as_set(&self, other_player_id: usize) -> PossibleCards {
    //     let mut result = PossibleCards::none();

    //     let cards = &self.player_states[other_player_id].cards;

    //     for pos in 1..=cards.current_hand_size {
    //         let card_id = cards.cards[pos].unwrap();
    //         let card = self.witnessed_cards[card_id].unwrap();
    //         result.add(card);
    //     }

    //     result
    // }

    // fn visible_in_hands(&self) -> PossibleCards {
    //     let mut result = PossibleCards::none();
    //     for other_player_id in 0..self.rules().number_of_players {
    //         if other_player_id == self.player_id {
    //             continue;
    //         }

    //         result.extend(self.hand_as_set(other_player_id));
    //     }

    //     result
    // }

    fn possible_touches_in_own_hand_or_more(&self) -> PossibleCards {
        let mut result = PossibleCards::all(self.rules());

        let played = self.public_state.firework.already_played();
        result.exclude(&played);

        let touched_in_other_hands = self.touched_in_other_hands();
        result.exclude(&touched_in_other_hands);

        result
    }

    fn good_touchable_or_more(&self) -> PossibleCards {
        let mut result = PossibleCards::all(self.rules());

        let definite_trash = self.public_state.definite_trash();
        result.exclude(&definite_trash);

        let touched_in_other_hands = self.touched_in_other_hands();
        result.exclude(&touched_in_other_hands);

        result
    }

    fn possible_touches_in_own_hand(&self) -> PossibleCards {
        let mut result = PossibleCards::none();

        let self_state = &self.player_states[self.player_id];
        for card_id in &self_state.touched {
            //We could apply additional information we have from interpretations to try to recuce the possible touches.
            //However, maybe we don't alyways want to do that? Not sure at the moment.
            //For now, it should suffice to just use the hint information directly.
            result.extend(self_state.possible_cards[card_id].clone());
        }

        result
    }

    fn good_touchable_or_less(&self) -> PossibleCards {
        let mut result = self.good_touchable_or_more();
        result.exclude(&self.possible_touches_in_own_hand());
        result
    }

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

        let receiver_cards = &self.player_states[receiver].cards;

        let mut positions = [false; 6];

        for (pos, pos_b) in positions.iter_mut().enumerate() {
            let Some(card_id) = receiver_cards.cards[pos] else {
                continue;
            };
            let card = self.witnessed_cards[card_id].unwrap();
            if card.satisfies(hinted_property) {
                *pos_b = true;
            }
        }

        PositionSet {
            positions,
            hand_size: receiver_cards.current_hand_size,
        }
    }

    fn assess_hint(
        &self,
        receiver: usize,
        hinted_property: Property,
        positions: PositionSet,
    ) -> ActionAssessment {
        assert_eq!(positions, self.get_positions(hinted_property, receiver));
        assert!(!positions.is_empty() || self.rules().allow_null_hints());
        assert_ne!(receiver, self.player_id);
        assert_eq!(
            positions.hand_size,
            self.player_states[receiver].cards.current_hand_size
        );

        let interpretations = self.player_states[receiver].get_hint_interpretations(
            hinted_property,
            positions,
            &self.public_state,
        );

        let correct_interpretation = interpretations.unwrap().get_truth(&self.witnessed_cards);

        if correct_interpretation.is_none() {
            return ActionAssessment::unconvectional();
        }

        let new_cards = self.new_cards(positions, receiver);
        let mut touchable = self.good_touchable_or_less();
        for &new_card in &new_cards {
            let succ = touchable.remove(&self.witnessed_cards[new_card].unwrap());
            if !succ {
                return ActionAssessment::unconvectional();
            }
        }

        //We probably need to add cards gotten by the correct interpretation here.
        //And delay might also be faster due to prompts/finesses, or slower due to delayed play cues.
        ActionAssessment {
            is_unconventional: false,

            new_touches: new_cards.len(),
            delay_until_relevant: (self.rules().number_of_players + receiver - self.player_id)
                % self.rules().number_of_players,
            action_type: ActionType::Hint,
            sure_influence_on_clue_count: -1,
        }
    }

    fn assess_hints(&self) -> Vec<(ActionAssessment, Action)> {
        let mut options = Vec::new();

        for receiver in 0..self.public_state.rules.number_of_players {
            if receiver == self.player_id {
                continue;
            }

            for hinted_property in Property::all(&self.public_state.rules) {
                let positions = self.get_positions(hinted_property, receiver);
                if positions.is_empty() {
                    continue;
                }

                let assessment = self.assess_hint(receiver, hinted_property, positions);

                options.push((
                    assessment,
                    Action::Hint {
                        receiver,
                        hinted_property,
                        positions,
                    },
                ));
            }
        }

        options
    }

    fn touched_in_other_hand(&self, player_id: usize) -> PossibleCards {
        let mut result = PossibleCards::none();

        for &card_id in &self.player_states[player_id].touched {
            result.add(self.witnessed_cards[card_id].unwrap())
        }

        result
    }

    fn touched_in_other_hands(&self) -> PossibleCards {
        let mut result = PossibleCards::none();
        {
            for player_id in 0..self.rules().number_of_players {
                if player_id == self.player_id {
                    continue;
                }
                result.extend(self.touched_in_other_hand(player_id));
            }

            result
        }
    }

    fn new_cards(&self, positions: PositionSet, receiver: usize) -> Vec<usize> {
        let mut result = Vec::new();

        let receiver_state = &self.player_states[receiver];

        for (pos, included) in positions.positions.iter().enumerate() {
            if !included {
                continue;
            }

            let card_id = receiver_state.cards.cards[pos].unwrap();
            if !receiver_state.touched.contains(&card_id) {
                result.push(card_id);
            }
        }

        result
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
                self.public_state.discard(card);
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
        let mut options = Vec::new();

        options.extend(self.player_states[self.player_id].suggest_plays(
            &self.public_state,
            &self.possible_touches_in_own_hand_or_more(),
        ));

        if self.public_state.clues != 0 {
            options.extend(self.assess_hints());
        }

        if self.public_state.clues != self.public_state.rules.max_clues {
            options.extend(self.player_states[self.player_id].suggest_discards());
        }

        options.sort_by_key(|(hint_value, _)| *hint_value);
        *options
            .last()
            .filter(|(a, _)| !a.is_unconventional)
            .map(|(_, v)| v)
            .unwrap()
    }
}

struct PlayerState {
    cards: HandCards,
    //I think this currently is strictly just what follows explicitely from hints.
    //Maybe this information should live in the public information?
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
        let id = self.cards.play_or_discard_card(position);
        let removed = self.possible_cards.remove(&id);
        assert!(removed.is_some());
        self.touched.remove(&id);
    }

    fn chop_position(&self) -> Option<usize> {
        for pos in (1..=self.cards.current_hand_size).rev() {
            let id = self.cards.cards[pos].unwrap();
            if !self.touched.contains(&id) {
                return Some(pos);
            }
        }
        None
    }

    fn get_hint_interpretations(
        &self,
        hinted_property: Property,
        positions: PositionSet,
        state: &PublicState,
    ) -> Option<Interpretations> {
        let touched_positions = self.touched_positions();
        let focus_position =
            positions.focus_position(touched_positions, self.cards.current_hand_size);
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

    fn fr_apply_hint(
        &mut self,
        hinted_property: Property,
        positions: PositionSet,
        state: &PublicState,
    ) {
        self.interpretations.push(
            self.get_hint_interpretations(hinted_property, positions, state)
                .unwrap(),
        );

        //Ugh. This influences hint interpretation, which I don't like at all. For now, we just do this after the interpretation thing.
        for pos in 1..=self.cards.current_hand_size {
            let card_id = self.cards.cards[pos].unwrap();
            let possible = &mut self.possible_cards[&card_id];
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

    fn assess_play(
        &self,
        position: usize,
        state: &PublicState,
        at_least_all_candidates_for_touched: &PossibleCards,
    ) -> ActionAssessment {
        let card_id = self.cards.cards[position].unwrap();
        let mut possible = self.possible_cards[&card_id].clone();

        for inter in &self.interpretations {
            if let Some(inter) = inter.unique_interpretation() {
                if let Some(ps) = inter.card_id_to_possibilities.get(&card_id) {
                    possible.intersect(ps);
                }
            }
        }

        if self.touched.contains(&card_id) {
            possible.intersect(at_least_all_candidates_for_touched);
        }

        if possible.is_empty() {
            panic!()
        }

        if possible.hashed.iter().all(|card| state.is_playable(card)) {
            let sure_influence_on_clue_count = if possible
                .hashed
                .iter()
                .all(|card| card.number == Number::Five)
            {
                1
            } else {
                0
            };

            return ActionAssessment {
                new_touches: 0,
                delay_until_relevant: 0,
                is_unconventional: false,
                action_type: ActionType::Play,
                sure_influence_on_clue_count,
            };
        }

        ActionAssessment::unconvectional()
    }

    fn suggest_discards(&self) -> Vec<(ActionAssessment, Action)> {
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

    fn suggest_plays(
        &self,
        state: &PublicState,
        at_least_all_candidates_for_touched: &PossibleCards,
    ) -> Vec<(ActionAssessment, Action)> {
        let options: Vec<_> = (1..=self.cards.current_hand_size)
            .map(|position| {
                (
                    self.assess_play(position, state, at_least_all_candidates_for_touched),
                    Action::Play {
                        card: None,
                        position,
                    },
                )
            })
            .collect();

        options
    }

    fn assess_discard(&self, position: usize) -> ActionAssessment {
        if Some(position) == self.chop_position() {
            ActionAssessment {
                new_touches: 0,
                delay_until_relevant: 0,
                is_unconventional: false,
                action_type: ActionType::Discard,
                sure_influence_on_clue_count: 1,
            }
        } else {
            ActionAssessment::unconvectional()
        }
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

    fn play_or_discard_card(&mut self, mut position: usize) -> usize {
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
}

mod inter {
    use indexmap::IndexMap;

    use crate::card::{Card, PossibleCards};

    #[derive(Debug)]
    pub struct Interpretations {
        ors: Vec<Interpretation>,
    }

    impl Interpretations {
        pub fn new(ors: Vec<Interpretation>) -> Option<Self> {
            if ors.is_empty() {
                None
            } else {
                Some(Self { ors })
            }
        }

        pub fn unique_interpretation(&self) -> Option<&Interpretation> {
            if self.ors.len() == 1 {
                Some(&self.ors[0])
            } else {
                None
            }
        }

        pub fn get_truth(&self, witnessed_cards: &[Option<Card>]) -> Option<Interpretation> {
            let mut result = None;
            for or in &self.ors {
                if or.is_true(witnessed_cards) {
                    assert!(result.is_none());
                    result = Some(or);
                }
            }

            result.cloned()
        }
    }

    //Currently, interpretations might contain restrictions about cards that are not on the hand anymore.
    #[derive(Debug, Clone)]
    pub struct Interpretation {
        pub card_id_to_possibilities: IndexMap<usize, PossibleCards>,
    }
    impl Interpretation {
        fn is_true(&self, witnessed_cards: &[Option<Card>]) -> bool {
            self.card_id_to_possibilities
                .iter()
                .all(|(&card_id, possibilities)| {
                    possibilities.contains(&witnessed_cards[card_id].unwrap())
                })
        }
    }
}

mod action_assessment;
