mod api;
mod internal;
mod rust_actors;
mod shared;
mod util;

use std::thread;
use std::thread::JoinHandle;

use iter_fixed::IntoIteratorFixed;

use crate::api::json::JSONActor;
use crate::rust_actors::human_actor::HumanActor;
use crate::rust_actors::mcts_actor_v1::MCTSActorV1;
use crate::rust_actors::random_actor::RandomActor;
use crate::rust_actors::rule_actor_v1::RuleActorV1;
use crate::shared::data::{Card, PassDirection, Rank, Suit};

fn main() {
    // let mut actor = RuleActorV1::new();
    // actor.initialize(0, &vec![
    //     Card(Rank::Three, Suit::Spades),
    //     Card(Rank::Seven, Suit::Spades),
    //     Card(Rank::Ace, Suit::Spades),
    //     Card(Rank::Two, Suit::Clubs),
    //     Card(Rank::Three, Suit::Clubs),
    //     Card(Rank::Four, Suit::Clubs),
    //     Card(Rank::Five, Suit::Clubs),
    //     Card(Rank::Six, Suit::Clubs),
    //     Card(Rank::Eight, Suit::Clubs),
    //     Card(Rank::Ten, Suit::Clubs),
    //     Card(Rank::Five, Suit::Diamonds),
    //     Card(Rank::Ace, Suit::Diamonds),
    //     Card(Rank::Jack, Suit::Hearts),
    // ]);
    // println!("{:?}", actor.choose_three_to_pass());

    let mut total_scores = [0, 0, 0, 0];

    for round_num in (0..1000).step_by(1) {
        let base_hands = util::deck::get_shuffled_hands();

        let handles: Vec<_> = (0..1).map(|player_shift| {
            let hands = [0, 1, 2, 3].into_iter_fixed().map(|i| base_hands[(player_shift + i) % 4].clone()).collect();
            thread::spawn(move || {
                internal::game::play_with_hands(
                    hands,
                    PassDirection::from_round(round_num),
                    [
                        &mut MCTSActorV1::<RuleActorV1>::new(50, 40),
                        &mut RuleActorV1::new(),
                        &mut RuleActorV1::new(),
                        &mut RuleActorV1::new(),
                    ],
                ).expect("Errors should not occur.")
            })
        }).collect();

        for (inc, handle) in handles.into_iter().enumerate() {
            let scores = handle.join().unwrap();
            total_scores[0] += scores[0];
            total_scores[1] += scores[1];
            total_scores[2] += scores[2];
            total_scores[3] += scores[3];

            println!("  {}: {:?}      (cumulative: {:?})", round_num, scores, total_scores);
        }

        // let scores = internal::game::play(PassDirection::from_round(round_num), [
        //     &mut MCTSActorV1::<RuleActorV1>::new(50, 10),
        //     &mut RuleActorV1::new(),
        //     &mut RuleActorV1::new(),
        //     &mut RuleActorV1::new(),
        // ]).expect("Errors should not occur.");
        // total_scores[0] += scores[0];
        // total_scores[1] += scores[1];
        // total_scores[2] += scores[2];
        // total_scores[3] += scores[3];
        //
        // println!("  {}: {:?}      (cumulative: {:?})", round_num, scores, total_scores);
    }
    println!("{:?}", total_scores);
}
