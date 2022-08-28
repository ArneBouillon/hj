use crate::game::data::Move;
use crate::game::errors::HJError;
use crate::{Card, PassDirection};
use crate::game::game_info::GameInfo;

pub trait Validator {
    fn validate_pass(direction: PassDirection, passed_cards: &Vec<Card>, pidx: usize, game_info: &GameInfo) -> Result<(), HJError>;
    fn validate_move(game_info: &GameInfo, played_moves: &Vec<Move>, new_move: Move) -> Result<(), HJError>;
}

pub struct NoValidator {}
impl Validator for NoValidator {
    fn validate_pass(_direction: PassDirection, _passed_cards: &Vec<Card>, _pidx: usize, _game_info: &GameInfo) -> Result<(), HJError> {
        Ok(())
    }

    fn validate_move(_game_info: &GameInfo, _played_moves: &Vec<Move>, _new_move: Move) -> Result<(), HJError> {
        Ok(())
    }
}
