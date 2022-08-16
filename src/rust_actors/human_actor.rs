use core::default::Default;
use crate::shared::actor::Actor;
use crate::shared::data::{Card, Move, PassDirection};

use std::io::Read;

use colored::Colorize;

pub struct HumanActor {
    cards: Vec<Card>,
    pidx: usize,
    score: [isize; 4],
}


impl HumanActor {
    pub fn new() -> Self {
        Self { cards: vec![], pidx: 0, score: [0; 4] }
    }

    fn show_moves(&self, moves: &Vec<Move>, winner_pidx: Option<usize>) {
        println!("{}", "\n".repeat(10));
        println!("Player 0 ({:3}){}Player 1 ({:3}){}Player 2 ({:3}){}Player 3 ({:3})", self.score[0], " ".repeat(11), self.score[1], " ".repeat(11), self.score[2], " ".repeat(11), self.score[3]);

        let mut ordered_cards = [None; 4];
        let mut card_string: String = Default::default();
        for mv in moves {
            ordered_cards[mv.pidx()] = Some(mv.card());
        }
        for pidx in 0..4 {
            let new_string = format!(
                "{:25}",
                match ordered_cards[pidx] {
                    None => "".to_owned(),
                    Some(card) => format!("{:?}", card),
                }
            );
            if Some(pidx) == winner_pidx { card_string.push_str(&new_string.bold()) }
            else { card_string.push_str(&new_string); }
        }

        println!("{}", card_string);
    }

    fn query_card(&self) -> Card {
        let mut cards_by_suit = [vec![], vec![], vec![], vec![]];
        for card in &self.cards { cards_by_suit[card.suit() as usize - 1].push(card); }
        for sidx in 0..4 { cards_by_suit[sidx].sort(); }

        let mut card_string: String = "\n\n\n".to_owned();
        let mut counter = 1;
        let mut choices = vec![];
        for cidx in 0.. {
            let mut found_one = false;
            for sidx in 0..4 {
                match cards_by_suit[sidx].get(cidx) {
                    None => card_string.push_str(&" ".repeat(25)),
                    Some(card) => {
                        card_string.push_str(&format!("{:4}{:21}", format!("{}.", counter), format!("{:?}", card)));
                        choices.push(card);
                        found_one = true;
                        counter += 1;
                    },
                }
            }
            card_string.push_str(&"\n");

            if !found_one { break; }
        }

        println!("{}", card_string);
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        **choices[input.trim().parse::<usize>().unwrap() - 1]
    }

    pub fn pause() {
        std::io::stdin().read(&mut [0]).unwrap();
    }
}

impl Actor for HumanActor {
    fn initialize(&mut self, pidx: usize, cards: &Vec<Card>) {
        self.pidx = pidx;
        self.cards = cards.clone();
    }

    fn play_card(&mut self, played_moves: &Vec<Move>) -> Card {
        self.show_moves(played_moves, None);
        let card = self.query_card();
        self.cards.retain(|c| *c != card);

        card
    }

    fn end_round(&mut self, winner_pidx: usize, played_moves: &Vec<Move>) {
        self.score[winner_pidx] += played_moves.iter().map(|m| m.card().score()).sum::<isize>();
        self.show_moves(played_moves, Some(winner_pidx));

        HumanActor::pause();
    }

    fn end_game(&mut self, _scores: [isize; 4]) {}

    fn get_pass(&mut self, direction: PassDirection) -> Vec<Card> {
        match direction {
            PassDirection::None => { println!("No-pass round"); HumanActor::pause(); vec![] }
            _ => {
                println!("Pass direction: {:?}", direction);
                HumanActor::pause();
                (0..3).map(|_| { let card = self.query_card(); self.cards.retain(|c| *c != card); card }).collect()
            }
        }
    }

    fn end_pass(&mut self, passed_cards: &Vec<Card>) {
        self.cards.append(&mut passed_cards.clone());
        if passed_cards.len() > 0 {
            println!("Cards received:");
            for card in passed_cards {
                println!("  {:?}", card);
            }
            HumanActor::pause();
        }
    }
}
