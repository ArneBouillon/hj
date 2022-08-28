use crate::game::data::Move;
use crate::game::actor::Actor;
use crate::util::deck::find_winner_pidx;

use std::time::SystemTime;
use std::collections::HashMap;

use crate::{Card, Rank, ActorRuleV1, Suit, ExtendedPlayerState, DefaultPlayerState};
use crate::game::game_info::{GameInfo, StopCondition};
use crate::rust_actors::actor_dummy::ActorDummy;
use crate::rust_actors::actor_mcts_v1::determinize::determinize;
use crate::rust_actors::player_state::DefaultPlayerStateInterface;
use crate::rust_actors::player_state::BasicPlayerStateInterface;
use crate::rust_actors::player_state::MediasResActor;
use crate::rust_actors::util::get_allowed_cards;

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

fn play_for_a_bit<SubActor: MediasResActor<DefaultPlayerState>>(game_info: &mut GameInfo, actors: &mut [&mut SubActor; 4], special_pidx: usize, special_card: Card) -> Option<[isize; 4]> {
    actors[special_pidx].add_dummy(special_card);
    game_info.play_without_validator(actors, StopCondition::Until(special_pidx));
    game_info.result()
}

fn play_randomly<SubActor: MediasResActor<DefaultPlayerState>>(game_info: &mut GameInfo, actors: &mut [&mut SubActor; 4], own_pidx: usize) -> [isize; 4] {
    game_info.play_without_validator(actors, StopCondition::None);
    game_info.result().unwrap()
}

fn mcts_rec<SubActor: MediasResActor<DefaultPlayerState>>(root: &mut Node, game_info: &mut GameInfo, actors: &mut [&mut SubActor; 4]) -> isize {
    let visits_ln = (root.visits as f32).ln();
    let possible_cards = get_allowed_cards(game_info);
    let own_pidx = game_info.current_pidx();

    root.ensure_children();

    let mut best_value = 0.;
    let mut best_card = possible_cards.first().unwrap();
    let mut was_zero = false;
    for possible_card in &possible_cards {
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

    let result = play_for_a_bit::<SubActor>(game_info, actors, own_pidx, *best_card);

    let result = if was_zero {
        let inner_result = if result == None { play_randomly::<SubActor>(game_info, actors, own_pidx) } else { result.unwrap() };
        inner_result[own_pidx]
    } else {
        if result == None { mcts_rec::<SubActor>(best_node, game_info, actors) } else { result.unwrap()[own_pidx] }
    };

    best_node.update(result);
    result
}

pub fn mcts_mod<SubActor: MediasResActor<DefaultPlayerState>, IncomingPlayerState: DefaultPlayerStateInterface>(pidx: usize, player_state: &IncomingPlayerState, played_moves: &Vec<Move>, time: usize) -> Vec<(Card, f32, usize)> {
    let mut root = Node::new(
        player_state.cards().clone(),
    );

    let start_time = SystemTime::now();
    while SystemTime::now().duration_since(start_time).unwrap().as_millis() < time as u128 {
        let (mut game_info, mut player_states) = determinize(pidx, player_state, played_moves);
        let mut actors = [
            &mut SubActor::new_from_player_state(&player_states[0]),
            &mut SubActor::new_from_player_state(&player_states[1]),
            &mut SubActor::new_from_player_state(&player_states[2]),
            &mut SubActor::new_from_player_state(&player_states[3]),
        ];
        let result = mcts_rec::<SubActor>(&mut root, &mut game_info, &mut actors);
        root.update(result);
    }

    root.children.iter().map(|(c, node)| (*c, node.value, node.visits)).collect()
}
