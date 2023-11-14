use indexmap::{IndexMap, IndexSet};
use rand::seq::SliceRandom;
use rand_chacha::ChaCha20Rng;

use crate::{
    card::{Card, Color, Number, PossibleCards},
    player::{Action, Player},
};

pub struct Discard {
    card_to_multiplicity: IndexMap<Card, usize>,
}

impl Discard {
    fn contains(&self, card: &Card) -> bool {
        self.card_to_multiplicity[card] != 0
    }

    fn new(rules: &Rules) -> Self {
        Discard {
            card_to_multiplicity: rules.possible_cards().into_iter().map(|c| (c, 0)).collect(),
        }
    }

    fn unreachable(&self, rules: &Rules) -> PossibleCards {
        let mut result = PossibleCards::all(rules);
        for color in rules.used_colors() {
            let one = Card {
                color,
                number: Number::One,
            };
            if self.card_to_multiplicity[&one] == 3 {
                continue;
            }
            result.remove(&one);

            let two = Card {
                color,
                number: Number::Two,
            };
            if self.card_to_multiplicity[&two] == 2 {
                continue;
            }
            result.remove(&two);

            let three = Card {
                color,
                number: Number::Three,
            };
            if self.card_to_multiplicity[&three] == 2 {
                continue;
            }
            result.remove(&three);

            let four = Card {
                color,
                number: Number::Four,
            };
            if self.card_to_multiplicity[&four] == 2 {
                continue;
            }
            result.remove(&four);

            let five = Card {
                color,
                number: Number::Five,
            };
            if self.card_to_multiplicity[&five] == 1 {
                continue;
            }
            result.remove(&five);
        }

        result
    }
}

pub struct PublicState {
    pub firework: Firework,
    pub discard: Discard,
    pub rules: Rules,
    pub clues: usize,
    pub strikes: usize,
}
impl PublicState {
    pub(crate) fn critical_saves(&self) -> PossibleCards {
        let mut result = PossibleCards::none();
        for (&color, &maybe_number) in &self.firework.piles {
            let two = Card {
                color,
                number: Number::Two,
            };
            if self.discard.contains(&two) && maybe_number.is_none() {
                result.add(two);
            }

            let three = Card {
                color,
                number: Number::Three,
            };
            if self.discard.contains(&three) && maybe_number.is_none()
                || maybe_number == Some(Number::One)
            {
                result.add(three);
            }

            let four = Card {
                color,
                number: Number::Four,
            };
            if self.discard.contains(&four) && maybe_number.is_none()
                || maybe_number == Some(Number::One)
                || maybe_number == Some(Number::Two)
            {
                result.add(four);
            }
        }

        result
    }

    fn add_clue(&mut self) -> bool {
        let succ = self.clues < self.rules.max_clues;
        if succ {
            self.clues += 1
        }
        succ
    }

    pub(crate) fn discard(&mut self) {
        let succ = self.add_clue();
        assert!(succ);
    }

    pub(crate) fn play(&mut self, card: Card) {
        let succ = self.firework.add(card);
        if !succ {
            self.strikes += 1;
        } else if card.number == Number::Five {
            self.add_clue();
        }
    }

    pub(crate) fn hint(&mut self) {
        assert_ne!(self.clues, 0);
        self.clues -= 1;
    }

    pub(crate) fn is_playable(&self, card: &Card) -> bool {
        self.firework.is_playable(card)
    }

    pub(crate) fn new(rules: Rules) -> Self {
        Self {
            firework: Firework::new(&rules.used_colors()),
            discard: Discard::new(&rules),
            rules,
            clues: rules.max_clues,
            strikes: 0,
        }
    }

    pub(crate) fn definite_trash(&self) -> PossibleCards {
        let mut result = self.firework.already_played();
        result.merge(&self.discard.unreachable(&self.rules));
        result
    }
}

