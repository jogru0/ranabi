use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;

use ranabi::{
    player::{basic::BasicPlayer, Player},
    state::{play_game, Rules},
};

fn main() {
    println!("Hello, world!");

    let rules = Rules::new();

    let players = (0..rules.number_of_players)
        .map(|id| Box::new(BasicPlayer::new(rules, id)) as Box<dyn Player>)
        .collect();

    let conclusion = play_game(Rules::new(), &mut ChaCha20Rng::seed_from_u64(666), players);
    if let Some(score) = conclusion {
        println!("Won with score {}/{}.", score, rules.max_score());
    } else {
        println!("Lost.")
    }
}
