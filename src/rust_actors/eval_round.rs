use crate::Card;
use crate::game::data::Move;
use crate::rust_actors::player_state::ExtendedPlayerStateInterface;
use crate::util::non_nan::NonNan;

pub mod eval_round_v1;

pub trait EvalRound {
    fn evaluate_round<
        PS: ExtendedPlayerStateInterface
    >(player_state: &PS, played_moves: &Vec<Move>, new_card: Card) -> NonNan;
}
