use iter_fixed::IntoIteratorFixed;
use crate::{ExtendedPlayerState, PassDirection};
use crate::game::actor::Actor;
use crate::game::data::{Card, Move, Rank, Suit};
use crate::util::non_nan::NonNan;

use itertools::Itertools;
use itertools::FoldWhile::{Continue, Done};
use crate::rust_actors::player_state::{DefaultPlayerStateInterface, ExtendedPlayerStateInterface, MediasResActor};

mod eval_round;
mod eval_state;

pub struct ActorRuleV1<PlayerState : ExtendedPlayerStateInterface> {
    dummy: Option<Card>,
    pub player_state: PlayerState,
}


impl<PlayerState : ExtendedPlayerStateInterface> ActorRuleV1<PlayerState> {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            dummy: None,
            player_state: Default::default(),
        }
    }
}

impl<PlayerState : ExtendedPlayerStateInterface> ActorRuleV1<PlayerState> {
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
                0.2 * self.evaluate_state(&Self::add_ghost_cards(cards, spade_card, club_card, heart_card)).value()
                    + 0.8 * self.evaluate_state(&cards).value()
            ).unwrap()
        ).expect("There should always be valid passing options").into_iter().flatten().collect();

        let mut result = self.player_state.cards().clone();
        result.retain(|c| !best_option.contains(&c));
        result
    }
}

impl<PlayerState : ExtendedPlayerStateInterface> Actor for ActorRuleV1<PlayerState> {
    fn initialize(&mut self, pidx: usize, cards: &Vec<Card>) {
        self.player_state.set_pidx(pidx);
        self.player_state.set_cards(cards.clone());
    }

    fn play_card(&mut self, played_moves: &Vec<Move>) -> Card {
        self.player_state.update_play_card(played_moves);

        let best_card = if let Some(card) = self.dummy {
            self.dummy = None;
            card
        } else {
            let possible_cards = if let Some(first_move) = played_moves.first() {
                let same_suit_cards: Vec<Card> = self.player_state.cards().iter().filter(|c| c.suit() == first_move.card().suit()).map(|c| *c).collect();
                if same_suit_cards.len() > 0 {
                    same_suit_cards
                } else {
                    let non_scoring_cards: Vec<Card> = self.player_state.cards().iter().filter(|c| c.score() == 0).map(|c| *c).collect();
                    if self.player_state.first_round() && non_scoring_cards.len() > 0 {
                        non_scoring_cards
                    } else {
                        self.player_state.cards().clone()
                    }
                }
            } else {
                if self.player_state.first_round() {
                    vec![Card(Rank::Two, Suit::Clubs)]
                } else {
                    let non_heart_cards: Vec<Card> = self.player_state.cards().iter().filter(|c| c.suit() != Suit::Hearts).map(|c| *c).collect();
                    if !self.player_state.hearts_played() && non_heart_cards.len() > 0 {
                        non_heart_cards
                    } else {
                        self.player_state.cards().clone()
                    }
                }
            };

            *possible_cards.iter().min_by_key(|c| {
                let mut by_suit = [vec![], vec![], vec![], vec![]];
                for card in self.player_state.cards() { if card != *c { by_suit[card.suit() as usize - 1].push(*card); } }
                self.evaluate_round(played_moves, **c).try_add(self.evaluate_state(&by_suit)).unwrap()
            }).expect("There should always be valid playing options")
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

impl<PlayerState : DefaultPlayerStateInterface> MediasResActor<PlayerState> for ActorRuleV1<ExtendedPlayerState> {
    fn new_from_player_state(player_state: &PlayerState) -> Self {
        let cards_in_game_by_suit = player_state.cards_in_game().into_iter_fixed().map(|a| a.iter().filter(|b| **b).count()).collect();
        let mut opponent_cards_in_game = player_state.cards_in_game().clone();
        for card in player_state.cards() { opponent_cards_in_game[card.suit().to_index()][card.rank().to_index()] = false; }
        let opponent_cards_in_game_by_suit = opponent_cards_in_game.into_iter_fixed().map(|a| a.iter().filter(|b| **b).count()).collect();

        Self {
            dummy: None,
            player_state: ExtendedPlayerState::new(player_state.cards().clone(), player_state.first_round(), player_state.hearts_played(), player_state.pidx(), player_state.cards_in_game().clone(), player_state.scores().clone(), player_state.scored().clone(), player_state.still_has().clone(), cards_in_game_by_suit, opponent_cards_in_game, opponent_cards_in_game_by_suit),
        }
    }

    fn add_dummy(&mut self, card: Card) {
        self.dummy = Some(card);
    }
}
