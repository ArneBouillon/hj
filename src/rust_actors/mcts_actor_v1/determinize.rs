use crate::Card;
use crate::rust_actors::mcts_actor_v1::game_state::GameState;
use crate::rust_actors::shared::PlayerState;

use itertools::Itertools;
use std::convert::TryInto;
use crate::shared::data::{Hand, Move};

use rand::seq::SliceRandom;
use rand::thread_rng;
use tap::Tap;

fn new_player_state(cards: Vec<Card>, other_state: &PlayerState) -> PlayerState {
    PlayerState {
        cards,
        cards_in_game: other_state.cards_in_game.clone(),
        first_round: other_state.first_round,
        hearts_played: other_state.hearts_played,
        scores: other_state.scores.clone(),
        still_has: other_state.still_has.clone(),
    }
}

pub(super) fn determinize(pidx: usize, player_state: &PlayerState, played_moves: &Vec<Move>) -> GameState {
    let mut opponent_cards_in_game = player_state.cards_in_game.clone();
    for card in &player_state.cards { opponent_cards_in_game[card.suit().to_index()][card.rank().to_index()] = false; }

    let mut has_played = [false; 4];
    for played_move in played_moves { has_played[played_move.pidx()] = true; }

    let mut opponent_cards_left = Card::all().iter()
                                             .filter(|c| opponent_cards_in_game[c.suit().to_index()][c.rank().to_index()])
                                             .map(|c| *c)
                                             .collect::<Vec<Card>>();
    opponent_cards_left.shuffle(&mut thread_rng());

    // println!("{}, {}, {:?}, {:?}", pidx, opponent_cards_left.len(), played_moves.iter().map(|m| m.pidx()).collect::<Vec<usize>>(), has_played);

    let num_if_played = opponent_cards_left.len() / 3;
    let num_if_not_played = num_if_played + if opponent_cards_left.len() % 3 == 0 { 0 } else { 1 };


    let ps0 = if pidx == 0 { player_state.clone() } else {
        opponent_cards_left.sort_by_key(|c| (!player_state.still_has[c.suit().to_index()][0]) as usize);
        new_player_state(
            opponent_cards_left.drain(
                if has_played[0] { 0..num_if_played } else { 0..num_if_not_played }
            ).collect(),
            player_state,
        ).tap(|_| opponent_cards_left.shuffle(&mut thread_rng()))
    };
    let ps1 = if pidx == 1 { player_state.clone() } else {
        opponent_cards_left.sort_by_key(|c| (!player_state.still_has[c.suit().to_index()][1]) as usize);
        new_player_state(opponent_cards_left.drain(if has_played[1] { 0..num_if_played } else { 0..num_if_not_played }).collect(), player_state)
            .tap(|_| opponent_cards_left.shuffle(&mut thread_rng()))
    };
    let ps2 = if pidx == 2 { player_state.clone() } else {        opponent_cards_left.sort_by_key(|c| (!player_state.still_has[c.suit().to_index()][2]) as usize);
        new_player_state(opponent_cards_left.drain(if has_played[2] { 0..num_if_played } else { 0..num_if_not_played }).collect(), player_state).tap(|_| opponent_cards_left.shuffle(&mut thread_rng())) };
    let ps3 = if pidx == 3 { player_state.clone() } else {         opponent_cards_left.sort_by_key(|c| (!player_state.still_has[c.suit().to_index()][3]) as usize);
        new_player_state(opponent_cards_left.drain(if has_played[3] { 0..num_if_played } else { 0..num_if_not_played }).collect(), player_state) };

    GameState {
        current_moves: played_moves.clone(),
        current_pidx: pidx,
        player_states: [ps0, ps1, ps2, ps3],
    }
}
