use crate::{Card, Rank, Suit};
use crate::game::game_info::GameInfo;

pub fn get_allowed_cards(game_info: &GameInfo) -> Vec<Card> {
    let played_moves = game_info.current_moves();
    let current_pidx = game_info.current_pidx();
    let current_cards = game_info.hands()[current_pidx].cards();

    if let Some(first_move) = played_moves.first() {
        let same_suit_cards: Vec<Card> = current_cards.iter().filter(|c| c.suit() == first_move.card().suit()).map(|c| *c).collect();
        if same_suit_cards.len() > 0 {
            same_suit_cards
        } else {
            let non_scoring_cards: Vec<Card> = current_cards.iter().filter(|c| c.score() == 0).map(|c| *c).collect();
            if game_info.first_round() && non_scoring_cards.len() > 0 {
                non_scoring_cards
            } else {
                current_cards.clone()
            }
        }
    } else {
        if game_info.first_round() {
            vec![Card(Rank::Two, Suit::Clubs)]
        } else {
            let non_heart_cards: Vec<Card> = current_cards.iter().filter(|c| c.suit() != Suit::Hearts).map(|c| *c).collect();
            if !game_info.hearts_played() && non_heart_cards.len() > 0 {
                non_heart_cards
            } else {
                current_cards.clone()
            }
        }
    }
}
