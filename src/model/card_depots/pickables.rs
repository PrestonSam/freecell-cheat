use crate::{model::{card::{Card, ProximateCard}, error::GameError, CardLocation}, utils::Ternary};

#[derive(Debug)]
pub struct PickableCard {
    pub(super) card: Card,
}

// Interesting idea, but you don't actually confirm that the pick matches the column or the game state at the time.
// You could probably bind those together with this move cleverly, but presently I'm not able to figure that out.
#[derive(Debug)]
pub struct PickableStack<Token>
where
    Token: PartialEq + Eq,
{
    pub(super) deepest_card: Card,
    pub(super) size: usize,
    pub(super) state_token: Token
}

pub trait HoldsCard {
    // Proposal: drop PickableCard and instead produce a CardLocation
    fn can_pick_card(&self) -> Option<PickableCard>;

    // Proposal: instead of PickableCard, consume ref CardLocation and produce Option<CardMove>, which contains an owned pick and put CardLocation
    fn can_put_card(&self, picked_card: &PickableCard) -> bool;

    fn pick_card(&mut self) -> Result<Card, GameError>;

    // Should this use CardMove? Maybe this should be a function that only CardMove can see, or something?
    fn take_card_from<T: HoldsCard>(&mut self, from: &mut T) -> Result<(), GameError>;
}

pub trait HoldsStack<Token>
where
    Token: PartialEq + Eq,
{
    // Proposal: produce StackLocation, which holds a size (usize) and a CardLocation (for the deepest or shallowest card?)
    fn can_pick_stack(&self, pick_size: usize) -> Option<PickableStack<Token>>;

    // I think I should create a new struct called Transaction that's returned here instead of bool
    // The transaction is still just a description of what should be done and can't actually do anything
    fn can_put_stack(&self, picked_stack: &PickableStack<Token>) -> bool;

    fn pick_stack(&mut self, pick: PickableStack<Token>) -> Result<Vec<Card>, GameError>;

    // This should become something like "execute transaction" and should accept a Transaction
    // The token should also contain identity about where it's from. If I have four columns with 0 as their state then I could generate a transaction on one column and feed it into another.
    fn take_stack_from<T: HoldsStack<Token>>(&mut self, from: &mut T, pick: PickableStack<Token>) -> Result<(), GameError>;
}

pub trait FindProxPair<T> {
    fn find_prox_pair(&self, prox_card: &ProximateCard) -> Ternary<T>;
}


pub enum Move {
    Card(CardMove),
    Stack(StackMove),
}

// Should contain two card locations. I guess that's it, really
pub struct CardMove {
    from: CardLocation,   
    to: CardLocation,
}

pub struct StackMove {
    size: usize,
    from: CardLocation,
    to: CardLocation,
}
