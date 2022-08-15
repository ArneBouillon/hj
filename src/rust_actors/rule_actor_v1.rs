use crate::PassDirection;
use crate::shared::actor::Actor;
use crate::shared::data::{Card, Move, Rank, Suit};
use crate::util::non_nan::NonNan;

use itertools::Itertools;
use itertools::FoldWhile::{Continue, Done};
use std::cmp::Reverse;

use iter_fixed::{IntoIteratorFixed, IteratorFixed};
use tap::prelude::*;
use rand::seq::SliceRandom;
use rand::thread_rng;
use rand::Rng;

pub struct RuleActorV1 {
    cards: Vec<Card>,
    first_round: bool,
    hearts_played: bool,
    pidx: usize,
}


impl RuleActorV1 {
    pub fn new() -> Self {
        Self { cards: vec![], first_round: true, hearts_played: false, pidx: 0 }
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

    fn eval_ranks(ranks: &Vec<Rank>, all_ranks: &Vec<Rank>) -> (f32, Vec<f32>) {
        if ranks.len() == all_ranks.len() { return (0., vec![]); }

        let mut ranks = ranks.clone();
        let mut all_ranks = all_ranks.clone();

        let mut wins = 0.;
        let mut empty_scores = vec![];
        while ranks.len() != 0 && all_ranks.len() >= ranks.len() + 3 {
            if ranks[0] >= all_ranks[3] {
                wins += 1.;
                ranks.remove(0);
                all_ranks.remove(0);
                all_ranks.remove(0);
                all_ranks.remove(0);
                all_ranks.remove(0);
            } else {
                ranks.remove(0);
                all_ranks.remove(0);
                all_ranks.remove(all_ranks.len() - 1);
                all_ranks.remove(all_ranks.len() - 1);
            }
        }

        (wins, empty_scores)
    }

    fn eval_state_spades(cards: &Vec<Card>) -> (f32, f32, Vec<f32>) {
        if cards.is_empty() {
            return (0., 0., vec![]);
        }

        let have_queen = cards.iter().any(|c| *c == Card(Rank::Queen, Suit::Spades));
        let num_big = cards.iter().filter(|c| c.rank() > Rank::Queen).count();
        let num_small = cards.iter().filter(|c| c.rank() < Rank::Queen).count();

        let mut cost = 0.;
        let mut empty_costs = vec![];
        if have_queen {
            match cards.len() {
                1 => { cost += 9.0; empty_costs.push(-9.0); }
                2 => if num_big > 0 { cost += 8.0; empty_costs.push(-5.0); empty_costs.push(-3.0); }
                     else { cost += 7.0; empty_costs.push(-7.0); }
                3 => { cost += 5.0; empty_costs.push(if num_small > 0 { -5.0 } else { -3.0 }) }
                4 => { cost += 2.0; empty_costs.push(-1.0) }
                _ => {}
            }
        } else {
            match (num_big, num_small) {
                (0, _) => {}
                (1, _) => {
                    cost += 5_usize.saturating_sub(num_small) as f32;
                    empty_costs.push(-(5_usize.saturating_sub(num_small) as f32));
                },
                (2, _) => {
                    cost += 7_usize.saturating_sub(2*num_small) as f32;
                    empty_costs.push(-(7_usize.saturating_sub(2*num_small) as f32) / 2.0);
                    empty_costs.push(-(7_usize.saturating_sub(2*num_small) as f32) / 2.0);
                },
                _ => panic!("There can only be 0, 1 or 2 big cards."),
            }
        }

        (cost, cards.len() as f32, empty_costs)
    }

    fn eval_state_clubs(cards: &Vec<Card>) -> (f32, f32, Vec<f32>) {
        if cards.is_empty() {
            return (0., 0., vec![]);
        }

        let mut cards_no_two = cards.clone();
        cards_no_two.retain(|c| c.rank() != Rank::Two);
        let have_two = cards.len() != cards_no_two.len();

        let mut ranks: Vec<Rank> = cards_no_two.iter().map(|c| c.rank()).collect();
        let mut all_ranks: Vec<Rank> = Rank::all().into_iter().collect();
        if have_two {
            all_ranks.remove(0);
            let mut counter = 3;
            for ridx in (0..12).rev() {
                if !ranks.iter().any(|r| *r == all_ranks[ridx]) {
                    all_ranks.remove(ridx);
                    counter -= 1;
                }
                if counter == 0 { break; }
            }
        }
        let (wins, empty_costs) = RuleActorV1::eval_ranks(&ranks, &all_ranks);
        let cost = wins;

        (cost, cards.len() as f32, empty_costs)
    }

    fn eval_state_diamonds(cards: &Vec<Card>) -> (f32, f32, Vec<f32>) {
        if cards.is_empty() {
            return (0., 0., vec![]);
        }

        let mut ranks: Vec<Rank> = cards.iter().map(|c| c.rank()).collect();
        let mut all_ranks: Vec<Rank> = Rank::all().into_iter().collect();

        let (wins, empty_costs) = RuleActorV1::eval_ranks(&ranks, &all_ranks);
        let cost = wins;

        (cost, cards.len() as f32, empty_costs)
    }

    fn eval_state_hearts(cards: &Vec<Card>) -> (f32, f32, Vec<f32>) {
        if cards.is_empty() {
            return (0., 0., vec![]);
        }

        let mut ranks: Vec<Rank> = cards.iter().map(|c| c.rank()).collect();
        let mut all_ranks: Vec<Rank> = Rank::all().into_iter().collect();

        let (wins, empty_costs) = RuleActorV1::eval_ranks(&ranks, &all_ranks);
        let cost = wins * 4.;

        (cost, cards.len() as f32, empty_costs)
    }

    fn evaluate_state(by_suit: &[Vec<Card>; 4]) -> NonNan {
        let suit_results = [
            RuleActorV1::eval_state_spades(&by_suit[0]),
            RuleActorV1::eval_state_clubs(&by_suit[1]),
            RuleActorV1::eval_state_diamonds(&by_suit[2]),
            RuleActorV1::eval_state_hearts(&by_suit[3]),
        ];

        let mut total_score = suit_results[0].0 + suit_results[1].0 + suit_results[2].0 + suit_results[3].0;
        let mut empty_scores: Vec<(&f32, &usize)> = suit_results[0].2.iter().zip([0].iter().cycle()).collect();
        empty_scores.append(&mut suit_results[1].2.iter().zip([1].iter().cycle()).collect());
        empty_scores.append(&mut suit_results[2].2.iter().zip([2].iter().cycle()).collect());
        empty_scores.append(&mut suit_results[3].2.iter().zip([3].iter().cycle()).collect());

        empty_scores.sort_by_key(|s| Reverse(NonNan::new(*s.0).unwrap()));
        let mut counts_left: [usize; 4] = by_suit.into_iter_fixed().map(|v| v.len()).collect();
        let mut rng = rand::thread_rng();
        for _ in 0..13 {
            let sidx = rng.gen_range(0..4);
            if counts_left[sidx] > 0 {
                counts_left[sidx] -= 1;
            } else {
                if let Some((throw_away_score, throw_away_sidx)) = empty_scores.pop() {
                    counts_left[*throw_away_sidx].saturating_sub(1);
                    total_score += 0.5 * throw_away_score;
                }
            }
        }

        NonNan::new(total_score).unwrap()
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
        for card in &self.cards {
            by_suit[card.suit() as usize - 1].push(card);
            by_suit_counts[card.suit() as usize - 1] += 1;
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
        }).sorted_by_key(|cards|
            NonNan::new(
                0.2 * RuleActorV1::evaluate_state(&RuleActorV1::add_ghost_cards(cards, spade_card, club_card, heart_card)).value()
                + 0.8 * RuleActorV1::evaluate_state(&cards).value()
            ).unwrap()
        ).next().expect("There should always be valid passing options").into_iter().flatten().collect();

        let mut result = self.cards.clone();
        result.retain(|c| !best_option.contains(&c));
        result
    }
}

