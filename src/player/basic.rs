use std::mem::swap;

use crate::{
    card::{Card, Number, PossibleCards},
    player::basic::action_assessment::ActionType,
    state::{PublicState, Rules},
};

use self::{action_assessment::ActionAssessment, player_state::PlayerState};

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

    //For now, ignoring cards that can be excluded as all copies are seen elsewhere.
    //In reality, less candidates might be entertained. Definitely not more!
    fn potentially_entertained_candidates_for_touched_in_that_players_own_hand(
        &self,
        player_id: usize,
    ) -> PossibleCards {
        let mut result = PossibleCards::all(self.rules());

        let played = self.public_state.firework.already_played();
        result.exclude(&played);

        let touched_in_other_hands = self.touched_in_other_hands_or_less(player_id);
        result.exclude(&touched_in_other_hands);

        result
    }

    fn possible_touches_in_hand(&self, player: usize) -> PossibleCards {
        //Didn't think about anything else so far.
        assert_eq!(player, self.player_id);

        let mut result = PossibleCards::none();

        let self_state = &self.player_states[self.player_id];
        for &card_id in &self_state.touched {
            //We could apply additional information we have from interpretations to try to recuce the possible touches.
            //However, maybe we don't alyways want to do that? Not sure at the moment.
            //For now, it should suffice to just use the hint information directly.
            result.extend(
                self_state
                    .objectively_possible_cards_according_to_hints_minus_visible_full_sets(
                        card_id,
                        &self.cards_that_player_definitely_sees_all_copies_of(player),
                    )
                    .clone(),
            );
        }

        result
    }

    //TODO!!!!! Needs to be in the (hypthetical) state!
    fn definitely_good_touchable_cards_definitely_known_by_this_player(
        &self,
        player: usize,
    ) -> PossibleCards {
        let mut result = PossibleCards::all(self.rules());

        let definite_trash = self.public_state.definite_trash();
        result.exclude(&definite_trash);

        let touched_in_other_hands = self.touched_in_other_hands_or_less(player);
        result.exclude(&touched_in_other_hands);

        result.exclude(&self.possible_touches_in_hand(player));
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

    fn assess_hint_this_player(
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
        assert_ne!(self.public_state.clues, 0);
        assert_eq!(
            self.player_states[receiver].cards.current_hand_size,
            positions.hand_size
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
        let mut touchable =
            self.definitely_good_touchable_cards_definitely_known_by_this_player(self.player_id);
        for &new_card in &new_cards {
            let succ = touchable.remove(&self.witnessed_cards[new_card].unwrap());
            if !succ {
                return ActionAssessment::unconvectional();
            }
        }

        //We probably need to add cards gotten by the correct interpretation here.
        //And delay might also be faster due to prompts/finesses, or slower due to delayed play cues.
        ActionAssessment::new(
            new_cards.len(),
            (self.rules().number_of_players + receiver - self.player_id)
                % self.rules().number_of_players,
            ActionType::Hint,
            -1,
            false,
            0,
        )
    }

    fn apply_hypothetical(&self, action: Action, assessment: &mut ActionAssessment) {
        let mut hypothetical_next = self.player_states[self.next_player_id()].clone();
        let mut hypothetical_state = self.public_state.clone();

        hypothetical_state.apply_action(action);

        if let Action::Hint {
            receiver,
            hinted_property,
            positions,
        } = action
        {
            if receiver == self.next_player_id() {
                hypothetical_next.fr_apply_hint(hinted_property, positions, &hypothetical_state);
            }
        }

        let next_player_might_be_locked_with_no_clue = hypothetical_state.clues == 0
            && hypothetical_next.potentially_is_locked_with_no_known_playable_card(
                &hypothetical_state.firework,
                &self.potentially_entertained_candidates_for_touched_in_that_players_own_hand(
                    self.next_player_id(),
                ),
                &self.touched_in_other_hands_or_more(self.next_player_id()),
                &self.cards_that_player_definitely_sees_all_copies_of(self.next_player_id()),
            );

        assessment.next_player_might_be_locked_with_no_clue =
            next_player_might_be_locked_with_no_clue;
    }

    fn assess_plays_this_player(&self) -> Vec<(ActionAssessment, Action)> {
        let options: Vec<_> = (1..=self.this_player().cards.current_hand_size)
            .map(|position| {
                (
                    self.assess_play_this_player(position),
                    Action::Play {
                        card: None,
                        position,
                    },
                )
            })
            .collect();

        options
    }

    fn assess_hints_this_player(&self) -> Vec<(ActionAssessment, Action)> {
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

                let assessment = self.assess_hint_this_player(receiver, hinted_property, positions);

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
        assert_ne!(player_id, self.player_id);
        let mut result = PossibleCards::none();

        for &card_id in &self.player_states[player_id].touched {
            result.add(self.witnessed_cards[card_id].unwrap())
        }

        result
    }

    fn touched_in_other_hands_or_less(&self, player_id: usize) -> PossibleCards {
        self.touched_visible_by_me_and(player_id)
    }

    fn touched_not_by_me(&self) -> PossibleCards {
        let mut result = PossibleCards::none();
        for p_id in 0..self.rules().number_of_players {
            if p_id == self.player_id {
                continue;
            }
            result.extend(self.touched_in_other_hand(p_id));
        }
        result
    }

    fn touched_in_other_hands_or_more(&self, player_id: usize) -> PossibleCards {
        assert_ne!(player_id, self.player_id);
        let mut result = self.touched_visible_by_me_and(player_id);
        result.merge(&self.possible_touches_in_hand(self.player_id));
        result
    }

    fn touched_visible_by_me_and(&self, player_id: usize) -> PossibleCards {
        let mut result = PossibleCards::none();
        {
            for p_id in 0..self.rules().number_of_players {
                if p_id == self.player_id || p_id == player_id {
                    continue;
                }
                result.extend(self.touched_in_other_hand(p_id));
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

    fn next_player_id(&self) -> usize {
        (self.player_id + 1) % self.rules().number_of_players
    }

    fn assess_play_this_player(&self, position: usize) -> ActionAssessment {
        let possibilities = self.this_player().possibilities_self_might_entertain(
            position,
            &self.potentially_entertained_candidates_for_touched_in_that_players_own_hand(
                self.player_id,
            ),
            &self.cards_that_player_definitely_sees_all_copies_of(self.player_id),
        );

        let is_touched = self.this_player().touched_positions().contains(position);

        if self
            .this_player()
            .is_definitely_aware_that_these_are_all_playable_right_now(
                &possibilities,
                is_touched,
                &self.public_state.firework,
                &self.touched_not_by_me(),
            )
        {
            let sure_influence_on_clue_count = if possibilities
                .hashed
                .iter()
                .all(|card| card.number == Number::Five)
            {
                1
            } else {
                0
            };

            return ActionAssessment::new(
                0,
                0,
                ActionType::Play,
                sure_influence_on_clue_count,
                false,
                1,
            );
        }

        ActionAssessment::new(0, 0, ActionType::Play, 0, true, 0)
    }

    fn this_player(&self) -> &PlayerState {
        &self.player_states[self.player_id]
    }

    fn assedd_discards_this_player(&self) -> Vec<(ActionAssessment, Action)> {
        (1..=self.this_player().cards.current_hand_size)
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
        let last_resort = if let Some(chop_position) = self.this_player().chop_position() {
            if position != chop_position {
                return ActionAssessment::unconvectional();
            }
            false
        } else {
            true
        };
        ActionAssessment::new(0, 0, ActionType::Discard, 1, last_resort, 0)
    }

    fn cards_that_player_definitely_sees_all_copies_of(&self, player_id: usize) -> PossibleCards {
        let mut pile = self.public_state.discard_pile.clone();
        for card in self.public_state.firework.already_played().hashed {
            pile.add(&card);
        }

        for p_id in 0..self.rules().number_of_players {
            if p_id == self.player_id || p_id == player_id {
                continue;
            }

            let cards = &self.player_states[p_id].cards;

            for pos in 1..=cards.current_hand_size {
                let card_id = cards.cards[pos].unwrap();
                pile.add(&self.witnessed_cards[card_id].unwrap())
            }
        }

        pile.full_sets(self.rules())
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
        //This assumes that self.player_id is active.
        let mut options = Vec::new();

        options.extend(self.assess_plays_this_player());

        if self.public_state.clues != 0 {
            options.extend(self.assess_hints_this_player());
        }

        if self.public_state.clues != self.public_state.rules.max_clues {
            options.extend(self.assedd_discards_this_player());
        }

        options.retain(|(a, _)| !a.is_unconventional());

        for &mut (ref mut assessment, action) in &mut options {
            self.apply_hypothetical(action, assessment);
        }

        options.sort_by_key(|(hint_value, _)| *hint_value);
        *options.last().map(|(_, v)| v).unwrap()
    }
}

mod player_state;

#[derive(Clone)]
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

    #[derive(Debug, Clone)]
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
