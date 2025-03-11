use crate::{model::{card::{Card, ProximateCard}, error::GameError}, utils::Ternary};

#[derive(Debug)]
pub struct PickableCard { pub(super) card: Card }

// Interesting idea, but you don't actually confirm that the pick matches the column or the game state at the time.
// You could probably bind those together with this move cleverly, but presently I'm not able to figure that out.
#[derive(Debug)]
pub struct PickableStack { pub(super) deepest_card: Card, pub(super) size: usize }

pub trait HoldsCard {
    fn can_pick_card(&self) -> Option<PickableCard>;

    fn can_put_card(&self, picked_card: &PickableCard) -> bool;

    fn pick_card(&mut self) -> Result<Card, GameError>;

    fn take_card_from<T: HoldsCard>(&mut self, from: &mut T) -> Result<(), GameError>;
}

pub trait HoldsStack {
    fn can_pick_stack(&self, pick_size: usize) -> Option<PickableStack>;

    fn can_put_stack(&self, picked_stack: &PickableStack) -> bool;

    fn pick_stack(&mut self, pick: PickableStack) -> Result<Vec<Card>, GameError>;

    fn take_stack_from<T: HoldsStack>(&mut self, from: &mut T, pick: PickableStack) -> Result<(), GameError>;
}

pub trait FindProxPair<T> {
    fn find_prox_pair(&self, prox_card: &ProximateCard) -> Ternary<T>;
}
