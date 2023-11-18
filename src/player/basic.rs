use std::mem::swap;

use indexmap::{IndexMap, IndexSet};

use crate::{
    card::{Card, Number, PossibleCards},
    state::{PublicState, Rules},
};

use self::{
    hint_value::HintValue,
    inter::{Interpretation, Interpretations},
};

use super::{Action, Player, PositionSet, Property};

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

    fn suggest_hint(&self) -> Option<(usize, Property, PositionSet)> {
        let mut options = Vec::new();

        for receiver in 0..self.public_state.rules.number_of_players {
            if receiver == self.player_id {
                continue;
            }

            'property: for hinted_property in Property::all(&self.public_state.rules) {
                let positions = self.get_positions(hinted_property, receiver);
                if positions.is_empty() {
                    continue;
                }

                let interpretations = self.player_states[receiver].get_hint_interpretations(
                    hinted_property,
                    positions,
                    &self.public_state,
                );

                let correct_interpretation =
                    interpretations.unwrap().get_truth(&self.witnessed_cards);

                if correct_interpretation.is_none() {
                    continue;
                }

                let new_cards = self.new_cards(positions, receiver);
                let mut touchable = self.good_touchable_or_less();
                for &new_card in &new_cards {
                    let succ = touchable.remove(&self.witnessed_cards[new_card].unwrap());
                    if !succ {
                        continue 'property;
                    }
                }

                //We probably need to add cards gotten by the correct interpretation here.
                //And delay might also be faster due to prompts/finesses, or slower due to delayed play queue.
                let hint_value = HintValue {
                    new_touches: new_cards.len(),
                    delay_until_relevant: (self.rules().number_of_players + receiver
                        - self.player_id)
                        % self.rules().number_of_players,
                };

                options.push((hint_value, (receiver, hinted_property, positions)));
            }
        }

        options.sort_by_key(|(hint_value, _)| *hint_value);
        options.last().map(|(_, v)| v).copied()
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
        if let Some(index) = self.player_states[self.player_id].suggest_play(
            &self.public_state,
            &self.possible_touches_in_own_hand_or_more(),
        ) {
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

    fn suggest_play(
        &self,
        state: &PublicState,
        at_least_all_candidates_for_touched: &PossibleCards,
    ) -> Option<usize> {
        let mut possible = self.possible_cards.clone();

        for inter in &self.interpretations {
            if let Some(inter) = inter.unique_interpretation() {
                for (&card_id, ps) in &inter.card_id_to_possibilities {
                    //Currently, interpretations might contain restrictions about cards that are not on the hand anymore.
                    if let Some(my_ps) = possible.get_mut(&card_id) {
                        my_ps.intersect(ps);
                    }
                }
            }
        }

        for touched in &self.touched {
            possible[touched].intersect(at_least_all_candidates_for_touched);
        }

        for (card_id, p) in possible {
            if p.is_empty() {
                panic!()
            }
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

    fn find(&self, card_id: usize) -> Option<usize> {
        self.cards.iter().position(|&id| id == Some(card_id))
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

mod hint_value {

    #[derive(PartialEq, Eq, Clone, Copy)]
    pub struct HintValue {
        pub new_touches: usize,
        pub delay_until_relevant: usize,
    }
}

impl PartialOrd for HintValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for HintValue {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.new_touches.cmp(&other.new_touches) {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        other.delay_until_relevant.cmp(&self.delay_until_relevant)
    }
}
