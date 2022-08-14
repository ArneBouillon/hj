use core::result::Result;
use core::result::Result::Ok;
use crate::internal::validate::validate_move;
use crate::shared::data::Move;
use crate::shared::errors::HJError;
use super::game_state::GameState;

pub fn play(first_pidx: usize, game_state: &mut GameState) -> Result<(usize, Vec<Move>), HJError> {
    let mut played_moves = vec![];
    for counter in first_pidx..first_pidx + 4 {
        let pidx = counter % 4;
        let new_move = Move(
            pidx,
            game_state.players_mut()[pidx].actor_mut().play_card(&played_moves),
        );

        validate_move(game_state, &played_moves, new_move)?;
        game_state.did_play_move(new_move);
        played_moves.push(new_move);
    }

    Ok((find_winner_pidx(&played_moves), played_moves))
}

fn find_winner_pidx(played_moves: &Vec<Move>) -> usize {
    let first_move = played_moves.first().expect("There should have been 4 cards played.");
    played_moves.iter().skip(1).fold(first_move, |best_move, current_move| {
        if current_move.card().suit() == best_move.card().suit()
            && current_move.card().rank() > best_move.card().rank() {

            current_move
        } else {
            best_move
        }
    }).pidx()
}
