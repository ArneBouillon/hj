use super::actor::Actor;
use super::data::Hand;

pub struct Player<'a> {
    actor: &'a mut dyn Actor,
    hand: Hand,
}


impl<'a> Player<'a> {
    pub fn new(actor: &'a mut dyn Actor, hand: Hand) -> Self {
        Self { actor, hand }
    }

    pub fn actor_mut(&mut self) -> &mut dyn Actor {
        self.actor
    }

    pub fn hand(&self) -> &Hand {
        &self.hand
    }

    pub fn hand_mut(&mut self) -> &mut Hand {
        &mut self.hand
    }
}
