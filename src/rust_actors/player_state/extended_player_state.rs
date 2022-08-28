use crate::Card;
use crate::rust_actors::player_state::{BasicPlayerStateInterface, DefaultPlayerStateInterface, ExtendedPlayerStateInterface};
use crate::rust_actors::player_state::default_player_state::DefaultPlayerState;
use crate::game::data::Move;

#[derive(Clone, Debug)]
pub struct ExtendedPlayerState {
    default_player_state: DefaultPlayerState,

    cards_in_game_by_suit: [usize; 4],
    opponent_cards_in_game: [[bool; 13]; 4],
    opponent_cards_in_game_by_suit: [usize; 4],
}

impl ExtendedPlayerState {
    pub fn new(cards: Vec<Card>, first_round: bool, hearts_played: bool, pidx: usize, cards_in_game: [[bool; 13]; 4], scores: [isize; 4], scored: [bool; 4], still_has: [[bool; 4]; 4], cards_in_game_by_suit: [usize; 4],
           opponent_cards_in_game: [[bool; 13]; 4],
           opponent_cards_in_game_by_suit: [usize; 4]) -> Self {
        Self { default_player_state: DefaultPlayerState::new(
            cards, first_round, hearts_played, pidx, cards_in_game, scores, scored, still_has
        ), cards_in_game_by_suit, opponent_cards_in_game, opponent_cards_in_game_by_suit }
    }

    fn update_moves(&mut self, moves: &Vec<Move>) {
        for Move(pidx, Card(rank, suit)) in moves {
            if *pidx != self.pidx() {
                if self.opponent_cards_in_game[suit.to_index()][rank.to_index()] { self.opponent_cards_in_game_by_suit[suit.to_index()] -= 1; }
                self.opponent_cards_in_game[suit.to_index()][rank.to_index()] = false;
            }

            if self.cards_in_game()[suit.to_index()][rank.to_index()] { self.cards_in_game_by_suit[suit.to_index()] -= 1; }
        }
    }
}

impl DefaultPlayerStateInterface for ExtendedPlayerState {
    fn cards_in_game(&self) -> &[[bool; 13]; 4] {
        self.default_player_state.cards_in_game()
    }

    fn scores(&self) -> &[isize; 4] {
        self.default_player_state.scores()
    }

    fn scored(&self) -> &[bool; 4] {
        self.default_player_state.scored()
    }

    fn still_has(&self) -> &[[bool; 4]; 4] {
        self.default_player_state.still_has()
    }

    fn still_has_mut(&mut self) -> &mut [[bool; 4]; 4] {
        self.default_player_state.still_has_mut()
    }

    fn final_scores(&self) -> [isize; 4] {
        self.default_player_state.final_scores()
    }
}

impl BasicPlayerStateInterface for ExtendedPlayerState {
    fn pidx(&self) -> usize {
        self.default_player_state.pidx()
    }

    fn set_pidx(&mut self, pidx: usize) {
        self.default_player_state.set_pidx(pidx);
    }

    fn cards(&self) -> &Vec<Card> {
        self.default_player_state.cards()
    }

    fn cards_mut(&mut self) -> &mut Vec<Card> {
        self.default_player_state.cards_mut()
    }

    fn set_cards(&mut self, cards: Vec<Card>) {
        self.default_player_state.set_cards(cards);
        for card in self.cards().clone() {
            self.opponent_cards_in_game[card.suit().to_index()][card.rank().to_index()] = false;
            self.opponent_cards_in_game_by_suit[card.suit().to_index()] -= 1;
        }
    }

    fn first_round(&self) -> bool {
        self.default_player_state.first_round()
    }

    fn set_first_round(&mut self, first_round: bool) {
        self.default_player_state.set_first_round(first_round)
    }

    fn hearts_played(&self) -> bool {
        self.default_player_state.hearts_played()
    }

    fn set_hearts_played(&mut self, hearts_played: bool) {
        self.default_player_state.set_hearts_played(hearts_played)
    }

    fn update_play_card(&mut self, played_moves: &Vec<Move>) {
        self.default_player_state.update_play_card(played_moves);

        self.update_moves(played_moves);
    }

    fn update_did_play_card(&mut self, card: &Card) {
        self.default_player_state.update_did_play_card(card);
    }

    fn update_end_round(&mut self, played_moves: &Vec<Move>, winner_pidx: usize) {
        self.default_player_state.update_end_round(played_moves, winner_pidx);

        self.update_moves(played_moves);
    }

    fn update_end_pass(&mut self, passed_cards: &Vec<Card>) {
        self.default_player_state.update_end_pass(passed_cards);
    }
}

impl ExtendedPlayerStateInterface for ExtendedPlayerState {
    fn cards_in_game_by_suit(&self) -> &[usize; 4] {
       &self.cards_in_game_by_suit
    }

    fn opponent_cards_in_game(&self) -> &[[bool; 13]; 4] {
        &self.opponent_cards_in_game
    }

    fn opponent_cards_in_game_by_suit(&self) -> &[usize; 4] {
        &self.opponent_cards_in_game_by_suit
    }
}

impl std::default::Default for ExtendedPlayerState {
    fn default() -> Self {
        Self {
            default_player_state: std::default::Default::default(),
            cards_in_game_by_suit: [13; 4],
            opponent_cards_in_game: [[true; 13]; 4],
            opponent_cards_in_game_by_suit: [13; 4],
        }
    }
}
