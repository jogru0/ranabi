use std::fmt::Display;

use indexmap::{IndexMap, IndexSet};
use rand::seq::SliceRandom;
use rand_chacha::ChaCha20Rng;

use crate::{
    card::{Card, Color, Number, PossibleCards},
    player::{action::Action, basic::BasicPlayer, Player},
};

pub struct DiscardPile {
    card_to_multiplicity: IndexMap<Card, usize>,
}

impl Display for DiscardPile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (card, &multiplicity) in &self.card_to_multiplicity {
            for _ in 0..multiplicity {
                write!(f, "{} ", card)?;
            }
        }

        Ok(())
    }
}

impl DiscardPile {
    fn contains(&self, card: &Card) -> bool {
        self.card_to_multiplicity[card] != 0
    }

    fn new(rules: &Rules) -> Self {
        DiscardPile {
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

    pub fn add(&mut self, card: &Card) {
        self.card_to_multiplicity[card] += 1;
    }
}

pub struct PublicState {
    pub firework: Firework,
    pub discard_pile: DiscardPile,
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
            if self.discard_pile.contains(&two) && maybe_number.is_none() {
                result.add(two);
            }

            let three = Card {
                color,
                number: Number::Three,
            };
            if self.discard_pile.contains(&three)
                && (maybe_number.is_none() || maybe_number == Some(Number::One))
            {
                result.add(three);
            }

            let four = Card {
                color,
                number: Number::Four,
            };
            if self.discard_pile.contains(&four)
                && (maybe_number.is_none()
                    || maybe_number == Some(Number::One)
                    || maybe_number == Some(Number::Two))
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

    pub(crate) fn discard(&mut self, card: Card) {
        self.discard_pile.add(&card);

        let succ = self.add_clue();
        assert!(succ);
    }

    pub(crate) fn play(&mut self, card: Card) {
        let succ = self.firework.add(card);
        if !succ {
            self.strikes += 1;
            self.discard_pile.add(&card);
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
            discard_pile: DiscardPile::new(&rules),
            rules,
            clues: rules.max_clues,
            strikes: 0,
        }
    }

    pub(crate) fn definite_trash(&self) -> PossibleCards {
        let mut result = self.firework.already_played();
        result.merge(&self.discard_pile.unreachable(&self.rules));
        result
    }
}

struct State {
    deck: Deck,
    active_player_id: usize,
    remaining_hints: usize,
    strikes: usize,
    firework: Firework,
    number_of_players: usize,
    number_of_actions_with_empty_deck: usize,
    hands: Vec<Vec<Card>>,
    discard: DiscardPile,
}

impl Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn print_player(
            state: &State,
            id: usize,
            f: &mut std::fmt::Formatter<'_>,
        ) -> std::fmt::Result {
            let prefix = if id == state.active_player_id {
                '>'
            } else {
                ' '
            };
            write!(f, "{prefix} ")?;
            for card in &state.hands[id] {
                write!(f, "{card} ")?;
            }

            Ok(())
        }

        print_player(self, 0, f)?;
        writeln!(f, "    {}", self.firework)?;
        print_player(self, 1, f)?;
        writeln!(f, "    {}     {}", self.remaining_hints, self.strikes)?;
        print_player(self, 2, f)?;
        writeln!(f, "    {}", self.discard)?;
        for i in 3..self.number_of_players {
            print_player(self, i, f)?;
            writeln!(f)?;
        }

        Ok(())
    }
}

impl State {
    fn new(rules: &Rules, deck: Deck) -> Self {
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
            discard: DiscardPile::new(rules),
        }
    }

    fn draw(&mut self) -> Option<Card> {
        let new = self.deck.draw()?;
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
                    self.discard.add(&card);
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
                self.discard.add(&card);

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
    pub fn allow_null_hints(&self) -> bool {
        false
    }

    pub fn get_shuffled_deck(&self, rng: &mut ChaCha20Rng) -> Deck {
        let mut deck = self.all_cards();
        deck.shuffle(rng);
        Deck::new(deck)
    }

    pub fn get_basic_player(&self) -> Vec<Box<dyn Player>> {
        (0..self.number_of_players)
            .map(|id| Box::new(BasicPlayer::new(*self, id)) as Box<dyn Player>)
            .collect()
    }

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

    pub fn new() -> Rules {
        Rules {
            number_of_players: 4,
            hand_size: 4,
            max_clues: 8,
        }
    }
}

impl Default for Rules {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Firework {
    piles: IndexMap<Color, Option<Number>>,
}

impl Display for Firework {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (&color, &maybe_number) in &self.piles {
            match maybe_number {
                Some(number) => write!(f, "{} ", Card { number, color })?,
                None => write!(f, "   ")?,
            }
        }

        Ok(())
    }
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

    pub fn already_played(&self) -> PossibleCards {
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

pub struct Record {
    rules: Rules,
    deck: Deck,
    actions: Vec<Action>,
}

impl Display for Record {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut state = State::new(&self.rules, self.deck.clone());

        for _ in 0..self.rules.number_of_players {
            for _ in 0..self.rules.hand_size {
                state.draw().unwrap();
            }
            state.go_to_next_player();
        }

        let mut turn = 1;

        for &action in &self.actions {
            assert!(state.is_concluded().is_none());

            writeln!(f, "\n==============\n")?;
            writeln!(f, "{state}")?;

            writeln!(f, "Turn {} action: {}", turn, action)?;

            let (_old, _new) = state.apply_action(action, &self.rules).unwrap();

            if let Some(_old) = _old {
                // action.add_card_information(old);
            }

            state.go_to_next_player();
            turn += 1;
        }

        match state.is_concluded() {
            Some(Some(score)) => writeln!(f, "Won with {score} points."),
            Some(None) => writeln!(f, "Lost."),
            None => panic!(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Deck {
    cards: Vec<Card>,
}
impl Deck {
    fn draw(&mut self) -> Option<Card> {
        self.cards.pop()
    }

    fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    fn new(cards: Vec<Card>) -> Self {
        Self { cards }
    }
}

pub fn record_game(
    rules: Rules,
    deck: Deck,
    mut players: Vec<Box<dyn Player>>,
) -> (Option<usize>, Record) {
    assert_eq!(rules.number_of_players, players.len());

    let mut state = State::new(&rules, deck.clone());

    let mut record = Vec::new();

    for p_id in 0..players.len() {
        for _ in 0..rules.hand_size {
            let card = state.draw().unwrap();
            for (pp_id, pplayer) in players.iter_mut().enumerate() {
                pplayer.witness_draw(p_id, (p_id != pp_id).then_some(card));
            }
        }
        state.go_to_next_player();
    }

    let mut turn = 1;

    loop {
        if let Some(score) = state.is_concluded() {
            return (
                score,
                Record {
                    rules,
                    deck,
                    actions: record,
                },
            );
        }

        println!("\n==============\n");
        println!("{state}");

        let mut action = players[state.active_player_id].request_action();

        println!("Turn {} action: {}", turn, action);

        let (old, new) = state.apply_action(action, &rules).unwrap();

        record.push(action);

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
        turn += 1;
    }
}

pub fn play_game(
    rules: Rules,
    rng: &mut ChaCha20Rng,
    players: Vec<Box<dyn Player>>,
) -> Option<usize> {
    let deck = rules.get_shuffled_deck(rng);
    record_game(rules, deck, players).0
}
