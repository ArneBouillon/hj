use core::result::Result;
use core::result::Result::Ok;
use crate::internal::game_state::GameState;
use crate::internal::validate;
use crate::shared::data::{Card, PassDirection};
use crate::shared::errors::HJError;

pub fn pass(direction: PassDirection, game_state: &mut GameState) -> Result<(), HJError> {
    let passed_cards: [Vec<Card>; 4] = [0, 1, 2, 3].map(|pidx|
        game_state.players_mut()[pidx].actor_mut().get_pass(direction)
    );

    for pidx in 0..4 { validate::validate_pass(direction, &passed_cards[pidx], pidx, game_state)?; }
    game_state.did_pass(direction, &passed_cards);

    Ok(())
}
