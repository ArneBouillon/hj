use crate::{Card, DefaultPlayerState, PassDirection, Rank, Suit};
use crate::game::actor::Actor;
use crate::game::data::Move;
use crate::util::non_nan::NonNan;
use itertools::Itertools;
use itertools::FoldWhile::{Continue, Done};
use std::marker::PhantomData;
use std::collections::HashMap;
use crate::rust_actors::determinize::{Determinize, determinize_v1};
use crate::rust_actors::eval_state::EvalState;
use crate::rust_actors::player_state::{ExtendedPlayerStateInterface, MediasResActor};

mod mcts;
mod mcts_mod;

pub struct ActorMCTSV1<
    D: Determinize,
    ES: EvalState,
    S: MediasResActor<DefaultPlayerState>,
    PS: ExtendedPlayerStateInterface
> {
    player_state: PS,

    newtype: bool,
    timeout: usize,
    tries: usize,

    determinize_type: PhantomData<D>,
    eval_state_type: PhantomData<ES>,
    sub_actor_type: PhantomData<S>,
}

impl<
    D: Determinize,
    ES: EvalState,
    S: MediasResActor<DefaultPlayerState>,
    PS: ExtendedPlayerStateInterface
> ActorMCTSV1<D, ES, S, PS> {
    #[allow(dead_code)]
    pub fn new(newtype: bool, timeout: usize, tries: usize) -> Self {
        Self {
            player_state: Default::default(),

            newtype,
            timeout,
            tries,

            determinize_type: PhantomData,
            eval_state_type: PhantomData,
            sub_actor_type: PhantomData,
        }
    }
}

impl<
    D: Determinize,
    ES: EvalState,
    S: MediasResActor<DefaultPlayerState>,
    PS: ExtendedPlayerStateInterface
> ActorMCTSV1<D, ES, S, PS> {
    fn add_ghost_cards(cards: &[Vec<Card>; 4], spade_card: Card, club_card: Card, heart_card: Card) -> [Vec<Card>; 4] {
        [
            cards[0].clone().into_iter().chain([spade_card].iter().copied()).collect(),
            cards[1].clone().into_iter().chain([club_card].iter().copied()).collect(),
            cards[2].clone(),
            cards[3].clone().into_iter().chain([heart_card].iter().copied()).collect(),
        ]
    }

    fn divide_three() -> Vec<[usize; 4]> {
        itertools::iproduct!((0..3), (0..3), (0..3), (0..3))
            .map(|nums| [nums.0, nums.1, nums.2, nums.3])
            .filter(|nums| nums.iter().sum::<usize>() == 3)
            .collect()
    }

    pub fn choose_three_to_pass(&self) -> Vec<Card> {
        let mut by_suit = [vec![], vec![], vec![], vec![]];
        let mut by_suit_counts = [0, 0, 0, 0];
        for card in self.player_state.cards() {
            by_suit[card.suit().to_index()].push(card);
            by_suit_counts[card.suit().to_index()] += 1;
        }
        for sidx in 0..3 {
            by_suit[sidx].sort_by_key(|c| match c {
                Card(Rank::Two, Suit::Clubs) => usize::MAX,
                _ => c.rank() as usize,
            });
        }

        let spade_card = if by_suit[0].contains(&&Card(Rank::Queen, Suit::Spades)) {
            Card(num::FromPrimitive::from_u8(by_suit[0].iter().rev().fold_while(14, |acc, item|
                if item.rank() as u8 == acc { Continue(acc - 1) } else { Done(acc) }
            ).into_inner()).unwrap(), Suit::Spades)
        } else { Card(Rank::Queen, Suit::Spades) };
        let club_card = Card(num::FromPrimitive::from_u8(by_suit[1].iter().rev().fold_while(14, |acc, item|
            if item.rank() as u8 == acc { Continue(acc - 1) } else { Done(acc) }
        ).into_inner()).unwrap(), Suit::Clubs);
        let heart_card = Card(num::FromPrimitive::from_u8(by_suit[3].iter().rev().fold_while(14, |acc, item|
            if item.rank() as u8 == acc { Continue(acc - 1) } else { Done(acc) }
        ).into_inner()).unwrap(), Suit::Hearts);

        let best_option: Vec<Card> = Self::divide_three().iter().filter_map(|division| {
            if division.iter().zip(by_suit_counts).any(|(d, c)| *d > c) {
                None
            } else {
                Some([
                    by_suit[0].clone().iter().take(by_suit[0].len() - division[0]).map(|c| **c).collect(),
                    by_suit[1].clone().iter().take(by_suit[1].len() - division[1]).map(|c| **c).collect(),
                    by_suit[2].clone().iter().take(by_suit[2].len() - division[2]).map(|c| **c).collect(),
                    by_suit[3].clone().iter().take(by_suit[3].len() - division[3]).map(|c| **c).collect(),
                ])
            }
        }).min_by_key(|cards|
            NonNan::new(
                0.2 * ES::evaluate_state(&self.player_state, &Self::add_ghost_cards(cards, spade_card, club_card, heart_card)).value()
                    + 0.8 * ES::evaluate_state(&self.player_state, &cards).value()
            ).unwrap()
        ).expect("There should always be valid passing options").into_iter().flatten().collect();

        let mut result = self.player_state.cards().clone();
        result.retain(|c| !best_option.contains(&c));
        result
    }
}

