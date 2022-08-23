use crate::rust_actors::mcts_actor_v1::game_state::GameState;
use crate::shared::data::Move;
use crate::shared::actor::Actor;
use crate::util::non_nan::NonNan;

use std::time::SystemTime;
use crate::{Card, HumanActor, Rank, RuleActorV1, Suit};
use crate::rust_actors::shared::MediasResActor;

pub struct Node {
    visits: usize,
    value: f32,
    children: Vec<Node>,
    children_left: isize,
    last_move: Option<Move>,
    state: GameState,
    result: Option<[isize; 4]>
}

impl Node {
    pub fn new(state: GameState, last_move: Option<Move>, result: Option<[isize; 4]>) -> Node {
        Node {
            visits: 0,
            value: 0.,
            children: Vec::new(),
            children_left: -1,
            last_move,
            state,
            result,
        }
    }

    pub fn children(&self) -> &Vec<Node> {
        &self.children
    }

    pub fn children_mut(&mut self) -> &mut Vec<Node> {
        &mut self.children
    }

    pub fn state(&self) -> &GameState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut GameState {
        &mut self.state
    }

    pub fn best_child(&self, visits: usize) -> usize {
        let visits_ln = (visits as f32).ln();

        let (index, _) = self.children().iter().enumerate().max_by_key(|(_, node)| {
            NonNan::new(node.search_weight(visits_ln)).unwrap()
        }).unwrap();

        index
    }

    pub fn best_card(&self) -> (Card, f32) {
        let node = self.children().iter().max_by_key(|node| {
            NonNan::new(node.weight()).unwrap()
        }).unwrap();

        (
            node.last_move.unwrap().card(),
            node.weight(),
        )
    }

    pub fn weight(&self) -> f32 {
        if self.visits == 0 { 0. } else { self.value as f32 / self.visits as f32 }
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
            self.children = initial_vec(self.state_mut());
            self.children_left = self.children.len() as isize;
        }

        self.children_left -= 1;
        self.children.len() - self.children_left as usize - 1
    }

    pub fn update(&mut self, result: [isize; 4]) {
        self.visits += 1;
        self.value += (36 - result[(self.state.current_pidx + 3) % 4]) as f32 / 46.0;
    }

    pub fn max_depth(&self) -> usize {
        match self.children.len() {
            0 => 1,
            _ => 1 + self.children.iter().map(|c| c.max_depth()).max().unwrap(),
        }
    }
}

fn initial_vec(game_state: &GameState) -> Vec<Node> {
    if game_state.player_states[game_state.current_pidx].cards.len() == 0 { return vec![]; }

    let played_moves = &game_state.current_moves;
    let player_state = &game_state.player_states[game_state.current_pidx];

    let possible_cards = if let Some(first_move) = played_moves.first() {
        let same_suit_cards: Vec<Card> = player_state.cards.iter().filter(|c| c.suit() == first_move.card().suit()).map(|c| *c).collect();
        if same_suit_cards.len() > 0 {
            same_suit_cards
        } else {
            let non_scoring_cards: Vec<Card> = player_state.cards.iter().filter(|c| c.score() == 0).map(|c| *c).collect();
            if player_state.first_round && non_scoring_cards.len() > 0 {
                non_scoring_cards
            } else {
                player_state.cards.clone()
            }
        }
    } else {
        if player_state.first_round {
            vec![Card(Rank::Two, Suit::Clubs)]
        } else {
            let non_heart_cards: Vec<Card> = player_state.cards.iter().filter(|c| c.suit() != Suit::Hearts).map(|c| *c).collect();
            if !player_state.hearts_played && non_heart_cards.len() > 0 {
                non_heart_cards
            } else {
                player_state.cards.clone()
            }
        }
    };

    possible_cards.iter()
                  .map(|card| {
                      let mut new_player_states = game_state.player_states.clone();
                      new_player_states[game_state.current_pidx].update_did_play_card(card);
                      let mut new_played_moves = played_moves.clone();
                      new_played_moves.push(Move(game_state.current_pidx, *card));

                      if new_played_moves.len() == 4 {
                          let winning_pidx = crate::internal::round::find_winner_pidx(&new_played_moves);
                          new_player_states.iter_mut().for_each(|s| s.update_end_round(&new_played_moves, winning_pidx));
                          let result = if new_player_states[game_state.current_pidx].cards.len() == 0 {
                              Some(new_player_states[game_state.current_pidx].final_scores())
                          } else { None };
                          let new_game_state = GameState {
                              current_moves: vec![],
                              current_pidx: winning_pidx,
                              player_states: new_player_states,
                          };

                          Node::new(new_game_state, Some(Move(game_state.current_pidx, *card)), result)
                      } else {
                          let next_pidx = (game_state.current_pidx + 1) % 4;
                          new_player_states[next_pidx].update_play_card(&new_played_moves);
                          let new_game_state = GameState {
                              current_moves: new_played_moves,
                              current_pidx: next_pidx,
                              player_states: new_player_states,
                          };

                          Node::new(new_game_state, Some(Move(game_state.current_pidx, *card)), None)
                      }
                  }).collect()
}

