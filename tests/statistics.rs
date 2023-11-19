use std::io::Write;
use std::{
    fs::File,
    time::{Duration, Instant},
};

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
    let mut failed = 0;
    let mut full_wins = 0;

    let mut time = Duration::ZERO;

    let mut failed_deck = None;

    for i in 0..iterations {
        if i % 100 == 0 {
            println!("Iteration {i}.");
        }

        let deck = rules.get_shuffled_deck(&mut rng);
        let players = rules.get_basic_player();

        let before = Instant::now();
        let (score, _) = record_game(rules, deck.clone(), players);
        let after = Instant::now();

        sum += score.unwrap_or_default();
        if score.is_none() {
            if failed_deck.is_none() {
                failed_deck = Some(deck);
            }
            failed += 1;
        }
        if score == Some(25) {
            full_wins += 1;
        }
        time += after - before;
    }

    let average = sum as f64 / iterations as f64;

    let average_time = time / iterations;

    let mut file = File::create("res/regression/stats.txt").unwrap();
    writeln!(&mut file, "Average: {average}").unwrap();
    writeln!(&mut file, "Failed: {failed} / {iterations}").unwrap();
    writeln!(&mut file, "Full wins: {full_wins}").unwrap();
    writeln!(
        &mut file,
        "Average time: {} ms",
        average_time.as_secs_f64() * 1000.
    )
    .unwrap();
    if let Some(deck) = failed_deck {
        writeln!(&mut file, "\n{}", deck.to_line()).unwrap();
    }
}
