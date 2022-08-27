use crate::shared::data::{Move, PassDirection};
use super::data::Card;

pub trait Actor {
    fn initialize(&mut self, pidx: usize, cards: &Vec<Card>);

    fn play_card(&mut self, played_moves: &Vec<Move>) -> Card;

    fn end_round(&mut self, winner_pidx: usize, played_moves: &Vec<Move>);

    fn end_game(&mut self, score: [isize; 4]);

    fn get_pass(&mut self, direction: PassDirection) -> Vec<Card>;

    fn end_pass(&mut self, passed_cards: &Vec<Card>);
}
