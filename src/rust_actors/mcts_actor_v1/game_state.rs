use crate::rust_actors::shared::PlayerState;
use crate::shared::data::Move;

#[derive(Clone)]
pub struct GameState {
    pub current_moves: Vec<Move>,
    pub current_pidx: usize,
    pub player_states: [PlayerState; 4],
}
