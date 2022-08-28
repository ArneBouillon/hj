use crate::Card;
use crate::game::data::{Hand, Move};

use rand::seq::SliceRandom;
use rand::thread_rng;

pub fn get_shuffled_hands() -> [Hand; 4] {
    let mut cards = Card::all();

    cards.shuffle(&mut thread_rng());
    cards.chunks(13)
         .map(move |cards| Hand::new(cards.to_vec()))
         .collect::<Vec<Hand>>()
         .try_into()
         .expect("52 cards should divide into exactly four hands.")
}

pub fn find_winner_pidx(played_moves: &Vec<Move>) -> usize {
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
