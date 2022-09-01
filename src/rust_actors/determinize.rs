use crate::DefaultPlayerState;
use crate::game::data::Move;
use crate::game::game_info::GameInfo;
use crate::rust_actors::player_state::DefaultPlayerStateInterface;

pub mod determinize_v1;

pub trait Determinize<
    PS: DefaultPlayerStateInterface
> {
    fn determinize(pidx: usize, player_state: &PS, played_moves: &Vec<Move>) -> (GameInfo, [DefaultPlayerState; 4]);
}
