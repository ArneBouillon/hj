use core::clone::Clone;
use core::iter::IntoIterator;
use crate::shared::actor::Actor;
use crate::shared::data::{Card, Move, Rank, Suit};

use rand::seq::SliceRandom;
use rand::thread_rng;
use crate::PassDirection;

pub struct RandomActor {
    cards: Vec<Card>,
    first_round: bool,
    hearts_played: bool,
    pidx: usize,
}


impl RandomActor {
    pub fn new() -> Self {
        Self { cards: vec![], first_round: true, hearts_played: false, pidx: 0 }
    }
}

impl Actor for RandomActor {
    fn initialize(&mut self, pidx: usize, cards: &Vec<Card>) {
        self.pidx = pidx;
        self.cards = cards.clone();
    }

    fn play_card(&mut self, played_moves: &Vec<Move>) -> Card {
        self.cards.shuffle(&mut thread_rng());

        let new_card = if let Some(first_move) = played_moves.first() {
            if let Some(new_card) = self.cards.iter().find(|c| c.suit() == first_move.card().suit()) {
                *new_card
            } else {
                if let (true, Some(new_card)) = (self.first_round, self.cards.iter().find(|c| c.score() == 0)) {
                    *new_card
                } else {
                    *self.cards.first().expect("There should be at least one card.")
                }
            }
        } else {
            if self.first_round {
                Card(Rank::Two, Suit::Clubs)
            } else if let (false, Some(new_card)) = (self.hearts_played, self.cards.iter().find(|c| c.suit() != Suit::Hearts)) {
                *new_card
            } else {
                *self.cards.first().expect("There should be at least one card.")
            }
        };

        self.cards.retain(|c| *c != new_card);
        new_card
    }

    fn end_round(&mut self, _winner_pidx: usize, played_moves: &Vec<Move>) {
        self.first_round = false;
        if played_moves.into_iter().any(|m| m.card().suit() == Suit::Hearts) {
            self.hearts_played = true;
        }
    }

    fn end_game(&mut self, _score: [isize; 4]) {}

    fn get_pass(&mut self, direction: PassDirection) -> Vec<Card> {
        let passed_cards = match direction {
            PassDirection::None => vec![],
            _ => {
                self.cards.shuffle(&mut thread_rng());
                self.cards.clone().into_iter().take(3).collect()
            }
        };
        for passed_card in &passed_cards { self.cards.retain(|c| c != passed_card); }
        passed_cards
    }

    fn end_pass(&mut self, passed_cards: &Vec<Card>) {
        self.cards.append(&mut passed_cards.clone());
    }
}
