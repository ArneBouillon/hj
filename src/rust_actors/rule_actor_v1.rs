use iter_fixed::IntoIteratorFixed;
use crate::PassDirection;
use crate::shared::actor::Actor;
use crate::shared::data::{Card, Move, Rank, Suit};
use crate::util::non_nan::NonNan;

use itertools::Itertools;
use itertools::FoldWhile::{Continue, Done};
use crate::internal::game::play;
use crate::rust_actors::shared::{MediasResActor, PlayerState};

mod eval_round;
mod eval_state;

pub struct RuleActorV1 {
    cards_in_game_by_suit: [usize; 4],
    opponent_cards_in_game: [[bool; 13]; 4],
    opponent_cards_in_game_by_suit: [usize; 4],
    pidx: usize,
    pub player_state: PlayerState,
}


impl RuleActorV1 {
    pub fn new() -> Self {
        Self {
            cards_in_game_by_suit: [13; 4],
            opponent_cards_in_game: [[true; 13]; 4],
            opponent_cards_in_game_by_suit: [13; 4],
            pidx: 0,
            player_state: Default::default(),
        }
    }
}

impl RuleActorV1 {
    fn add_ghost_cards(cards: &[Vec<Card>; 4], spade_card: Card, club_card: Card, heart_card: Card) -> [Vec<Card>; 4] {
        [
            cards[0].clone().into_iter().chain([spade_card].iter().copied()).collect(),
            cards[1].clone().into_iter().chain([club_card].iter().copied()).collect(),
            cards[2].clone(),
            cards[3].clone().into_iter().chain([heart_card].iter().copied()).collect(),
        ]

        // [
        //     cards[0].clone(),
        //     cards[1].clone(),
        //     cards[2].clone(),
        //     cards[3].clone(),
        // ]
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
        for card in &self.player_state.cards {
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

        let best_option: Vec<Card> = RuleActorV1::divide_three().iter().filter_map(|division| {
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
                0.2 * self.evaluate_state(&RuleActorV1::add_ghost_cards(cards, spade_card, club_card, heart_card)).value()
                    + 0.8 * self.evaluate_state(&cards).value()
            ).unwrap()
        ).expect("There should always be valid passing options").into_iter().flatten().collect();

        let mut result = self.player_state.cards.clone();
        result.retain(|c| !best_option.contains(&c));
        result
    }
}

impl RuleActorV1 {
    fn process_moves(&mut self, moves: &Vec<Move>) {
        if let Some(Move(_, Card(_, first_suit))) = moves.first() {
            for Move(pidx, Card(rank, suit)) in moves {
                if *pidx != self.pidx {
                    if self.opponent_cards_in_game[suit.to_index()][rank.to_index()] { self.opponent_cards_in_game_by_suit[suit.to_index()] -= 1; }
                    self.opponent_cards_in_game[suit.to_index()][rank.to_index()] = false;
                }

                if self.player_state.cards_in_game[suit.to_index()][rank.to_index()] { self.cards_in_game_by_suit[suit.to_index()] -= 1; }
                self.player_state.cards_in_game[suit.to_index()][rank.to_index()] = false;
                if suit != first_suit { self.player_state.still_has[suit.to_index()][*pidx] = false; }
            }
        }
    }
}

impl Actor for RuleActorV1 {
    fn initialize(&mut self, pidx: usize, cards: &Vec<Card>) {
        self.pidx = pidx;
        self.player_state.cards = cards.clone();
        for card in cards {
            self.opponent_cards_in_game[card.suit().to_index()][card.rank().to_index()] = false;
            self.opponent_cards_in_game_by_suit[card.suit().to_index()] -= 1;
        }
    }

    fn play_card(&mut self, played_moves: &Vec<Move>) -> Card {
        // println!("Asked to play card; cards left: {}", self.player_state.cards.len());
        self.player_state.update_play_card(played_moves);
        self.process_moves(played_moves);

        let possible_cards = if let Some(first_move) = played_moves.first() {
            let same_suit_cards: Vec<Card> = self.player_state.cards.iter().filter(|c| c.suit() == first_move.card().suit()).map(|c| *c).collect();
            if same_suit_cards.len() > 0 {
                same_suit_cards
            } else {
                let non_scoring_cards: Vec<Card> = self.player_state.cards.iter().filter(|c| c.score() == 0).map(|c| *c).collect();
                if self.player_state.first_round && non_scoring_cards.len() > 0 {
                    non_scoring_cards
                } else {
                    self.player_state.cards.clone()
                }
            }
        } else {
            if self.player_state.first_round {
                vec![Card(Rank::Two, Suit::Clubs)]
            } else {
                let non_heart_cards: Vec<Card> = self.player_state.cards.iter().filter(|c| c.suit() != Suit::Hearts).map(|c| *c).collect();
                if !self.player_state.hearts_played && non_heart_cards.len() > 0 {
                    non_heart_cards
                } else {
                    self.player_state.cards.clone()
                }
            }
        };

        // println!("# of cards: {}", self.player_state.cards.len());
        let best_card = possible_cards.iter().min_by_key(|c| {
            let mut by_suit = [vec![], vec![], vec![], vec![]];
            for card in &self.player_state.cards { if card != *c { by_suit[card.suit() as usize - 1].push(*card); } }
            self.evaluate_round(played_moves, **c).try_add(self.evaluate_state(&by_suit)).unwrap()
        }).expect("There should always be valid playing options");

        self.player_state.update_did_play_card(best_card);

        *best_card
    }

    fn end_round(&mut self, winner_pidx: usize, played_moves: &Vec<Move>) {
        self.player_state.update_end_round(played_moves, winner_pidx);
        self.process_moves(played_moves);
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

impl MediasResActor for RuleActorV1 {
    fn new_from_player_state(pidx: usize, player_state: &PlayerState) -> Self {
        let cards_in_game_by_suit = player_state.cards_in_game.into_iter_fixed().map(|a| a.iter().filter(|b| **b).count()).collect();
        let mut opponent_cards_in_game = player_state.cards_in_game.clone();
        for card in &player_state.cards { opponent_cards_in_game[card.suit().to_index()][card.rank().to_index()] = false; }
        let opponent_cards_in_game_by_suit = opponent_cards_in_game.into_iter_fixed().map(|a| a.iter().filter(|b| **b).count()).collect();

        Self {
            cards_in_game_by_suit,
            opponent_cards_in_game,
            opponent_cards_in_game_by_suit,
            pidx,
            player_state: player_state.clone(),
        }
    }
}
