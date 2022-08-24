use crate::shared::actor::Actor;
use crate::shared::data::{Card, Hand, Move, PassDirection, Rank, Suit};
use crate::shared::player::Player;
use std::convert::TryInto;

use iter_fixed::IntoIteratorFixed;

use crate::util;

pub struct GameState<'a> {
    current_round: usize,
    hearts_played: bool,
    players: [Player<'a>; 4],
    score: [isize; 4],
    scored: [bool; 4],
}


impl<'a> GameState<'a> {
    pub fn new_from_hands(hands: [Hand; 4], actors: [&'a mut dyn Actor; 4]) -> Self {
        let players = actors.into_iter_fixed().zip(hands).enumerate().map(|(pidx, (a, h))| {
            a.initialize(pidx, h.cards());
            Player::new(a, h)
        }).collect();

        Self { current_round: 0, hearts_played: false, players, score: [0; 4], scored: [false; 4] }
    }

    pub fn new(actors: [&'a mut dyn Actor; 4]) -> Self {
        let hands = util::deck::get_shuffled_hands();
        Self::new_from_hands(hands, actors)
    }

    pub fn current_round(&self) -> usize {
        self.current_round
    }

    pub fn hearts_played(&self) -> bool {
        self.hearts_played
    }

    pub fn players(&self) -> &[Player<'a>; 4] {
        &self.players
    }

    pub fn players_mut(&mut self) -> &mut [Player<'a>; 4] {
        &mut self.players
    }

    pub fn score(&self) -> [isize; 4] {
        self.score.clone()
    }

    pub fn did_play_move(&mut self, played_move: Move) {
        if let Card(_, Suit::Hearts) = played_move.card() {
            self.hearts_played = true;
        }

        self.players[played_move.pidx()].hand_mut().remove(played_move.card());
    }

    pub fn did_play_round(&mut self, winner_pidx: usize, played_moves: Vec<Move>) {
        self.current_round += 1;

        self.score[winner_pidx] += played_moves.iter().map(|m| m.card().score()).sum::<isize>();
        self.scored[winner_pidx] |= played_moves.iter().any(|m| m.card().score() != 0);

        for player in self.players_mut() {
            player.actor_mut().end_round(winner_pidx, &played_moves);
        }
    }

    pub fn did_play_game(&mut self) {
        if self.scored.iter().filter(|b| **b).count() == 1 {
            for pidx in 0..4 {
                if self.scored[pidx] { self.score[pidx] = 0; }
                else { self.score[pidx] = 36; }
            }
        }
    }

    pub fn did_pass(&mut self, direction: PassDirection, passed_cards: &[Vec<Card>; 4]) {
        for pidx in 0..4 {
            for card in &passed_cards[pidx] {
                self.players[pidx].hand_mut().remove(*card);
                self.players[(pidx + direction.index_shift()) % 4].hand_mut().add(*card);
            }
        }
    }
}
