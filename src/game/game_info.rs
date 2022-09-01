use crate::{Card, PassDirection, Rank, Suit};
use crate::game::actor::Actor;
use crate::game::data::{Hand, Move};
use crate::game::errors::HJError;
use crate::game::validator::{NoValidator, Validator};
use crate::internal::hj_validator::HJValidator;
use crate::rust_actors::actor_human::ActorHuman;
use crate::util::deck::find_winner_pidx;

#[derive(Clone)]
pub struct GameInfo {
    current_moves: Vec<Move>,
    current_pidx: usize,
    current_round: usize,
    hands: [Hand; 4],
    hearts_played: bool,
    score: [isize; 4],
    scored: [bool; 4],
}

#[derive(Eq, PartialEq, Debug)]
pub enum StopCondition {
    None,
    OneMove,
    Until(usize),
}

impl GameInfo {
    pub fn new(hands: [Hand; 4]) -> Self {
        let first_pidx = (0..4).into_iter()
            .find(|pidx| hands[*pidx].any_match(Card(Rank::Two, Suit::Clubs)))
            .expect("Someone should have the two of clubs.");

        Self {
            current_moves: vec![],
            current_pidx: first_pidx,
            current_round: 0,
            hands,
            hearts_played: false,
            score: [0; 4],
            scored: [false; 4]
        }
    }

    pub fn reconstruct(current_moves: Vec<Move>, current_pidx: usize, current_round: usize, hands: [Hand; 4], hearts_played: bool, score: [isize; 4], scored: [bool; 4]) -> Self {
        // println!(" -- Reconstructing; hearts_played={}", hearts_played);
        Self { current_moves, current_pidx, current_round, hands, hearts_played, score, scored }
    }

    pub fn current_moves(&self) -> &Vec<Move> { &self.current_moves }

    pub fn current_pidx(&self) -> usize { self.current_pidx }

    pub fn current_round(&self) -> usize {
        self.current_round
    }

    pub fn first_round(&self) -> bool { self.current_round == 0 }

    pub fn hands(&self) -> &[Hand; 4] { &self.hands }

    pub fn hearts_played(&self) -> bool {
        self.hearts_played
    }

    pub fn score(&self) -> [isize; 4] {
        self.score.clone()
    }

    pub fn result(&self) -> Option<[isize; 4]> {
        if self.current_round == 13 {
            Some(self.score.clone())
        } else {
            None
        }
    }

    fn did_play_move(&mut self, played_move: Move) {
        if let Card(_, Suit::Hearts) = played_move.card() {
            self.hearts_played = true;
        }

        self.hands[played_move.pidx()].remove(played_move.card());

        self.current_moves.push(played_move);
        self.current_pidx = (self.current_pidx + 1) % 4;
    }

    fn did_play_round(&mut self, winner_pidx: usize) {
        self.current_round += 1;

        self.score[winner_pidx] += self.current_moves.iter().map(|m| m.card().score()).sum::<isize>();
        self.scored[winner_pidx] |= self.current_moves.iter().any(|m| m.card().score() != 0);

        self.current_moves = vec![];
        self.current_pidx = winner_pidx;
    }

    fn did_play_game(&mut self) {
        if self.scored.iter().filter(|b| **b).count() == 1 {
            for pidx in 0..4 {
                if self.scored[pidx] { self.score[pidx] = 0; }
                else { self.score[pidx] = 36; }
            }
        }
    }

    fn did_pass(&mut self, direction: PassDirection, passed_cards: &[Vec<Card>; 4]) {
        for pidx in 0..4 {
            for card in &passed_cards[pidx] {
                let new_pidx = (pidx + direction.index_shift()) % 4;
                self.hands[pidx].remove(*card);
                self.hands[new_pidx].add(*card);
                if card == &Card(Rank::Two, Suit::Clubs) { self.current_pidx = new_pidx; }
            }
        }
    }

    pub fn pass<V: Validator, T: Actor + ?Sized>(&mut self, direction: PassDirection, actors: &mut [&mut T; 4]) -> Result<(), HJError> {
        let passed_cards: [Vec<Card>; 4] = [0, 1, 2, 3].map(|pidx|
            actors[pidx].get_pass(direction)
        );

        for pidx in 0..4 { V::validate_pass(direction, &passed_cards[pidx], pidx, self)?; }
        self.did_pass(direction, &passed_cards);
        for pidx in 0..4 { actors[(pidx + direction.index_shift()) % 4].end_pass(&passed_cards[pidx]); }

        Ok(())
    }

    pub fn play<V: Validator, T: Actor + ?Sized>(&mut self, actors: &mut [&mut T; 4], stop: StopCondition) -> Result<(), HJError> {
        let mut done_move = false;

        while self.current_round < 13 {
            while self.current_moves.len() < 4 {
                if stop == StopCondition::OneMove && done_move { return Ok(()); }
                if stop == StopCondition::Until(self.current_pidx) && done_move { return Ok(()); }
                let new_move = Move(
                    self.current_pidx,
                    actors[self.current_pidx].play_card(&self.current_moves),
                );

                V::validate_move(self, &self.current_moves, new_move)?;
                self.did_play_move(new_move);
                done_move = true;
            }

            let winner_pidx = find_winner_pidx(&self.current_moves);
            for actor in actors.iter_mut() {
                actor.end_round(winner_pidx, &self.current_moves);
            }
            self.did_play_round(winner_pidx);
        }

        self.did_play_game();
        for actor in actors.iter_mut() { actor.end_game(self.result().unwrap()); }
        Ok(())
    }

    pub fn play_without_validator<T: Actor + ?Sized>(&mut self, actors: &mut [&mut T; 4], stop: StopCondition) {
        self.play::<NoValidator, T>(actors, stop).expect("Without a validator, no error should occur."); // TODO: No validator
    }
}
