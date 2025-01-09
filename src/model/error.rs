#[derive(Debug)]
pub enum GameError {
    NoSuchReserveSlot(usize),
    TriedToPickEmptyReserveSlot,
    ReserveSlotIsOccupied,

    NoSuchFoundationStack(usize),
    TriedToPickEmptyFoundationStack,

    TriedToPickEmptyColumn,
    InsufficientCardsForStackPick { stack_size: usize },

    NoSuchColumn(usize),
}