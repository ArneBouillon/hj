use iter_fixed::IntoIteratorFixed;
use crate::{Card, DefaultPlayerState};

use crate::game::data::{Hand, Move};

use rand::seq::SliceRandom;
use rand::thread_rng;
use tap::Tap;
use crate::game::game_info::GameInfo;
use crate::rust_actors::player_state::DefaultPlayerStateInterface;
use crate::rust_actors::player_state::BasicPlayerStateInterface;

fn new_player_state<PlayerState : DefaultPlayerStateInterface>(cards: Vec<Card>, other_state: &PlayerState) -> DefaultPlayerState {
    // println!("In new_player_state; hearts_played={}", other_state.hearts_played());
    DefaultPlayerState::new(
        cards,
        other_state.first_round(),
        other_state.hearts_played(),
        other_state.pidx(),
        other_state.cards_in_game().clone(),
        other_state.scores().clone(),
        other_state.scored().clone(),
        other_state.still_has().clone(),
    )
}

pub(super) fn determinize<PlayerState : DefaultPlayerStateInterface>(pidx: usize, player_state: &PlayerState, played_moves: &Vec<Move>) -> (GameInfo, [DefaultPlayerState; 4]) {
    let mut opponent_cards_in_game = player_state.cards_in_game().clone();
    for card in player_state.cards() { opponent_cards_in_game[card.suit().to_index()][card.rank().to_index()] = false; }

    let mut has_played = [false; 4];
    for played_move in played_moves { has_played[played_move.pidx()] = true; }

    let mut opponent_cards_left = Card::all().iter()
                                             .filter(|c| opponent_cards_in_game[c.suit().to_index()][c.rank().to_index()])
                                             .map(|c| *c)
                                             .collect::<Vec<Card>>();
    opponent_cards_left.shuffle(&mut thread_rng());

    let num_if_played = opponent_cards_left.len() / 3;
    let num_if_not_played = num_if_played + if opponent_cards_left.len() % 3 == 0 { 0 } else { 1 };

    let ps0 = if pidx == 0 { new_player_state(player_state.cards().clone(), player_state) } else {
        opponent_cards_left.sort_by_key(|c| (!player_state.still_has()[c.suit().to_index()][0]) as usize);
        new_player_state(
            opponent_cards_left.drain(
                if has_played[0] { 0..num_if_played } else { 0..num_if_not_played }
            ).collect(),
            player_state,
        ).tap(|_| opponent_cards_left.shuffle(&mut thread_rng()))
    };
    let ps1 = if pidx == 1 { new_player_state(player_state.cards().clone(), player_state) } else {
        opponent_cards_left.sort_by_key(|c| (!player_state.still_has()[c.suit().to_index()][1]) as usize);
        new_player_state(opponent_cards_left.drain(if has_played[1] { 0..num_if_played } else { 0..num_if_not_played }).collect(), player_state)
            .tap(|_| opponent_cards_left.shuffle(&mut thread_rng()))
    };
    let ps2 = if pidx == 2 { new_player_state(player_state.cards().clone(), player_state) } else {        opponent_cards_left.sort_by_key(|c| (!player_state.still_has()[c.suit().to_index()][2]) as usize);
        new_player_state(opponent_cards_left.drain(if has_played[2] { 0..num_if_played } else { 0..num_if_not_played }).collect(), player_state).tap(|_| opponent_cards_left.shuffle(&mut thread_rng())) };
    let ps3 = if pidx == 3 { new_player_state(player_state.cards().clone(), player_state) } else {         opponent_cards_left.sort_by_key(|c| (!player_state.still_has()[c.suit().to_index()][3]) as usize);
        new_player_state(opponent_cards_left.drain(if has_played[3] { 0..num_if_played } else { 0..num_if_not_played }).collect(), player_state) };

    let hands = [Hand::new(ps0.cards().clone()), Hand::new(ps1.cards().clone()), Hand::new(ps2.cards().clone()), Hand::new(ps3.cards().clone())];
    let player_states = [ps0, ps1, ps2, ps3];
    (
        GameInfo::reconstruct(played_moves.clone(), pidx, 13 - player_state.cards().len(), hands, player_state.hearts_played(), player_state.scores().clone(), player_state.scored().clone()),
        player_states
    )
}
