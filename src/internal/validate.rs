use core::option::Option::Some;
use core::result::Result;
use core::result::Result::{Err, Ok};
use crate::internal::game_state::GameState;
use crate::shared::data::{Move, Suit};
use crate::shared::errors::HJError;

pub fn validate_move(game_state: &GameState, played_moves: &Vec<Move>, new_move: Move) -> Result<(), HJError> {
    let Move(new_pidx, new_card) = new_move;
    let current_hand = game_state.players()[new_pidx].hand();

    if !game_state.players()[new_pidx].hand().any_match(new_card) {
        return Err(HJError::InvalidCardError("A player can only play one of their cards.".to_owned()))
    }

    if let Some(Move(_, first_card)) = played_moves.first() {
        if first_card.suit() != new_card.suit() && current_hand.any_of_suit(first_card.suit()) {
            return Err(HJError::InvalidCardError("When possible, the suit of the first card should be followed.".to_owned()))
        }
    } else {
        if new_card.suit() == Suit::Hearts
            && !game_state.hearts_played()
            && !current_hand.all_hearts() {

            return Err(HJError::InvalidCardError("When possible, the first card should not be hearts if no hearts have been played yet.".to_owned()));
        }
    }

    if game_state.current_round() == 0
        && new_card.score() != 0
        && !current_hand.all_have_score() {

        return Err(HJError::InvalidCardError("When possible, cards in the first round should not carry points.".to_owned()))
    }

    Ok(())
}