impl<
    D: Determinize,
    ES: EvalState,
    S: MediasResActor<DefaultPlayerState>,
    PS: ExtendedPlayerStateInterface
> Actor for ActorMCTSV1<D, ES, S, PS> {
    fn initialize(&mut self, pidx: usize, cards: &Vec<Card>) {
        self.player_state.set_pidx(pidx);
        self.player_state.set_cards(cards.clone());
    }

    fn play_card(&mut self, played_moves: &Vec<Move>) -> Card {
        self.player_state.update_play_card(played_moves);

        let best_card = if self.newtype {
            mcts_mod::mcts_mod::<D, S, PS>(self.player_state.pidx(), &mut self.player_state, played_moves, self.timeout).into_iter().max_by_key(|(_, value, visits)| {
                NonNan::new(if *visits == 0 { 0. } else { value / *visits as f32 }).unwrap()
            }).unwrap().0
        } else {
            (0..self.tries).map(|_| {
                let (game_info, player_states) = D::determinize(self.player_state.pidx(), &self.player_state, played_moves);
                mcts::mcts(&game_info, &player_states, self.timeout)
            }).fold(HashMap::<Card, (f32, usize)>::new(), |mut acc, item| {
                item.iter().for_each(|tup|
                    match acc.get_mut(&tup.0) {
                        Some(entry) => { entry.0 += tup.1; entry.1 += tup.2; },
                        None => { acc.insert(tup.0, (tup.1, tup.2)); },
                    }
                );
                acc
            }).into_iter().max_by_key(|(_, (value, visits))| {
                NonNan::new(if *visits == 0 { 0. } else { value / *visits as f32 }).unwrap()
            }).unwrap().0
        };

        self.player_state.update_did_play_card(&best_card);

        best_card
    }

    fn end_round(&mut self, winner_pidx: usize, played_moves: &Vec<Move>) {
        self.player_state.update_end_round(played_moves, winner_pidx);
    }

    fn end_game(&mut self, _score: [isize; 4]) {}

    fn get_pass(&mut self, direction: PassDirection) -> Vec<Card> {
        if direction == PassDirection::None { return vec![]; }

        let passed_cards = self.choose_three_to_pass();
        for passed_card in &passed_cards { self.player_state.update_did_play_card(passed_card); }
        passed_cards
    }

    fn end_pass(&mut self, passed_cards: &Vec<Card>) {
        self.player_state.update_end_pass(passed_cards);
    }
}
