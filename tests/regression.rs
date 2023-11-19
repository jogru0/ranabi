use std::fs::File;
use std::io::Write;

use ranabi::{
    card::{Card, Color, Number},
    state::{record_game, Deck, Rules},
};
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;

fn regression_test(rules: Rules, deck: Deck, expected: Option<usize>, name: &str) {
    let players = rules.get_basic_player();

    let (score, record) = record_game(rules, deck, players);

    let mut file = File::create(format!("res/regression/{name}.txt")).unwrap();
    writeln!(&mut file, "{record}").unwrap();

    assert_eq!(score, expected);
}

#[test]
fn devil_game() {
    let mut rng = ChaCha20Rng::seed_from_u64(666);
    let rules = Rules::new();
    let deck = rules.get_shuffled_deck(&mut rng);
    regression_test(rules, deck, Some(22), "devil");
}

#[test]
fn failed_0() {
    let deck = Deck::new(vec![
        Card {
            number: Number::Three,
            color: Color::Blue,
        },
        Card {
            number: Number::Three,
            color: Color::Blue,
        },
        Card {
            number: Number::One,
            color: Color::Blue,
        },
        Card {
            number: Number::Five,
            color: Color::Yellow,
        },
        Card {
            number: Number::Two,
            color: Color::Green,
        },
        Card {
            number: Number::Four,
            color: Color::Red,
        },
        Card {
            number: Number::Five,
            color: Color::Green,
        },
        Card {
            number: Number::Two,
            color: Color::White,
        },
        Card {
            number: Number::Two,
            color: Color::Yellow,
        },
        Card {
            number: Number::Three,
            color: Color::White,
        },
        Card {
            number: Number::One,
            color: Color::Yellow,
        },
        Card {
            number: Number::Two,
            color: Color::White,
        },
        Card {
            number: Number::One,
            color: Color::Red,
        },
        Card {
            number: Number::Four,
            color: Color::Yellow,
        },
        Card {
            number: Number::One,
            color: Color::Green,
        },
        Card {
            number: Number::Two,
            color: Color::Red,
        },
        Card {
            number: Number::One,
            color: Color::Blue,
        },
        Card {
            number: Number::One,
            color: Color::White,
        },
        Card {
            number: Number::One,
            color: Color::Yellow,
        },
        Card {
            number: Number::Five,
            color: Color::White,
        },
        Card {
            number: Number::Three,
            color: Color::Green,
        },
        Card {
            number: Number::Two,
            color: Color::Blue,
        },
        Card {
            number: Number::Four,
            color: Color::Blue,
        },
        Card {
            number: Number::One,
            color: Color::Blue,
        },
        Card {
            number: Number::Two,
            color: Color::Yellow,
        },
        Card {
            number: Number::Two,
            color: Color::Green,
        },
        Card {
            number: Number::Four,
            color: Color::Blue,
        },
        Card {
            number: Number::One,
            color: Color::Red,
        },
        Card {
            number: Number::Four,
            color: Color::White,
        },
        Card {
            number: Number::Three,
            color: Color::Red,
        },
        Card {
            number: Number::Four,
            color: Color::White,
        },
        Card {
            number: Number::Two,
            color: Color::Red,
        },
        Card {
            number: Number::Three,
            color: Color::Yellow,
        },
        Card {
            number: Number::Three,
            color: Color::White,
        },
        Card {
            number: Number::Four,
            color: Color::Red,
        },
        Card {
            number: Number::One,
            color: Color::White,
        },
        Card {
            number: Number::One,
            color: Color::Green,
        },
        Card {
            number: Number::Five,
            color: Color::Red,
        },
        Card {
            number: Number::Three,
            color: Color::Green,
        },
        Card {
            number: Number::Three,
            color: Color::Red,
        },
        Card {
            number: Number::Five,
            color: Color::Blue,
        },
        Card {
            number: Number::Two,
            color: Color::Blue,
        },
        Card {
            number: Number::Three,
            color: Color::Yellow,
        },
        Card {
            number: Number::One,
            color: Color::Green,
        },
        Card {
            number: Number::Four,
            color: Color::Green,
        },
        Card {
            number: Number::One,
            color: Color::White,
        },
        Card {
            number: Number::Four,
            color: Color::Yellow,
        },
        Card {
            number: Number::Four,
            color: Color::Green,
        },
        Card {
            number: Number::One,
            color: Color::Red,
        },
        Card {
            number: Number::One,
            color: Color::Yellow,
        },
    ]);
    regression_test(Rules::new(), deck, Some(20), "failed_0");
}

