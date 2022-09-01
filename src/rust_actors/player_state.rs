use crate::Card;
use crate::game::actor::Actor;
use crate::game::data::Move;

pub mod basic_player_state;
pub mod default_player_state;
pub mod extended_player_state;

pub trait BasicPlayerStateInterface: std::default::Default + Clone + std::fmt::Debug {
    fn pidx(&self) -> usize;
    fn set_pidx(&mut self, pidx: usize);
    fn cards(&self) -> &Vec<Card>;
    fn cards_mut(&mut self) -> &mut Vec<Card>;
    fn set_cards(&mut self, cards: Vec<Card>);
    fn first_round(&self) -> bool;
    fn set_first_round(&mut self, first_round: bool);
    fn hearts_played(&self) -> bool;
    fn set_hearts_played(&mut self, hearts_played: bool);

    fn update_play_card(&mut self, played_moves: &Vec<Move>);
    fn update_did_play_card(&mut self, card: &Card);
    fn update_end_round(&mut self, played_moves: &Vec<Move>, winner_pidx: usize);
    fn update_end_pass(&mut self, passed_cards: &Vec<Card>);
}

pub trait DefaultPlayerStateInterface: BasicPlayerStateInterface {
    fn cards_in_game(&self) -> &[[bool; 13]; 4];
    fn scores(&self) -> &[isize; 4];
    fn scored(&self) -> &[bool; 4];
    fn still_has(&self) -> &[[bool; 4]; 4];
    fn still_has_mut(&mut self) -> &mut [[bool; 4]; 4];

    fn final_scores(&self) -> [isize; 4];
}

pub trait ExtendedPlayerStateInterface: DefaultPlayerStateInterface {
    fn cards_in_game_by_suit(&self) -> &[usize; 4];
    fn opponent_cards_in_game(&self) -> &[[bool; 13]; 4];
    fn opponent_cards_in_game_by_suit(&self) -> &[usize; 4];
}

pub trait MediasResActor<PlayerState: BasicPlayerStateInterface>: Actor {
    fn new_from_player_state(player_state: &PlayerState) -> Self;
    fn add_dummy(&mut self, card: Card);
}
