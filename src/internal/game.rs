use core::result::Result;
use core::result::Result::Ok;
use crate::shared::actor::Actor;
use crate::shared::data::{Card, Rank, Suit};
use crate::shared::errors::HJError;
use super::game_state::GameState;
use super::round;

pub fn play(actors: [&mut dyn Actor; 4]) -> Result<[isize; 4], HJError> {
    let mut game_state = GameState::new(actors);
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
