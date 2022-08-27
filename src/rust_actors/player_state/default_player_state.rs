use crate::Card;
use crate::rust_actors::player_state::{BasicPlayerStateInterface, DefaultPlayerStateInterface};
use crate::rust_actors::player_state::basic_player_state::BasicPlayerState;
use crate::shared::data::Move;

#[derive(Clone)]
pub struct DefaultPlayerState {
    basic_player_state: BasicPlayerState,

    cards_in_game: [[bool; 13]; 4],
    scores: [isize; 4],
    still_has: [[bool; 4]; 4],
}

impl DefaultPlayerState {
    pub fn new(cards: Vec<Card>, first_round: bool, hearts_played: bool, pidx: usize, cards_in_game: [[bool; 13]; 4], scores: [isize; 4], still_has: [[bool; 4]; 4]) -> Self {
        Self { basic_player_state: BasicPlayerState::new(cards, first_round, hearts_played, pidx), cards_in_game, scores, still_has }
    }

    fn update_moves(&mut self, moves: &Vec<Move>) {
        if let Some(Move(_, Card(_, first_suit))) = moves.first() {
            for Move(pidx, Card(rank, suit)) in moves {
                self.cards_in_game[suit.to_index()][rank.to_index()] = false;
                if suit != first_suit { self.still_has[suit.to_index()][*pidx] = false; }
            }
        }
    }
}

impl BasicPlayerStateInterface for DefaultPlayerState {
    fn pidx(&self) -> usize {
        self.basic_player_state.pidx()
    }

    fn set_pidx(&mut self, pidx: usize) {
        self.basic_player_state.set_pidx(pidx);
    }

    fn cards(&self) -> &Vec<Card> {
        self.basic_player_state.cards()
    }

    fn cards_mut(&mut self) -> &mut Vec<Card> {
        self.basic_player_state.cards_mut()
    }

    fn set_cards(&mut self, cards: Vec<Card>) {
        self.basic_player_state.set_cards(cards)
    }

    fn first_round(&self) -> bool {
        self.basic_player_state.first_round()
    }

    fn set_first_round(&mut self, first_round: bool) {
        self.basic_player_state.set_first_round(first_round);
    }

    fn hearts_played(&self) -> bool {
        self.basic_player_state.hearts_played()
    }

    fn set_hearts_played(&mut self, hearts_played: bool) {
        self.basic_player_state.set_hearts_played(hearts_played);
    }

    fn update_play_card(&mut self, played_moves: &Vec<Move>) {
        self.basic_player_state.update_play_card(played_moves);

        self.update_moves(played_moves);
    }

    fn update_did_play_card(&mut self, card: &Card) {
        self.basic_player_state.update_did_play_card(card);
    }

    fn update_end_round(&mut self, played_moves: &Vec<Move>, winner_pidx: usize) {
        self.basic_player_state.update_end_round(played_moves, winner_pidx);

        self.scores[winner_pidx] += played_moves.iter().map(|m| m.card().score()).sum::<isize>();
        self.update_moves(played_moves);
    }

    fn update_end_pass(&mut self, passed_cards: &Vec<Card>) {
        self.basic_player_state.update_end_pass(passed_cards);
    }
}

impl DefaultPlayerStateInterface for DefaultPlayerState {
    fn cards_in_game(&self) -> &[[bool; 13]; 4] {
        &self.cards_in_game
    }

    fn scores(&self) -> &[isize; 4] {
        &self.scores
    }

    fn still_has(&self) -> &[[bool; 4]; 4] {
        &self.still_has
    }

    fn still_has_mut(&mut self) -> &mut [[bool; 4]; 4] {
        &mut self.still_has
    }

    fn final_scores(&self) -> [isize; 4] {
        self.scores.clone() // TODO: Should take 36 into account
    }
}

impl std::default::Default for DefaultPlayerState {
    fn default() -> Self {
        Self {
            basic_player_state: std::default::Default::default(),
            cards_in_game: [[true; 13]; 4],
            scores: [0; 4],
            still_has: [[true; 4]; 4],
        }
    }
}