struct State {
    deck: Vec<Card>,
    active_player_id: usize,
    remaining_hints: usize,
    strikes: usize,
    firework: Firework,
    number_of_players: usize,
    number_of_actions_with_empty_deck: usize,
    hands: Vec<Vec<Card>>,
}
impl State {
    fn new(rules: &Rules, rng: &mut ChaCha20Rng) -> Self {
        let mut deck = rules.all_cards();
        deck.shuffle(rng);

        let firework = Firework::new(&rules.used_colors());
        let number_of_players = rules.number_of_players;

        Self {
            deck,
            active_player_id: 0,
            remaining_hints: rules.max_clues,
            strikes: 0,
            firework,
            number_of_players,
            number_of_actions_with_empty_deck: 0,
            hands: vec![Vec::new(); number_of_players],
        }
    }

    fn draw(&mut self) -> Option<Card> {
        let Some(new) = self.deck.pop() else {return None};
        self.hands[self.active_player_id].insert(0, new);
        Some(new)
    }

    fn is_concluded(&self) -> Option<Option<usize>> {
        if self.strikes == 3 {
            Some(None)
        } else if self.number_of_actions_with_empty_deck == self.number_of_players
            || self.firework.is_complete()
        {
            Some(Some(self.firework.score()))
        } else {
            None
        }
    }

    fn remove_card(&mut self, position: usize) -> Result<Card, RuleViolation> {
        let internal_index = position
            .checked_sub(1)
            .ok_or(RuleViolation::InvalidCardPosition)?;
        let cards = &mut self.hands[self.active_player_id];
        if cards.len() <= internal_index {
            return Err(RuleViolation::InvalidCardPosition);
        }
        Ok(cards.remove(internal_index))
    }

    fn apply_action(
        &mut self,
        action: Action,
        rules: &Rules,
    ) -> Result<(Option<Card>, Option<Card>), RuleViolation> {
        if self.deck.is_empty() {
            self.number_of_actions_with_empty_deck += 1;
        }

        match action {
            Action::Play {
                position,
                card: no_card,
            } => {
                if no_card.is_some() {
                    Err(RuleViolation::InvalidCardInformation)?;
                }

                let card = self.remove_card(position)?;

                if self.firework.add(card) {
                    println!("Played {card} successfully!");
                    if card.number == Number::Five && self.remaining_hints < rules.max_clues {
                        self.remaining_hints += 1;
                    }
                } else {
                    println!("Misplayed {card}!");
                    self.strikes += 1;
                }

                Ok((Some(card), self.draw()))
            }
            Action::Discard {
                card: no_card,
                position,
            } => {
                if no_card.is_some() {
                    Err(RuleViolation::InvalidCardInformation)?;
                }

                if self.remaining_hints == rules.max_clues {
                    Err(RuleViolation::NoMoreDiscardsAvailable)?;
                }

                let card = self.remove_card(position)?;

                self.remaining_hints += 1;

                Ok((Some(card), self.draw()))
            }

            Action::Hint {
                receiver,
                hinted_property,
                positions,
            } => {
                if receiver == self.active_player_id || self.number_of_players <= receiver {
                    Err(RuleViolation::InvalidReceiver)?;
                }

                if self.remaining_hints == 0 {
                    Err(RuleViolation::NoMoreHintsAvailable)?;
                }

                if positions.is_empty() {
                    Err(RuleViolation::NullHint)?;
                }

                for (ii, card) in self.hands[receiver].iter().enumerate() {
                    let i = ii + 1;
                    if card.satisfies(hinted_property) != positions.contains(i) {
                        Err(RuleViolation::IncorrectHint)?;
                    }
                }

                self.remaining_hints -= 1;
                Ok((None, None))
            }
        }
    }

