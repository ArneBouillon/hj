use crate::{Card, Rank, RuleActorV1, Suit};
use crate::util::non_nan::NonNan;

use iter_fixed::IntoIteratorFixed;
use std::cmp::Reverse;
use rand::Rng;

impl RuleActorV1 {
    fn eval_ranks(ranks: &Vec<Rank>, all_ranks: &Vec<Rank>) -> (f32, Vec<f32>) {
        if ranks.len() == all_ranks.len() { return (0., vec![]); }

        let mut ranks = ranks.clone();
        let mut all_ranks = all_ranks.clone();

        let mut wins = 0.;
        let empty_scores = vec![];
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

    fn eval_state_spades(&self, cards: &Vec<Card>) -> (f32, f32, Vec<f32>) {
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

    fn eval_state_clubs(&self, cards: &Vec<Card>) -> (f32, f32, Vec<f32>) {
        if cards.is_empty() {
            return (0., 0., vec![]);
        }

        let mut cards_no_two = cards.clone();
        cards_no_two.retain(|c| c.rank() != Rank::Two);
        let have_two = cards.len() != cards_no_two.len();

        let ranks: Vec<Rank> = cards_no_two.iter().map(|c| c.rank()).collect();
        let mut all_ranks: Vec<Rank> = self.cards_in_game[1].iter().enumerate().filter_map(|(i, r)| if *r { Some(Rank::from_index(i as u8)) } else { None }).collect(); //Rank::all().into_iter().collect();
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

    fn eval_state_diamonds(&self, cards: &Vec<Card>) -> (f32, f32, Vec<f32>) {
        if cards.is_empty() {
            return (0., 0., vec![]);
        }

        let ranks: Vec<Rank> = cards.iter().map(|c| c.rank()).collect();
        let all_ranks: Vec<Rank> = self.cards_in_game[2].iter().enumerate().filter_map(|(i, r)| if *r { Some(Rank::from_index(i as u8)) } else { None }).collect(); //Rank::all().into_iter().collect();

        let (wins, empty_costs) = RuleActorV1::eval_ranks(&ranks, &all_ranks);
        let cost = wins;

        (cost, cards.len() as f32, empty_costs)
    }

    fn eval_state_hearts(&self, cards: &Vec<Card>) -> (f32, f32, Vec<f32>) {
        if cards.is_empty() {
            return (0., 0., vec![]);
        }

        let ranks: Vec<Rank> = cards.iter().map(|c| c.rank()).collect();
        let all_ranks: Vec<Rank> = self.cards_in_game[3].iter().enumerate().filter_map(|(i, r)| if *r { Some(Rank::from_index(i as u8)) } else { None }).collect();

        let (wins, empty_costs) = RuleActorV1::eval_ranks(&ranks, &all_ranks);
        let cost = wins * 4.;

        (cost, cards.len() as f32, empty_costs)
    }

    pub fn evaluate_state(&self, by_suit: &[Vec<Card>; 4]) -> NonNan {
        let suit_results = [
            self.eval_state_spades(&by_suit[0]),
            self.eval_state_clubs(&by_suit[1]),
            self.eval_state_diamonds(&by_suit[2]),
            self.eval_state_hearts(&by_suit[3]),
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
                    counts_left[*throw_away_sidx] = counts_left[*throw_away_sidx].saturating_sub(1);
                    total_score += 0. * throw_away_score;
                }
            }
        }

        NonNan::new(total_score).unwrap()
    }
}
