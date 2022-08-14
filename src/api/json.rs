use crate::shared::actor::Actor;
use crate::shared::data::{Card, Move};

use std::process::{Child, Command, Stdio};
use std::io::{Read, Write};

use json;
use num;

pub struct JSONActor {
    child : Child,
}

impl JSONActor {
    pub fn new(command_string: String) -> Self {
        Self {
            child: Command::new(command_string).stdout(Stdio::piped())
                                               .stdin(Stdio::piped())
                                               .spawn()
                                               .expect("Failed to start process.")
        }
    }

    fn serialize_cards(cards: &Vec<Card>) -> Vec<Vec<usize>> {
        cards.iter().map(|c| vec![c.rank() as usize, c.suit() as usize]).collect::<Vec<Vec<usize>>>()
    }

    fn serialize_moves(moves: &Vec<Move>) -> Vec<Vec<usize>> {
        moves.iter().map(|m| vec![m.pidx(), m.card().rank() as usize, m.card().suit() as usize]).collect::<Vec<Vec<usize>>>()
    }

    fn deserialize_card(card: Vec<usize>) -> Card {
        Card::new(num::FromPrimitive::from_u8(card[0] as u8).unwrap(), num::FromPrimitive::from_u8(card[1] as u8).unwrap())
    }
}

impl Actor for JSONActor {
    fn initialize(&mut self, pidx: usize, cards: &Vec<Card>) {
        println!("Initializing");
        let message = json::object!{
            message: "initialize",
            pidx: pidx,
            cards: JSONActor::serialize_cards(cards),
        };

        (&mut self.child.stdin).as_mut().unwrap().write((json::stringify(message) + "\n").as_bytes()).unwrap();
        let mut buffer = [32; 1];
        let mut string = "".to_owned();
        loop {
            (&mut self.child.stdout.as_mut().unwrap()).take(1).read(&mut buffer).unwrap();
            if buffer == [b'\n'] { break; }
            else { string += &String::from_utf8_lossy(&buffer); }
        }

        let _parsed = json::parse(&string).unwrap();
    }

    fn play_card(&mut self, played_moves: &Vec<Move>) -> Card {
        let message = json::object!{
            message: "play_card",
            played_moves: JSONActor::serialize_moves(played_moves),
        };

        (&mut self.child.stdin).as_mut().unwrap().write((json::stringify(message) + "\n").as_bytes()).unwrap();
        let mut buffer = [32; 1];
        let mut string = "".to_owned();
        loop {
            (&mut self.child.stdout.as_mut().unwrap()).take(1).read(&mut buffer).unwrap();
            if buffer == [b'\n'] { break; }
            else { string += &String::from_utf8_lossy(&buffer); }
        }
        let parsed = json::parse(&string).unwrap();

        match &parsed["card"] {
            json::JsonValue::Array(vec) => JSONActor::deserialize_card(vec.iter().map(|num| num.as_usize().unwrap()).collect()),
            _ => panic!("Should be an array."),
        }
    }

    fn end_round(&mut self, winner_pidx: usize, played_moves: &Vec<Move>) {
        let message = json::object!{
            message: "end_round",
            winner_pidx: winner_pidx,
            played_moves: JSONActor::serialize_moves(played_moves),
        };

        (&mut self.child.stdin).as_mut().unwrap().write((json::stringify(message) + "\n").as_bytes()).unwrap();
        let mut buffer = [32; 1000];
        (&mut self.child.stdout.as_mut().unwrap()).read(&mut buffer).unwrap();
        let _parsed = json::parse(&String::from_utf8_lossy(&buffer)).unwrap();
    }

    fn end_game(&mut self, scores: [isize; 4]) {
        let message = json::object!{
            message: "end_game",
            scores: scores.to_vec(),
        };

        (&mut self.child.stdin).as_mut().unwrap().write((json::stringify(message) + "\n").as_bytes()).unwrap();
        let mut buffer = [32; 1000];
        (&mut self.child.stdout.as_mut().unwrap()).read(&mut buffer).unwrap();
        let _parsed = json::parse(&String::from_utf8_lossy(&buffer)).unwrap();
    }
}
