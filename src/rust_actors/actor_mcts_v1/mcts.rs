use crate::rust_actors::actor_mcts_v1::game_state::GameState;
use crate::shared::data::Move;
use crate::shared::actor::Actor;
use crate::util::non_nan::NonNan;

use std::time::SystemTime;
use crate::{Card, Rank, ActorRuleV1, Suit, ExtendedPlayerState, DefaultPlayerState};
use crate::rust_actors::player_state::DefaultPlayerStateInterface;
use crate::rust_actors::player_state::BasicPlayerStateInterface;
use crate::rust_actors::player_state::MediasResActor;

struct Node<PlayerState : DefaultPlayerStateInterface> {
    visits: usize,
    value: f32,
    children: Vec<Node<PlayerState>>,
    children_left: isize,
    last_move: Option<Move>,
    state: GameState<PlayerState>,
    result: Option<[isize; 4]>
}

impl<PlayerState : DefaultPlayerStateInterface> Node<PlayerState> {
    pub fn new(state: GameState<PlayerState>, last_move: Option<Move>, result: Option<[isize; 4]>) -> Node<PlayerState> {
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

    pub fn children(&self) -> &Vec<Node<PlayerState>> {
        &self.children
    }

    pub fn children_mut(&mut self) -> &mut Vec<Node<PlayerState>> {
        &mut self.children
    }

    pub fn state_mut(&mut self) -> &mut GameState<PlayerState> {
        &mut self.state
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
            self.children = initial_vec(self.state_mut());
            self.children_left = self.children.len() as isize;
        }

        self.children_left -= 1;
        self.children.len() - self.children_left as usize - 1
    }

    pub fn update(&mut self, result: [isize; 4]) {
        self.visits += 1;
        self.value += (36 - result[(self.state.current_pidx + 3) % 4]) as f32 / 46.;
    }
}

fn initial_vec<PlayerState : DefaultPlayerStateInterface>(game_state: &GameState<PlayerState>) -> Vec<Node<PlayerState>> {
    if game_state.player_states[game_state.current_pidx].cards().len() == 0 { return vec![]; }

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

    possible_cards.iter()
                  .map(|card| {
                      let mut new_player_states = game_state.player_states.clone();
                      new_player_states[game_state.current_pidx].update_did_play_card(card);
                      let mut new_played_moves = played_moves.clone();
                      new_played_moves.push(Move(game_state.current_pidx, *card));

                      if new_played_moves.len() == 4 {
                          let winning_pidx = crate::internal::round::find_winner_pidx(&new_played_moves);
                          new_player_states.iter_mut().for_each(|s| s.update_end_round(&new_played_moves, winning_pidx));
                          let result = if new_player_states[game_state.current_pidx].cards().len() == 0 {
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

fn play_randomly(state: &GameState<DefaultPlayerState>) -> [isize; 4] {
    let mut actors = [
        ActorRuleV1::<ExtendedPlayerState>::new_from_player_state(&state.player_states[0]),
        ActorRuleV1::<ExtendedPlayerState>::new_from_player_state(&state.player_states[1]),
        ActorRuleV1::<ExtendedPlayerState>::new_from_player_state(&state.player_states[2]),
        ActorRuleV1::<ExtendedPlayerState>::new_from_player_state(&state.player_states[3]),
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

fn mcts_rec(root: &mut Node<DefaultPlayerState>) -> [isize; 4] {
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

pub fn mcts(game_state: &mut GameState<DefaultPlayerState>, time: usize) -> Vec<(Card, f32, usize)> {
    let mut root = Node::new(
        game_state.clone(),
        None,
        None,
    );

    let start_time = SystemTime::now();
    while SystemTime::now().duration_since(start_time).unwrap().as_millis() < time as u128 {
        let result = mcts_rec(&mut root);
        root.update(result);
    }

    root.children.iter().map(|node| (node.last_move.unwrap().card(), node.value, node.visits)).collect()
}
