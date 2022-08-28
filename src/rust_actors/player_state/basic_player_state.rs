use crate::{Card, Suit};
use crate::rust_actors::player_state::BasicPlayerStateInterface;
use crate::game::data::Move;

#[derive(Clone, Debug)]
pub struct BasicPlayerState {
    cards: Vec<Card>,
    first_round: bool,
    hearts_played: bool,
    pidx: usize,
}

impl BasicPlayerState {
    pub fn new(cards: Vec<Card>, first_round: bool, hearts_played: bool, pidx: usize) -> Self {
        Self { cards, first_round, hearts_played, pidx }
    }
}

impl BasicPlayerStateInterface for BasicPlayerState {
    fn pidx(&self) -> usize {
        self.pidx
    }

    fn set_pidx(&mut self, pidx: usize) {
        self.pidx = pidx
    }

    fn cards(&self) -> &Vec<Card> {
        &self.cards
    }

    fn cards_mut(&mut self) -> &mut Vec<Card> {
        &mut self.cards
    }

    fn set_cards(&mut self, cards: Vec<Card>) {
        self.cards = cards;
    }

    fn first_round(&self) -> bool {
        self.first_round
    }

    fn set_first_round(&mut self, first_round: bool) {
        self.first_round = first_round
    }

    fn hearts_played(&self) -> bool {
        self.hearts_played
    }

    fn set_hearts_played(&mut self, hearts_played: bool) {
        self.hearts_played = hearts_played
    }

    fn update_play_card(&mut self, played_moves: &Vec<Move>) {
        if played_moves.into_iter().any(|m| m.card().suit() == Suit::Hearts) {
            self.hearts_played = true;
        }
    }

    fn update_did_play_card(&mut self, card: &Card) {
        self.cards.retain(|c| c != card);
    }

    fn update_end_round(&mut self, played_moves: &Vec<Move>, _winner_pidx: usize) {
        self.first_round = false;
        if played_moves.into_iter().any(|m| m.card().suit() == Suit::Hearts) {
            self.hearts_played = true;
        }
    }

    fn update_end_pass(&mut self, passed_cards: &Vec<Card>) {
        self.cards.append(&mut passed_cards.clone());
    }
}

impl std::default::Default for BasicPlayerState {
    fn default() -> Self {
        Self {
            cards: vec![],
            first_round: true,
            hearts_played: false,
            pidx: 0,
        }
    }
}
