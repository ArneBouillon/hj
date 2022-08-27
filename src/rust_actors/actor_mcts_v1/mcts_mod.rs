use crate::shared::data::Move;
use crate::shared::actor::Actor;

use std::time::SystemTime;
use std::collections::HashMap;

use crate::{Card, Rank, ActorRuleV1, Suit, ExtendedPlayerState, DefaultPlayerState};
use crate::rust_actors::actor_mcts_v1::determinize::determinize;
use crate::rust_actors::actor_mcts_v1::game_state::GameState;
use crate::rust_actors::player_state::DefaultPlayerStateInterface;
use crate::rust_actors::player_state::BasicPlayerStateInterface;
use crate::rust_actors::player_state::MediasResActor;

struct Node {
    visits: usize,
    value: f32,
    children: HashMap<Card, Node>,
    children_left: isize,
    cards: Vec<Card>,
}

impl Node {
    pub fn new(cards: Vec<Card>) -> Node {
        Node {
            visits: 0,
            value: 0.,
            children: HashMap::new(),
            children_left: -1,
            cards,
        }
    }

    pub fn children(&self) -> &HashMap<Card, Node> {
        &self.children
    }

    pub fn children_mut(&mut self) -> &mut HashMap<Card, Node> {
        &mut self.children
    }

    pub fn cards_mut(&mut self) -> &mut Vec<Card> {
        &mut self.cards
    }

    pub fn search_weight(&self, parent_visits_ln: f32) -> f32 {
        if self.visits == 0 {
            0.
        } else {
            (self.value as f32 / self.visits as f32) + (2. * parent_visits_ln / self.visits as f32).sqrt()
        }
    }

    pub fn ensure_children(&mut self) {
        if self.children_left == -1 {
            self.children = initial_vec(self.cards_mut());
            self.children_left = self.children.len() as isize;
        }
    }

    pub fn update(&mut self, result: isize) {
        self.visits += 1;
        self.value += (36 - result) as f32 / 46.;
    }
}


fn initial_vec(cards: &Vec<Card>) -> HashMap<Card, Node> {
    cards.iter().map(|c| {
        let mut new_cards = cards.clone();
        new_cards.retain(|cc| cc != c);
        (*c, Node::new(new_cards))
    }).collect()
}

fn play_for_a_bit<SubActor: MediasResActor<DefaultPlayerState>>(state: &mut GameState<DefaultPlayerState>, special_pidx: usize, special_card: Card) -> Option<[isize; 4]> {
    let mut actors = [
        SubActor::new_from_player_state(&state.player_states[0]),
        SubActor::new_from_player_state(&state.player_states[1]),
        SubActor::new_from_player_state(&state.player_states[2]),
        SubActor::new_from_player_state(&state.player_states[3]),
    ];
    let mut done_special = false;

    loop {
        state.player_states[state.current_pidx].update_play_card(&state.current_moves);
        let new_card = if state.current_pidx == special_pidx {
            // println!("In special pidx");
            if done_special { return None; }

            done_special = true;
            special_card
        } else {
            actors[state.current_pidx].play_card(&state.current_moves)
        };
        state.player_states[state.current_pidx].update_did_play_card(&new_card);
        state.current_moves.push(Move(state.current_pidx, new_card));

        if state.current_moves.len() == 4 {
            // println!("Round finished");
            state.current_pidx = crate::internal::round::find_winner_pidx(&state.current_moves);
            actors.iter_mut().for_each(|actor| actor.end_round(state.current_pidx, &state.current_moves));
            state.player_states.iter_mut().for_each(|ps| ps.update_end_round(&state.current_moves, state.current_pidx));
            state.current_moves = vec![];
            if state.player_states[0].cards().len() == 0 { return Some(actors[0].player_state.final_scores()); }
        } else {
            state.current_pidx = (state.current_pidx + 1) % 4;
        }
    }
}

