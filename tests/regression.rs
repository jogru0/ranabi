use std::fs::File;
use std::io::Write;

use ranabi::state::{deck::Deck, record_game, Rules};

fn regression_test(rules: Rules, deck: Deck, expected: Option<usize>, name: &str) {
    let players = rules.get_basic_player();

    let (score, record, _) = record_game(rules, deck.clone(), players);

    let mut file = File::create(format!("res/regression/{name}.txt")).unwrap();
    writeln!(&mut file, "{record}").unwrap();
    writeln!(&mut file, "{}", deck.to_line()).unwrap();

    assert_eq!(score, expected);
}

#[test]
fn devil_game() {
    let deck = Deck::from_line("y3y3r3b2b1w2r2w4b4y1b2r4g3w5b1y4w1w3b3y4g3w3r1r3y1g2b5b4g4w1y2g1g1g5b1g4b3r5y2w4r4g1w1r1y5r1w2y1r2g2");
    regression_test(Rules::new(), deck, Some(22), "devil");
}

#[test]
fn failed_0() {
    let deck = Deck::from_line("b3b3b1y5g2r4g5w2y2w3y1w2r1y4g1r2b1w1y1w5g3b2b4b1y2g2b4r1w4r3w4r2y3w3r4w1g1r5g3r3b5b2y3g1g4w1y4g4r1y1");
    regression_test(Rules::new(), deck, Some(20), "failed_0");
}

#[test]
fn failed_1() {
    let deck = Deck::from_line("b3r2y2g5w3b5w1r4w1y5y4r4r3r1y3y1r3g3g2r1w2w4b4b2b3b2y3g4g3b1r1g4w1g1b4b1g2y4w5r5w2y1r2w4g1b1g1y2y1w3");
    regression_test(Rules::new(), deck, Some(23), "failed_1");
}

#[test]
fn failed_2() {
    let deck = Deck::from_line("r4r1g1y2b4b1r3w5r1g2y2g1y3w4r3g5r1b2r5w2y1b1w3g3b5b2w3g3y1g2y5r2y1b1g4w1w1b3r4y4b3g1b4r2w4w1g4w2y3y4");
    regression_test(Rules::new(), deck, None, "failed_2");
}
