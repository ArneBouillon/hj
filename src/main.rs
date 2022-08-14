mod api;
mod internal;
mod rust_actors;
mod shared;

use crate::api::json::JSONActor;
use crate::rust_actors::human_actor::HumanActor;
use crate::rust_actors::random_actor::RandomActor;

fn main() {
    let scores = internal::game::play([
        &mut HumanActor::new(),
        &mut RandomActor::new(),
        &mut RandomActor::new(),
        &mut RandomActor::new(),
    ]).expect("Errors should not occur.");
    println!("{:?}", scores);
}
