use crate::{Card, Suit};
use crate::shared::actor::Actor;
use crate::shared::data::Move;

#[derive(Clone)]
pub struct PlayerState {
    pub cards: Vec<Card>,
    pub cards_in_game: [[bool; 13]; 4],
    pub first_round: bool,
    pub hearts_played: bool,
    pub scores: [isize; 4],
    pub still_has: [[bool; 4]; 4],
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            cards: vec![],
            cards_in_game: [[true; 13]; 4],
            first_round: true,
            hearts_played: false,
            scores: [0; 4],
            still_has: [[true; 4]; 4],
        }
    }
}

impl PlayerState {
    fn update_moves(&mut self, moves: &Vec<Move>) {
        if let Some(Move(_, Card(_, first_suit))) = moves.first() {
            for Move(pidx, Card(rank, suit)) in moves {
                self.cards_in_game[suit.to_index()][rank.to_index()] = false;
                if suit != first_suit { self.still_has[suit.to_index()][*pidx] = false; }
            }
        }
    }

    pub fn update_play_card(&mut self, played_moves: &Vec<Move>) {
        self.update_moves(played_moves);
    }

    pub fn update_did_play_card(&mut self, card: &Card) {
        self.cards.retain(|c| c != card);
    }

    pub fn update_end_round(&mut self, played_moves: &Vec<Move>, _winner_pidx: usize) {
        self.first_round = false;
        if played_moves.into_iter().any(|m| m.card().suit() == Suit::Hearts) {
            self.hearts_played = true;
        }

        self.update_moves(played_moves);

        let winner_pidx = crate::internal::round::find_winner_pidx(played_moves);
        self.scores[winner_pidx] += played_moves.iter().map(|m| m.card().score()).sum::<isize>();
    }

    pub fn update_end_pass(&mut self, passed_cards: &Vec<Card>) {
        self.cards.append(&mut passed_cards.clone());
    }

    pub fn final_scores(&self) -> [isize; 4] {
        self.scores.clone() // TODO: Should take 36 into account
    }
}


pub trait MediasResActor : Actor {
    fn new_from_player_state(pidx: usize, player_state: &PlayerState) -> Self;
}
