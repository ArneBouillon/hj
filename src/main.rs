mod api;
mod game;
mod internal;
mod rust_actors;
mod util;

use std::thread;

use iter_fixed::IntoIteratorFixed;
use crate::api::json::JSONActor;

use crate::rust_actors::actor_mcts_v1::ActorMCTSV1;
use crate::rust_actors::actor_rule_v1::ActorRuleV1;
use crate::rust_actors::player_state::default_player_state::DefaultPlayerState;
use crate::rust_actors::player_state::extended_player_state::ExtendedPlayerState;
use crate::game::data::{Card, PassDirection, Rank, Suit};
use crate::rust_actors::actor_mcts_mod_v1::ActorMCTSModV1;
use crate::rust_actors::actor_random::ActorRandom;
use crate::rust_actors::determinize::determinize_v1::DeterminizeV1;
use crate::rust_actors::eval_round::eval_round_v1::EvalRoundV1;
use crate::rust_actors::eval_state::eval_state_v1::EvalStateV1;
use crate::rust_actors::player_state::basic_player_state::BasicPlayerState;

fn main() {
    let mut total_scores = [0, 0, 0, 0];

    for round_num in (0..100).step_by(1) {
        let base_hands = util::deck::get_shuffled_hands();

        let handles: Vec<_> = (0..1).map(|player_shift| {
            let hands = [0, 1, 2, 3].into_iter_fixed().map(|i| base_hands[(player_shift + i) % 4].clone()).collect();
            thread::spawn(move || {
                internal::game::play_with_hands(
                    hands,
                    PassDirection::from_round(round_num),
                    [
                        &mut ActorMCTSModV1::<DeterminizeV1, EvalStateV1, ActorRuleV1<EvalRoundV1, EvalStateV1, ExtendedPlayerState>, ExtendedPlayerState>::new(50),
                        &mut ActorRandom::<BasicPlayerState>::new(),
                        &mut ActorRandom::<BasicPlayerState>::new(),
                        &mut JSONActor::new("/home/arne/repos/hj/src/json_actors/random_actor.py".to_owned()),
                    ],
                ).expect("Errors should not occur.")
            })
        }).collect();

        for handle in handles {
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
