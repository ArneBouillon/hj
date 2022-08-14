use crate::shared::data::Move;
use super::data::Card;

pub trait Actor {
    fn initialize(&mut self, pidx: usize, cards: &Vec<Card>);

    fn play_card(&mut self, played_moves: &Vec<Move>) -> Card;

    fn end_round(&mut self, winner_pidx: usize, played_moves: &Vec<Move>);

    fn end_game(&mut self, score: [isize; 4]);
}
