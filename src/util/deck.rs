use crate::Card;
use crate::shared::data::Hand;

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