fn play_randomly(state: &GameState) -> [isize; 4] {
    let mut actors = [
        RuleActorV1::new_from_player_state(0, &state.player_states[0]),
        RuleActorV1::new_from_player_state(1, &state.player_states[1]),
        RuleActorV1::new_from_player_state(2, &state.player_states[2]),
        RuleActorV1::new_from_player_state(3, &state.player_states[3]),
    ];

    // for m in &state.current_moves { println!("{}", m.pidx()); }
    // println!("{}, {}, {}, {}", actors[0].player_state.cards.len(), actors[1].player_state.cards.len(), actors[2].player_state.cards.len(), actors[3].player_state.cards.len());
    //
    // HumanActor::pause();

    let mut moves = state.current_moves.clone();
    let mut current_pidx = state.current_pidx;
    loop {
        moves.push(Move(current_pidx, actors[current_pidx].play_card(&moves)));

        if moves.len() == 4 {
            current_pidx = crate::internal::round::find_winner_pidx(&moves);
            actors.iter_mut().for_each(|actor| actor.end_round(current_pidx, &moves));
            moves = vec![];
            if actors[0].player_state.cards.len() == 0 { return actors[0].player_state.final_scores(); }
        } else {
            current_pidx = (current_pidx + 1) % 4;
        }
    }
}

pub fn mcts_rec(root: &mut Node) -> [isize; 4] {
    if root.fully_expanded() {
        let index = root.best_child(root.visits);
        let best_child = root.children_mut().get_mut(index).unwrap();

        let result = match best_child.result {
            Some(game_result) => game_result,
            None => mcts_rec(best_child),
        };

        best_child.update(result);

        result
    } else {
        let index = root.expand();
        let new_child = root.children_mut().get_mut(index).unwrap();

        let result = match new_child.result {
            Some(game_result) => game_result,
            None => play_randomly(root.state_mut()),
        };

        let new_child = root.children_mut().get_mut(index).unwrap();
        new_child.update(result);

        result
    }
}

pub fn mcts(game_state: &mut GameState, time: usize) -> Vec<(Card, f32, usize)> {
    let mut root = Node::new(
        game_state.clone(),
        None,
        None,
    );

    let start_time = SystemTime::now();
    let mut count: usize = 0;
    while SystemTime::now().duration_since(start_time).unwrap().as_millis() < time as u128 {
        let result = mcts_rec(&mut root);
        // println!("{:?}", result);
        // HumanActor::pause();
        root.update(result);
        count += 1;
    }

    root.children.iter().map(|node| (node.last_move.unwrap().card(), node.value, node.visits)).collect()

    // // println!("Number of simulations: {}", count);
    // // println!("Test: {}", root.value);
    // let (best_card, weight) = root.best_card();
    // // println!("Expected result: {}", weight);
    //
    // // println!("Number of simulations: {}", count);
    // // println!("Max depth: {}", root.max_depth());
    //
    // best_card
}
