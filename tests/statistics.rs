use std::io::Write;
use std::{fs::File, time::Duration};

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
    let mut specific_score = [0; 26];

    let mut total_decision_time = Duration::ZERO;
    let mut total_decisions = 0;

    let mut failed_deck = None;

    for i in 0..iterations {
        if i % 100 == 0 {
            println!("Iteration {i}.");
        }

        let deck = rules.get_shuffled_deck(&mut rng);
        let players = rules.get_basic_player();

        let (score, _, (additional_decision_time, additional_decisions)) =
            record_game(rules, deck.clone(), players);

        sum += score.unwrap_or_default();
        if let Some(score) = score {
            specific_score[score] += 1;
        } else {
            if failed_deck.is_none() {
                failed_deck = Some(deck);
            }
            failed += 1;
        }

        total_decision_time += additional_decision_time;
        total_decisions += additional_decisions;
    }

    let average = sum as f64 / iterations as f64;

    let average_time_per_decision = total_decision_time / total_decisions.try_into().unwrap();

    let mut file = File::create("res/regression/stats.txt").unwrap();
    writeln!(&mut file, "Average: {average}\n").unwrap();

    let mut acc = 0;
    for number in specific_score.iter_mut().rev() {
        acc += *number;
        *number = acc;
    }

    assert_eq!(iterations, specific_score[0] + failed);

    for (score, number) in specific_score.into_iter().enumerate() {
        writeln!(
            &mut file,
            "At least {}: {:.2}%",
            score,
            100. / iterations as f64 * number as f64
        )
        .unwrap();
    }

    writeln!(
        &mut file,
        "\nAverage time per decision: {} µs",
        average_time_per_decision.as_secs_f64() * 1000. * 1000.
    )
    .unwrap();
    if let Some(deck) = failed_deck {
        writeln!(&mut file, "\n{}", deck.to_line()).unwrap();
    }
}
