use core::result::Result;
use core::result::Result::Ok;
use crate::shared::actor::Actor;
use crate::shared::data::{Card, Hand, PassDirection, Rank, Suit};
use crate::shared::errors::HJError;
use super::game_state::GameState;
use super::pass;
use super::round;

fn _play(pass_direction: PassDirection, mut game_state: GameState) -> Result<[isize; 4], HJError> {
    pass::pass(pass_direction, &mut game_state)?;

    let mut first_pidx = (0..4).into_iter()
        .find(|pidx| game_state.players()[*pidx].hand().any_match(Card(Rank::Two, Suit::Clubs)))
        .expect("Someone should have the two of clubs.");

    for _ in 0..13 {
        let (winner_pidx, played_moves) = round::play(first_pidx, &mut game_state)?;
        game_state.did_play_round(winner_pidx, played_moves);
        first_pidx = winner_pidx;
    }

    game_state.did_play_game();
    Ok(game_state.score())
}

pub fn play_with_hands(hands: [Hand; 4], pass_direction: PassDirection, actors: [&mut dyn Actor; 4]) -> Result<[isize; 4], HJError> {
    let game_state = GameState::new_from_hands(hands, actors);
    _play(pass_direction, game_state)
}

#[allow(dead_code)]
pub fn play(pass_direction: PassDirection, actors: [&mut dyn Actor; 4]) -> Result<[isize; 4], HJError> {
    let game_state = GameState::new(actors);
    _play(pass_direction, game_state)
}
