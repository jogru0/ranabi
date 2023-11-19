use std::fs::File;
use std::io::Write;

use ranabi::state::{record_game, Rules};
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;

#[test]
fn devil_game() {
    let mut rng = ChaCha20Rng::seed_from_u64(666);
    let rules = Rules::new();
    let deck = rules.get_shuffled_deck(&mut rng);
    let players = rules.get_basic_player();

    let (score, record) = record_game(rules, deck, players);

    let mut file = File::create("res/regression/devil.txt").unwrap();
    writeln!(&mut file, "{record}").unwrap();

    assert_eq!(score, Some(21));
}
