use ranabi::state::{record_game, Rules};
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;

#[ignore = "release only"]
#[test]
fn stats() {
    let mut rng = ChaCha20Rng::seed_from_u64(42069);
    let rules = Rules::new();

    let iterations = 4269;

    let mut sum = 0;

    for i in 0..iterations {
        if i % 100 == 0 {
            println!("Iteration {i}.");
        }

        let deck = rules.get_shuffled_deck(&mut rng);
        let players = rules.get_basic_player();
        let (score, _) = record_game(rules, deck, players);

        sum += score.unwrap_or_default();
    }

    let average = sum as f64 / iterations as f64;

    assert_eq!(average, 18.85687514640431);
}
