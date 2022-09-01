use crate::game::data::Move;
use crate::game::actor::Actor;
use crate::util::non_nan::NonNan;

use std::time::SystemTime;
use crate::{Card, Rank, ActorRuleV1, Suit, ExtendedPlayerState, DefaultPlayerState, EvalStateV1, EvalRoundV1};
use crate::game::game_info::{GameInfo, StopCondition};
use crate::rust_actors::actor_dummy::ActorDummy;
use crate::rust_actors::player_state::DefaultPlayerStateInterface;
use crate::rust_actors::player_state::BasicPlayerStateInterface;
use crate::rust_actors::player_state::MediasResActor;
use crate::rust_actors::util;
use crate::util::deck::find_winner_pidx;

struct Node {
    visits: usize,
    value: f32,
    children: Vec<Node>,
    children_left: isize,
    game_info: GameInfo,
    last_move: Option<Move>,
    player_states: [DefaultPlayerState; 4],
    result: Option<[isize; 4]>,
}

impl Node {
    pub fn new(game_info: GameInfo, last_move: Option<Move>, player_states: [DefaultPlayerState; 4], result: Option<[isize; 4]>) -> Node {
        Node {
            visits: 0,
            value: 0.,
            children: Vec::new(),
            children_left: -1,
            game_info,
            last_move,
            player_states,
            result,
        }
    }

    pub fn children(&self) -> &Vec<Node> {
        &self.children
    }

    pub fn children_mut(&mut self) -> &mut Vec<Node> {
        &mut self.children
    }

    pub fn game_info_mut(&mut self) -> &mut GameInfo {
        &mut self.game_info
    }

    pub fn best_child(&self, visits: usize) -> usize {
        let visits_ln = (visits as f32).ln();

        let (index, _) = self.children().iter().enumerate().max_by_key(|(_, node)| {
            NonNan::new(node.search_weight(visits_ln)).unwrap()
        }).unwrap();

        index
    }

    pub fn search_weight(&self, parent_visits_ln: f32) -> f32 {
        if self.visits == 0 {
            0.
        } else {
            (self.value as f32 / self.visits as f32) + (2. * parent_visits_ln / self.visits as f32).sqrt()
        }
    }

    #[inline]
    pub fn fully_expanded(&self) -> bool {
        self.children_left == 0
    }

    pub fn expand(&mut self) -> usize {
        if self.children_left == -1 {
            self.children = initial_vec(&self.game_info, &self.player_states);
            self.children_left = self.children.len() as isize;
        }

        self.children_left -= 1;
        self.children.len() - self.children_left as usize - 1
    }

    pub fn update(&mut self, result: [isize; 4]) {
        self.visits += 1;
        self.value += (36 - result[(self.game_info.current_pidx() + 3) % 4]) as f32 / 46.;
    }
}

fn initial_vec(game_info: &GameInfo, player_states: &[DefaultPlayerState; 4]) -> Vec<Node> {
    if game_info.hands()[game_info.current_pidx()].cards().len() == 0 { return vec![]; }

    let possible_cards = util::get_allowed_cards(game_info);
    possible_cards.iter()
                  .map(|card| {
                      let mut actors = [0, 1, 2, 3].map(|pidx|
                          ActorDummy::new(
                              player_states[pidx].clone(),
                              if game_info.current_pidx() == pidx { Some(*card) } else { None },
                          )
                      );
                      let mut new_game_info = game_info.clone();
                      new_game_info.play_without_validator(&mut actors.iter_mut().collect::<Vec<&mut ActorDummy<_>>>().try_into().unwrap(), StopCondition::OneMove);

                      let result = new_game_info.result();
                      Node::new(
                          new_game_info,
                          Some(Move(game_info.current_pidx(), *card)),
                          actors.map(|a| a.player_state_move()),
                          result,
                      )
                  }).collect()
}

fn play_randomly<S: MediasResActor<DefaultPlayerState>>(root: &Node) -> [isize; 4] {
    let mut game_info = root.game_info.clone();
    let mut actors = [
        &mut S::new_from_player_state(&root.player_states[0]),
        &mut S::new_from_player_state(&root.player_states[1]),
        &mut S::new_from_player_state(&root.player_states[2]),
        &mut S::new_from_player_state(&root.player_states[3]),
    ];
    game_info.play_without_validator(&mut actors, StopCondition::None);
    game_info.result().unwrap()
}

fn mcts_rec<S: MediasResActor<DefaultPlayerState>>(root: &mut Node) -> [isize; 4] {
    if root.fully_expanded() {
        let index = root.best_child(root.visits);
        let best_child = root.children_mut().get_mut(index).unwrap();

        let result = match best_child.result {
            Some(game_result) => game_result,
            None => mcts_rec::<S>(best_child),
        };

        best_child.update(result);

        result
    } else {
        let index = root.expand();
        let new_child = root.children_mut().get_mut(index).unwrap();

        let result = match new_child.result {
            Some(game_result) => game_result,
            None => play_randomly::<S>(root),
        };

        let new_child = root.children_mut().get_mut(index).unwrap();
        new_child.update(result);

        result
    }
}

pub fn mcts<S: MediasResActor<DefaultPlayerState>>(
    game_info: &GameInfo,
    player_states: &[DefaultPlayerState; 4],
    time: usize,
) -> Vec<(Card, f32, usize)> {
    let mut root = Node::new(
        game_info.clone(),
        None,
        player_states.clone(),
        None,
    );

    let start_time = SystemTime::now();
    while SystemTime::now().duration_since(start_time).unwrap().as_millis() < time as u128 {
        let result = mcts_rec::<S>(&mut root);
        root.update(result);
    }

    root.children.iter().map(|node| (node.last_move.unwrap().card(), node.value, node.visits)).collect()
}
