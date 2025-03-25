use super::CardLocation;

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
