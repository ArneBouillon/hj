use crate::Card;
use crate::rust_actors::player_state::DefaultPlayerStateInterface;
use crate::util::non_nan::NonNan;

pub mod eval_state_v1;

pub trait EvalState {
    fn evaluate_state<
        PS: DefaultPlayerStateInterface
    >(player_state: &PS, by_suit: &[Vec<Card>; 4]) -> NonNan;
}
