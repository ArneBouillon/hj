use crate::{Card, Rank, ActorRuleV1, Suit};
use crate::rust_actors::player_state::ExtendedPlayerStateInterface;
use crate::game::data::Move;
use crate::util::non_nan::NonNan;

impl<PlayerState : ExtendedPlayerStateInterface> ActorRuleV1<PlayerState> {
    pub(super) fn evaluate_round(&self, played_moves: &Vec<Move>, new_card: Card) -> NonNan {
        let Card(new_rank, new_suit) = new_card;
        if let Some(Move(_, Card(first_rank, first_suit))) = played_moves.first() {
            if new_suit != *first_suit || new_rank < *first_rank { return NonNan::zero() }
        }

        let partial_score = (played_moves.iter().map(|m| m.card().score()).sum::<isize>() + new_card.score()) as f32;

        NonNan::new(
            match played_moves.len() {
                3 => partial_score,
                len => {
                    let togo = 3 - len;
                    let total_still_left = (self.player_state.pidx()+1 .. self.player_state.pidx()+4).filter(|pidx| self.player_state.still_has()[new_suit.to_index()][pidx % 4]).count();
                    let togo_still_have = (self.player_state.pidx()+1 .. self.player_state.pidx()+togo+1).filter(|pidx| self.player_state.still_has()[new_suit.to_index()][pidx % 4]).count();

                    let total_left = self.player_state.opponent_cards_in_game()[new_suit.to_index()].iter().filter(|b| **b).count();
                    let better_left = if partial_score >= 0. {
                        self.player_state.opponent_cards_in_game()[new_suit.to_index()][..new_rank.to_index()].iter().filter(|b| **b).count()
                    } else {
                        self.player_state.opponent_cards_in_game()[new_suit.to_index()][new_rank.to_index()..].iter().filter(|b| **b).count()
                    };

                    let n = total_still_left as u32; let a = togo_still_have as u32; let g = better_left as u32; let b = total_left as u32 - g;
                    let odds_someone_will_take_over = match a {
                        0 => 0.,
                        1 => ((n-1).pow(g) * (n.pow(b) - (n-1).pow(b))) as f32,
                        2 => (2 * (n-1).pow(g) * (n.pow(b) - (n-1).pow(b)) - (n-2).pow(g) * (n.pow(b) - 2*(n-1).pow(b) + (n-2).pow(b))) as f32,
                        3 => (3 * (n-1).pow(g) * (n.pow(b) - (n-1).pow(b)) - 3 * (n-2).pow(g) * (n.pow(b) - 2*(n-1).pow(b) + (n-2).pow(b))) as f32,
                        _ => unreachable!()
                    } / if n == 0 { 1. } else { n.pow(b + g) as f32 };

                    (1. - odds_someone_will_take_over) * (
                        partial_score
                            + togo_still_have as f32 * (if new_suit == Suit::Hearts { 1. } else { 0.2 })
                            + (togo - togo_still_have) as f32 * (0.8 + if self.player_state.opponent_cards_in_game()[Suit::Spades.to_index()][Rank::Queen.to_index()] { 3. } else { 0. })
                    )
                },
            }
        ).unwrap()
    }
}
