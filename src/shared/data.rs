use core::clone::Clone;

use num_derive::FromPrimitive;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum PassDirection {
    Left,
    Right,
    Cross,
    None,
}

#[derive(Copy, Clone)]
pub struct Move(pub usize, pub Card);

#[derive(Debug)]
pub struct Hand {
    cards: Vec<Card>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Card(pub Rank, pub Suit);

#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, FromPrimitive)]
pub enum Rank {
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
    Seven = 7,
    Eight = 8,
    Nine = 9,
    Ten = 10,
    Jack = 11,
    Queen = 12,
    King = 13,
    Ace = 14,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, FromPrimitive)]
pub enum Suit {
    Spades = 1,
    Clubs = 2,
    Diamonds = 3,
    Hearts = 4,
}


impl PassDirection {
    pub fn index_shift(&self) -> usize {
        match self {
            PassDirection::None => 0,
            PassDirection::Left => 1,
            PassDirection::Cross => 2,
            PassDirection::Right => 3,
        }
    }
}

impl Move {
    pub fn pidx(&self) -> usize {
        self.0
    }

    pub fn card(&self) -> Card {
        self.1
    }
}

impl Hand {
    pub fn new(cards: Vec<Card>) -> Self {
        Self { cards }
    }

    pub fn cards(&self) -> &Vec<Card> {
        &self.cards
    }

    pub fn all_hearts(&self) -> bool {
        self.cards.iter().all(|c| c.suit() == Suit::Hearts)
    }

    pub fn all_have_score(&self) -> bool {
        self.cards.iter().all(|c| c.score() != 0)
    }

    pub fn any_of_suit(&self, suit: Suit) -> bool {
        self.cards.iter().any(|c| c.suit() == suit)
    }

    pub fn any_match(&self, card: Card) -> bool {
        self.cards.iter().any(|c| *c == card)
    }

    pub fn remove(&mut self, card: Card) {
        self.cards.retain(|c| *c != card);
    }

    pub fn add(&mut self, card: Card) {
        self.cards.push(card);
    }
}

impl Card {
    pub fn new(rank: Rank, suit: Suit) -> Self {
        Self(rank, suit)
    }

    pub fn rank(&self) -> Rank {
        self.0
    }

    pub fn suit(&self) -> Suit {
        self.1
    }

    pub fn score(self) -> isize {
        match self {
            Card(_, Suit::Hearts) => 1,
            Card(Rank::Queen, Suit::Spades) => 13,
            Card(Rank::Jack, Suit::Diamonds) => -10,
            _ => 0,
        }
    }
}

impl Rank {
    pub fn all() -> [Rank; 13] {
        [
            Rank::Two, Rank::Three, Rank::Four, Rank::Five, Rank::Six, Rank::Seven, Rank::Eight,
            Rank::Nine, Rank::Ten, Rank::Jack, Rank::Queen, Rank::King, Rank::Ace,
        ]
    }
}

impl Suit {
    pub fn all() -> [Suit; 4] {
        [Suit::Spades, Suit::Clubs, Suit::Diamonds, Suit::Hearts]
    }
}
