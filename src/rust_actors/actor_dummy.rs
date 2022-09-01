use crate::{Card, PassDirection};
use crate::game::actor::Actor;
use crate::game::data::Move;
use crate::rust_actors::player_state::BasicPlayerStateInterface;

#[derive(Debug)]
pub struct ActorDummy<PlayerState: BasicPlayerStateInterface> {
    player_state: PlayerState,
    next_card: Option<Card>,
}

impl<PlayerState: BasicPlayerStateInterface> ActorDummy<PlayerState> {
    pub fn new(player_state: PlayerState, next_card: Option<Card>) -> Self {
        Self { next_card, player_state }
    }

    pub fn player_state_move(self) -> PlayerState {
        self.player_state
    }
}

impl<PlayerState: BasicPlayerStateInterface> Actor for ActorDummy<PlayerState> {
    fn initialize(&mut self, _pidx: usize, _cards: &Vec<Card>) {
        panic!("Dummy actors should never receive #initialize");
    }

    fn play_card(&mut self, played_moves: &Vec<Move>) -> Card {
        if let Some(card) = self.next_card {
            self.player_state.update_play_card(played_moves);
            self.player_state.update_did_play_card(&card);
            self.next_card = None;
            card
        } else {
            panic!("This dummy actor has no card left to play")
        }
    }

    fn end_round(&mut self, winner_pidx: usize, played_moves: &Vec<Move>) {
        self.player_state.update_end_round(played_moves, winner_pidx);
    }

    fn end_game(&mut self, _score: [isize; 4]) {
        todo!()
    }

    fn get_pass(&mut self, _direction: PassDirection) -> Vec<Card> {
        todo!()
    }

    fn end_pass(&mut self, _passed_cards: &Vec<Card>) {
        todo!()
    }
}
