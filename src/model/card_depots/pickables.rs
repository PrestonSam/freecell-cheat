use derive_more::From;

use crate::{model::{card::{Card, ProximateCard}, error::GameError, CardLocation}, utils::Ternary};

#[derive(Debug, From, Clone)]
pub struct PickableCard(pub(crate) Card, pub(crate) CardLocation);

impl PickableCard {
    pub fn new(card: &Card, location: impl Into<CardLocation>) -> Self {
        Self(card.clone(), location.into())
    }
}

// Interesting idea, but you don't actually confirm that the pick matches the column or the game state at the time.
// You could probably bind those together with this move cleverly, but presently I'm not able to figure that out.
#[derive(Debug, PartialEq, Eq)]
pub struct PickableStack
{
    pub(super) deepest_card: Card,
    pub location: CardLocation,
    pub(super) size: usize, // Seems to be in the CardLocation now, might not need this
}

impl PickableStack {
    pub fn new(deepest_card: &Card, location: impl Into<CardLocation>, size: usize) -> Self {
        Self {
            deepest_card: deepest_card.clone(),
            location: location.into(),
            size,
        }
    }
}

impl PartialOrd for PickableStack {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PickableStack {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let PickableStack { location: self_loc, size: self_size, .. } = self;
        let PickableStack { location: othr_loc, size: othr_size, .. } = other;

        self_loc.get_distance()
            .cmp(&othr_loc.get_distance())
            .then_with(|| self_size.cmp(othr_size))
    }
}

pub trait HoldsCard {
    fn try_get_card_pick(&self) -> Option<PickableCard>;

    // Proposal: instead of PickableCard, consume ref CardLocation and produce Option<CardMove>, which contains an owned pick and put CardLocation
    fn try_get_card_move(&self, picked_card: &PickableCard) -> Option<CardMove>;

    fn pick_card(&mut self) -> Result<Card, GameError>;

    // Should this use CardMove? Maybe this should be a function that only CardMove can see, or something?
    fn take_card_from(&mut self, from: &mut dyn HoldsCard) -> Result<(), GameError>;
}

pub trait HoldsStack
{
    // Proposal: produce StackLocation, which holds a size (usize) and a CardLocation (for the deepest or shallowest card?)
    fn can_pick_stack(&self, pick_size: usize) -> Option<PickableStack>;

    // I think I should create a new struct called Transaction that's returned here instead of bool
    // The transaction is still just a description of what should be done and can't actually do anything
    fn can_put_stack(&self, picked_stack: &PickableStack) -> Option<StackMove>;

    fn pick_stack(&mut self, pick: PickableStack) -> Result<Vec<Card>, GameError>;

    // This should become something like "execute transaction" and should accept a Transaction
    // The token should also contain identity about where it's from. If I have four columns with 0 as their state then I could generate a transaction on one column and feed it into another.
    fn take_stack_from<T: HoldsStack>(&mut self, from: &mut T, pick: PickableStack) -> Result<(), GameError>;
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
    pub card: Card,
    pub from: CardLocation,
    pub to: CardLocation,
}

impl CardMove {
    pub fn new(pick: &PickableCard, put: impl Into<CardLocation>) -> Self {
        CardMove {
            card: pick.0.clone(),
            from: pick.1.clone(),
            to: put.into(),
        }
    }
}

// Presumably there should be tokens here, too?
pub struct StackMove {
    pub size: usize,
    pub deepest_card: Card,
    pub from: CardLocation,
    pub to: CardLocation,
}

impl StackMove {
    pub fn new(pick: &PickableStack, put: impl Into<CardLocation>) -> Self {
        StackMove {
            size: pick.size,
            deepest_card: pick.deepest_card.clone(),
            from: pick.location.clone(),
            to: put.into(),
        }
    }
}