#[test]
fn failed_1() {
    let deck = Deck::new(vec![
        Card {
            number: Number::Three,
            color: Color::Blue,
        },
        Card {
            number: Number::Two,
            color: Color::Red,
        },
        Card {
            number: Number::Two,
            color: Color::Yellow,
        },
        Card {
            number: Number::Five,
            color: Color::Green,
        },
        Card {
            number: Number::Three,
            color: Color::White,
        },
        Card {
            number: Number::Five,
            color: Color::Blue,
        },
        Card {
            number: Number::One,
            color: Color::White,
        },
        Card {
            number: Number::Four,
            color: Color::Red,
        },
        Card {
            number: Number::One,
            color: Color::White,
        },
        Card {
            number: Number::Five,
            color: Color::Yellow,
        },
        Card {
            number: Number::Four,
            color: Color::Yellow,
        },
        Card {
            number: Number::Four,
            color: Color::Red,
        },
        Card {
            number: Number::Three,
            color: Color::Red,
        },
        Card {
            number: Number::One,
            color: Color::Red,
        },
        Card {
            number: Number::Three,
            color: Color::Yellow,
        },
        Card {
            number: Number::One,
            color: Color::Yellow,
        },
        Card {
            number: Number::Three,
            color: Color::Red,
        },
        Card {
            number: Number::Three,
            color: Color::Green,
        },
        Card {
            number: Number::Two,
            color: Color::Green,
        },
        Card {
            number: Number::One,
            color: Color::Red,
        },
        Card {
            number: Number::Two,
            color: Color::White,
        },
        Card {
            number: Number::Four,
            color: Color::White,
        },
        Card {
            number: Number::Four,
            color: Color::Blue,
        },
        Card {
            number: Number::Two,
            color: Color::Blue,
        },
        Card {
            number: Number::Three,
            color: Color::Blue,
        },
        Card {
            number: Number::Two,
            color: Color::Blue,
        },
        Card {
            number: Number::Three,
            color: Color::Yellow,
        },
        Card {
            number: Number::Four,
            color: Color::Green,
        },
        Card {
            number: Number::Three,
            color: Color::Green,
        },
        Card {
            number: Number::One,
            color: Color::Blue,
        },
        Card {
            number: Number::One,
            color: Color::Red,
        },
        Card {
            number: Number::Four,
            color: Color::Green,
        },
        Card {
            number: Number::One,
            color: Color::White,
        },
        Card {
            number: Number::One,
            color: Color::Green,
        },
        Card {
            number: Number::Four,
            color: Color::Blue,
        },
        Card {
            number: Number::One,
            color: Color::Blue,
        },
        Card {
            number: Number::Two,
            color: Color::Green,
        },
        Card {
            number: Number::Four,
            color: Color::Yellow,
        },
        Card {
            number: Number::Five,
            color: Color::White,
        },
        Card {
            number: Number::Five,
            color: Color::Red,
        },
        Card {
            number: Number::Two,
            color: Color::White,
        },
        Card {
            number: Number::One,
            color: Color::Yellow,
        },
        Card {
            number: Number::Two,
            color: Color::Red,
        },
        Card {
            number: Number::Four,
            color: Color::White,
        },
        Card {
            number: Number::One,
            color: Color::Green,
        },
        Card {
            number: Number::One,
            color: Color::Blue,
        },
        Card {
            number: Number::One,
            color: Color::Green,
        },
        Card {
            number: Number::Two,
            color: Color::Yellow,
        },
        Card {
            number: Number::One,
            color: Color::Yellow,
        },
        Card {
            number: Number::Three,
            color: Color::White,
        },
    ]);
    regression_test(Rules::new(), deck, Some(23), "failed_1");
}