impl Actor for RuleActorV1 {
    fn initialize(&mut self, pidx: usize, cards: &Vec<Card>) {
        self.pidx = pidx;
        self.cards = cards.clone();
    }

    fn play_card(&mut self, played_moves: &Vec<Move>) -> Card {
        self.cards.shuffle(&mut thread_rng());

        let possible_cards = if let Some(first_move) = played_moves.first() {
            let same_suit_cards: Vec<Card> = self.cards.iter().filter(|c| c.suit() == first_move.card().suit()).map(|c| *c).collect();
            if same_suit_cards.len() > 0 {
                same_suit_cards
            } else {
                let non_scoring_cards: Vec<Card> = self.cards.iter().filter(|c| c.score() == 0).map(|c| *c).collect();
                if self.first_round && non_scoring_cards.len() > 0 {
                    non_scoring_cards
                } else {
                    self.cards.clone()
                }
            }
        } else {
            if self.first_round {
                vec![Card(Rank::Two, Suit::Clubs)]
            } else {
                let non_heart_cards: Vec<Card> = self.cards.iter().filter(|c| c.suit() != Suit::Hearts).map(|c| *c).collect();
                if !self.hearts_played && non_heart_cards.len() > 0 {
                    non_heart_cards
                } else {
                    self.cards.clone()
                }
            }
        };

        let best_card = *possible_cards.first().unwrap();
        self.cards.retain(|c| *c != best_card);
        best_card
    }

    fn end_round(&mut self, winner_pidx: usize, played_moves: &Vec<Move>) {
        self.first_round = false;
        if played_moves.into_iter().any(|m| m.card().suit() == Suit::Hearts) {
            self.hearts_played = true;
        }
    }

    fn end_game(&mut self, score: [isize; 4]) {}

    fn get_pass(&mut self, direction: PassDirection) -> Vec<Card> {
        if direction == PassDirection::None { return vec![]; }

        let passed_cards = self.choose_three_to_pass();
        for passed_card in &passed_cards { self.cards.retain(|c| c != passed_card); }
        passed_cards
    }

    fn end_pass(&mut self, passed_cards: &Vec<Card>) {
        self.cards.append(&mut passed_cards.clone());
    }
}
