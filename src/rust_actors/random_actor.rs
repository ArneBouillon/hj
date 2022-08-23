use core::clone::Clone;
use core::iter::IntoIterator;
use crate::shared::actor::Actor;
use crate::shared::data::{Card, Move, Rank, Suit};

use rand::seq::SliceRandom;
use rand::thread_rng;
use crate::PassDirection;
use crate::rust_actors::shared::{MediasResActor, PlayerState};

pub struct RandomActor {
    pidx: usize,
    pub player_state: PlayerState,
}


impl RandomActor {
    pub fn new() -> Self {
        Self { pidx: 0, player_state: Default::default() }
    }
}

impl Actor for RandomActor {
    fn initialize(&mut self, pidx: usize, cards: &Vec<Card>) {
        self.pidx = pidx;
        self.player_state.cards = cards.clone();
    }

    fn play_card(&mut self, played_moves: &Vec<Move>) -> Card {
        self.player_state.update_play_card(played_moves);
        self.player_state.cards.shuffle(&mut thread_rng());

        let new_card = if let Some(first_move) = played_moves.first() {
            if let Some(new_card) = self.player_state.cards.iter().find(|c| c.suit() == first_move.card().suit()) {
                *new_card
            } else {
                if let (true, Some(new_card)) = (self.player_state.first_round, self.player_state.cards.iter().find(|c| c.score() == 0)) {
                    *new_card
                } else {
                    *self.player_state.cards.first().expect("There should be at least one card.")
                }
            }
        } else {
            if self.player_state.first_round {
                Card(Rank::Two, Suit::Clubs)
            } else if let (false, Some(new_card)) = (self.player_state.hearts_played, self.player_state.cards.iter().find(|c| c.suit() != Suit::Hearts)) {
                *new_card
            } else {
                *self.player_state.cards.first().expect("There should be at least one card.")
            }
        };

        self.player_state.cards.retain(|c| *c != new_card);
        new_card
    }

    fn end_round(&mut self, _winner_pidx: usize, played_moves: &Vec<Move>) {
        self.player_state.first_round = false;
        if played_moves.into_iter().any(|m| m.card().suit() == Suit::Hearts) {
            self.player_state.hearts_played = true;
        }
    }

    fn end_game(&mut self, _score: [isize; 4]) {}

    fn get_pass(&mut self, direction: PassDirection) -> Vec<Card> {
        let passed_cards = match direction {
            PassDirection::None => vec![],
            _ => {
                self.player_state.cards.shuffle(&mut thread_rng());
                self.player_state.cards.clone().into_iter().take(3).collect()
            }
        };
        for passed_card in &passed_cards { self.player_state.cards.retain(|c| c != passed_card); }
        passed_cards
    }

    fn end_pass(&mut self, passed_cards: &Vec<Card>) {
        self.player_state.cards.append(&mut passed_cards.clone());
    }
}

impl MediasResActor for RandomActor {
    fn new_from_player_state(pidx: usize, player_state: &PlayerState) -> Self {
        Self {
            pidx,
            player_state: player_state.clone()
        }
    }
}