fn play_randomly<SubActor: MediasResActor<DefaultPlayerState>>(state: &GameState<DefaultPlayerState>) -> [isize; 4] {
    let mut actors = [
        SubActor::new_from_player_state(&state.player_states[0]),
        SubActor::new_from_player_state(&state.player_states[1]),
        SubActor::new_from_player_state(&state.player_states[2]),
        SubActor::new_from_player_state(&state.player_states[3]),
    ];

    let mut moves = state.current_moves.clone();
    let mut current_pidx = state.current_pidx;
    loop {
        moves.push(Move(current_pidx, actors[current_pidx].play_card(&moves)));

        if moves.len() == 4 {
            current_pidx = crate::internal::round::find_winner_pidx(&moves);
            actors.iter_mut().for_each(|actor| actor.end_round(current_pidx, &moves));
            moves = vec![];
            if actors[0].player_state.cards().len() == 0 { return actors[0].player_state.final_scores(); }
        } else {
            current_pidx = (current_pidx + 1) % 4;
        }
    }
}

fn mcts_rec<SubActor: MediasResActor<DefaultPlayerState>>(root: &mut Node, game_state: &mut GameState<DefaultPlayerState>) -> isize {
    let visits_ln = (root.visits as f32).ln();

    let played_moves = &game_state.current_moves;
    let player_state = &game_state.player_states[game_state.current_pidx];

    let possible_cards = if let Some(first_move) = played_moves.first() {
        let same_suit_cards: Vec<Card> = player_state.cards().iter().filter(|c| c.suit() == first_move.card().suit()).map(|c| *c).collect();
        if same_suit_cards.len() > 0 {
            same_suit_cards
        } else {
            let non_scoring_cards: Vec<Card> = player_state.cards().iter().filter(|c| c.score() == 0).map(|c| *c).collect();
            if player_state.first_round() && non_scoring_cards.len() > 0 {
                non_scoring_cards
            } else {
                player_state.cards().clone()
            }
        }
    } else {
        if player_state.first_round() {
            vec![Card(Rank::Two, Suit::Clubs)]
        } else {
            let non_heart_cards: Vec<Card> = player_state.cards().iter().filter(|c| c.suit() != Suit::Hearts).map(|c| *c).collect();
            if !player_state.hearts_played() && non_heart_cards.len() > 0 {
                non_heart_cards
            } else {
                player_state.cards().clone()
            }
        }
    };

    root.ensure_children();

    let mut best_value = 0.;
    let mut best_card = possible_cards.first().unwrap();
    let mut was_zero = false;
    for possible_card in &possible_cards {
        if root.children().len() != player_state.cards().len() {
            println!("{} {}", root.children().len(), player_state.cards().len());
        }
        let child_node = root.children().get(possible_card).unwrap();
        if child_node.visits == 0 {
            best_card = possible_card;
            was_zero = true;
            break;
        } else if child_node.search_weight(visits_ln) > best_value {
            best_value = child_node.search_weight(visits_ln);
            best_card = possible_card;
        }
    }
    let best_node = root.children_mut().get_mut(best_card).unwrap();

    let own_pidx = game_state.current_pidx;
    let result = play_for_a_bit::<SubActor>(game_state, own_pidx, *best_card);

    let result = if was_zero {
        let inner_result = if result == None { play_randomly::<SubActor>(game_state) } else { result.unwrap() };
        inner_result[own_pidx]
    } else {
        if result == None { mcts_rec::<SubActor>(best_node, game_state) } else { result.unwrap()[own_pidx] }
    };

    best_node.update(result);
    result
}

pub fn mcts_mod<SubActor: MediasResActor<DefaultPlayerState>, IncomingPlayerState: DefaultPlayerStateInterface>(pidx: usize, player_state: &mut IncomingPlayerState, played_moves: &Vec<Move>, time: usize) -> Vec<(Card, f32, usize)> {
    let mut root = Node::new(
        player_state.cards().clone(),
    );

    let start_time = SystemTime::now();
    while SystemTime::now().duration_since(start_time).unwrap().as_millis() < time as u128 {
        let mut game_state = determinize(pidx, player_state, played_moves);
        let result = mcts_rec::<SubActor>(&mut root, &mut game_state);
        root.update(result);
    }

    root.children.iter().map(|(c, node)| (*c, node.value, node.visits)).collect()
}