    fn go_to_next_player(&mut self) {
        self.active_player_id += 1;
        self.active_player_id %= self.number_of_players;
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Rules {
    pub number_of_players: usize,
    pub hand_size: usize,
    pub max_clues: usize,
}

impl Rules {
    pub fn max_score(&self) -> usize {
        self.used_colors().len() * 5
    }

    fn possible_cards(&self) -> Vec<Card> {
        let mut result = Vec::with_capacity(5 * self.used_colors().len());
        for color in self.used_colors() {
            result.push(Card {
                number: Number::One,
                color,
            });
            result.push(Card {
                number: Number::Two,
                color,
            });
            result.push(Card {
                number: Number::Three,
                color,
            });
            result.push(Card {
                number: Number::Four,
                color,
            });
            result.push(Card {
                number: Number::Five,
                color,
            });
        }

        result
    }
}

#[derive(Debug)]
pub enum RuleViolation {
    InvalidCardPosition,
    InvalidReceiver,
    IncorrectHint,
    NullHint,
    NoMoreHintsAvailable,
    NoMoreDiscardsAvailable,
    InvalidCardInformation,
}

impl Rules {
    pub fn used_colors(&self) -> Vec<Color> {
        vec![
            Color::Red,
            Color::Blue,
            Color::Green,
            Color::Yellow,
            Color::White,
        ]
    }

    fn all_cards(&self) -> Vec<Card> {
        let colors = self.used_colors();

        let mut result = Vec::with_capacity(colors.len() * 6);

        for color in colors {
            result.push(Card {
                number: Number::One,
                color,
            });
            result.push(Card {
                number: Number::One,
                color,
            });
            result.push(Card {
                number: Number::One,
                color,
            });
            result.push(Card {
                number: Number::Two,
                color,
            });
            result.push(Card {
                number: Number::Two,
                color,
            });
            result.push(Card {
                number: Number::Three,
                color,
            });
            result.push(Card {
                number: Number::Three,
                color,
            });
            result.push(Card {
                number: Number::Four,
                color,
            });
            result.push(Card {
                number: Number::Four,
                color,
            });
            result.push(Card {
                number: Number::Five,
                color,
            });
        }

        result
    }

    pub(crate) fn new() -> Rules {
        Rules {
            number_of_players: 4,
            hand_size: 4,
            max_clues: 8,
        }
    }
}

pub struct Firework {
    piles: IndexMap<Color, Option<Number>>,
}

impl Firework {
    fn new(used_colors: &[Color]) -> Self {
        let mut piles = IndexMap::with_capacity(used_colors.len());
        for &color in used_colors {
            piles.insert(color, None);
        }
        Self { piles }
    }

    fn is_complete(&self) -> bool {
        self.piles.values().all(|n| n == &Some(Number::Five))
    }

    fn score(&self) -> usize {
        self.piles
            .values()
            .map(|mn| mn.map_or(0, |n| n.score()))
            .sum()
    }

    fn add(&mut self, card: Card) -> bool {
        let current = &mut self.piles[&card.color];

        if card.number.comes_after(*current) {
            *current = Some(card.number);
            true
        } else {
            false
        }
    }

    pub(crate) fn currently_playable(&self) -> PossibleCards {
        let mut hashed = IndexSet::with_capacity(self.piles.len());

        for (&color, &number) in &self.piles {
            if let Some(card) = Card::next(color, number) {
                hashed.insert(card);
            }
        }

        PossibleCards { hashed }
    }

    fn is_playable(&self, card: &Card) -> bool {
        self.currently_playable().contains(card)
    }

    fn already_played(&self) -> PossibleCards {
        let mut result = PossibleCards::none();

        for (&color, &(mut maybe_number)) in &self.piles {
            while let Some(number) = maybe_number {
                result.add(Card { number, color });
                maybe_number = number.decrease();
            }
        }

        result
    }
}

pub fn play_game(
    rules: Rules,
    rng: &mut ChaCha20Rng,
    mut players: Vec<Box<dyn Player>>,
) -> Option<usize> {
    assert_eq!(rules.number_of_players, players.len());

    let mut state = State::new(&rules, rng);

    for p_id in 0..players.len() {
        for _ in 0..rules.hand_size {
            let card = state.draw().unwrap();
            for (pp_id, pplayer) in players.iter_mut().enumerate() {
                pplayer.witness_draw(p_id, (p_id != pp_id).then_some(card));
            }
        }
        state.go_to_next_player();
    }

    loop {
        if let Some(score) = state.is_concluded() {
            return score;
        }

        let mut action = players[state.active_player_id].request_action();

        println!("Requested action by {}: {}", state.active_player_id, action);

        let (old, new) = state.apply_action(action, &rules).unwrap();

        if let Some(old) = old {
            action.add_card_information(old);
        }

        for player in &mut players {
            player.witness_action(action, state.active_player_id);
        }

        if let Some(new) = new {
            for (pp_id, pplayer) in players.iter_mut().enumerate() {
                pplayer.witness_draw(
                    state.active_player_id,
                    (state.active_player_id != pp_id).then_some(new),
                );
            }
        }

        state.go_to_next_player();
    }
}
