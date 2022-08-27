use crate::rust_actors::player_state::DefaultPlayerStateInterface;
use crate::shared::data::Move;

#[derive(Clone)]
pub struct GameState<PlayerState : DefaultPlayerStateInterface> {
    pub current_moves: Vec<Move>,
    pub current_pidx: usize,
    pub player_states: [PlayerState; 4],
}
