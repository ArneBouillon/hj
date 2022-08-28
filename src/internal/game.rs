use core::result::Result;
use core::result::Result::Ok;
use crate::game::actor::Actor;
use crate::game::data::{Card, Hand, PassDirection, Rank, Suit};
use crate::game::errors::HJError;
use crate::game::game_info::{GameInfo, StopCondition};
use crate::internal::hj_validator::HJValidator;

#[allow(dead_code)]
pub fn play_with_hands(hands: [Hand; 4], pass_direction: PassDirection, mut actors: [&mut dyn Actor; 4]) -> Result<[isize; 4], HJError> {
    actors[0].initialize(0, hands[0].cards());
    actors[1].initialize(1, hands[1].cards());
    actors[2].initialize(2, hands[2].cards());
    actors[3].initialize(3, hands[3].cards());

    let mut game_info = GameInfo::new(hands);

    game_info.pass::<HJValidator, dyn Actor>(pass_direction, &mut actors)?;
    game_info.play::<HJValidator, dyn Actor>(&mut actors, StopCondition::None)?;

    Ok(game_info.score())
}