#[test]
fn failed_2() {
    let deck = Deck::new(vec![
        Card {
            number: Number::Four,
            color: Color::Red,
        },
        Card {
            number: Number::One,
            color: Color::Red,
        },
        Card {
            number: Number::One,
            color: Color::Green,
        },
        Card {
            number: Number::Two,
            color: Color::Yellow,
        },
        Card {
            number: Number::Four,
            color: Color::Blue,
        },
        Card {
            number: Number::One,
            color: Color::Blue,
        },
        Card {
            number: Number::Three,
            color: Color::Red,
        },
        Card {
            number: Number::Five,
            color: Color::White,
        },
        Card {
            number: Number::One,
            color: Color::Red,
        },
        Card {
            number: Number::Two,
            color: Color::Green,
        },
        Card {
            number: Number::Two,
            color: Color::Yellow,
        },
        Card {
            number: Number::One,
            color: Color::Green,
        },
        Card {
            number: Number::Three,
            color: Color::Yellow,
        },
        Card {
            number: Number::Four,
            color: Color::White,
        },
        Card {
            number: Number::Three,
            color: Color::Red,
        },
        Card {
            number: Number::Five,
            color: Color::Green,
        },
        Card {
            number: Number::One,
            color: Color::Red,
        },
        Card {
            number: Number::Two,
            color: Color::Blue,
        },
        Card {
            number: Number::Five,
            color: Color::Red,
        },
        Card {
            number: Number::Two,
            color: Color::White,
        },
        Card {
            number: Number::One,
            color: Color::Yellow,
        },
        Card {
            number: Number::One,
            color: Color::Blue,
        },
        Card {
            number: Number::Three,
            color: Color::White,
        },
        Card {
            number: Number::Three,
            color: Color::Green,
        },
        Card {
            number: Number::Five,
            color: Color::Blue,
        },
        Card {
            number: Number::Two,
            color: Color::Blue,
        },
        Card {
            number: Number::Three,
            color: Color::White,
        },
        Card {
            number: Number::Three,
            color: Color::Green,
        },
        Card {
            number: Number::One,
            color: Color::Yellow,
        },
        Card {
            number: Number::Two,
            color: Color::Green,
        },
        Card {
            number: Number::Five,
            color: Color::Yellow,
        },
        Card {
            number: Number::Two,
            color: Color::Red,
        },
        Card {
            number: Number::One,
            color: Color::Yellow,
        },
        Card {
            number: Number::One,
            color: Color::Blue,
        },
        Card {
            number: Number::Four,
            color: Color::Green,
        },
        Card {
            number: Number::One,
            color: Color::White,
        },
        Card {
            number: Number::One,
            color: Color::White,
        },
        Card {
            number: Number::Three,
            color: Color::Blue,
        },
        Card {
            number: Number::Four,
            color: Color::Red,
        },
        Card {
            number: Number::Four,
            color: Color::Yellow,
        },
        Card {
            number: Number::Three,
            color: Color::Blue,
        },
        Card {
            number: Number::One,
            color: Color::Green,
        },
        Card {
            number: Number::Four,
            color: Color::Blue,
        },
        Card {
            number: Number::Two,
            color: Color::Red,
        },
        Card {
            number: Number::Four,
            color: Color::White,
        },
        Card {
            number: Number::One,
            color: Color::White,
        },
        Card {
            number: Number::Four,
            color: Color::Green,
        },
        Card {
            number: Number::Two,
            color: Color::White,
        },
        Card {
            number: Number::Three,
            color: Color::Yellow,
        },
        Card {
            number: Number::Four,
            color: Color::Yellow,
        },
    ]);

    regression_test(Rules::new(), deck, None, "failed_2");
}
