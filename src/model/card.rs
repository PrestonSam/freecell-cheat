pub const NUMBER_OF_CARDS_IN_PACK: usize = 14;

pub const ACE: usize = 1;

pub const JACK: usize = 11;

pub const QUEEN: usize = 12;

pub const KING: usize = 13;



#[derive(Debug, PartialEq, Eq, Copy, Clone, PartialOrd, Ord)]
pub struct Value(usize);

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Color {
    Red,
    Black,
}

impl Color {
    pub fn get_opposing_color(&self) -> Color {
        match self {
            Self::Red => Self::Black,
            Self::Black => Self::Red,
        }
    }
}

#[derive(Debug)]
pub enum PackType {
    Spades,
    Clubs,
    Hearts,
    Diamonds,
}

impl PackType {
    pub fn get_colour(&self) -> Color {
        match &self {
            PackType::Spades | PackType::Clubs => Color::Black,
            PackType::Hearts | PackType::Diamonds => Color::Red,
        }
    }

    pub fn is_opposing_color(&self, other: &PackType) -> bool {
        self.get_colour() != other.get_colour()
    }

    pub fn get_opposing_pack_types(&self) -> (PackType, PackType) {
        match self.get_colour() {
            Color::Red => (PackType::Spades, PackType::Clubs),
            Color::Black => (PackType::Hearts, PackType::Diamonds),
        }
    }
}




pub struct Pack(PackType, [char; 14]);

impl Pack {
    fn from(pack_type: PackType) -> &'static Self {
        match pack_type {
            PackType::Spades => &SPADES,
            PackType::Clubs => &CLUBS,
            PackType::Hearts => &HEARTS,
            PackType::Diamonds => &DIAMONDS,
        }
    }
}


pub const SPADES: Pack
    = Pack(PackType::Spades, [ '🂡', '🂢', '🂣', '🂤', '🂥', '🂦', '🂧', '🂨', '🂩', '🂪', '🂫', '🂬', '🂭', '🂮', ]);

pub const CLUBS: Pack
    = Pack(PackType::Clubs, [ '🃑', '🃒', '🃓', '🃔', '🃕', '🃖', '🃗', '🃘', '🃙', '🃚', '🃛', '🃜', '🃝', '🃞', ]);

pub const HEARTS: Pack
    = Pack(PackType::Hearts, [ '🂱', '🂲', '🂳', '🂴', '🂵', '🂶', '🂷', '🂸', '🂹', '🂺', '🂻', '🂼', '🂽', '🂾', ]);

pub const DIAMONDS: Pack
    = Pack(PackType::Diamonds, [ '🃁', '🃂', '🃃', '🃄', '🃅', '🃆', '🃇', '🃈', '🃉', '🃊', '🃋', '🃌', '🃍', '🃎', ]);


impl std::fmt::Debug for Pack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Pack")
            .field(&self.0)
            .finish()
    }
}



#[derive(Debug, Clone)]
pub struct FuzzyCard {
    pub color: Color,
    pub value: Value,
}

#[derive(Clone)]
pub struct Card<'a>(Value, &'a Pack);

impl<'a> std::fmt::Debug for Card<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Card")
            .field(&self.0.0)
            .field(&self.1.0)
            .finish()
    }
}

impl<'data> Card<'data> {
    pub fn new(value: usize, pack: &'data Pack) -> Self {
        Self(Value(value), pack)
    }

    pub fn get_color(&self) -> Color {
        self.1.0.get_colour()
    }

    pub fn get_value(&self) -> Value {
        self.0
    }

    pub fn is_same_pack(&self, other: &Card<'data>) -> bool {
        let Card(_, s_pack) = self;
        let Card(_, o_pack) = other;

        std::ptr::eq(s_pack, o_pack)
    }

    pub fn is_complementary_pack(&self, other: &Card<'data>) -> bool {
        let Card(_, Pack(s_type, _)) = self;
        let Card(_, Pack(o_type, _)) = other;

        s_type.is_opposing_color(o_type)
    }

    pub fn is_playable_pair_smaller(&self, other: &Card<'data>) -> bool {
        let Card(s_value, _) = *self;
        let Card(o_value, _) = *other;

        s_value.0 + 1 == o_value.0
    }

    pub fn is_playable_pair_bigger(&self, other: &Card<'data>) -> bool {
        let Card(s_value, _) = *self;
        let Card(o_value, _) = *other;

        o_value.0 + 1 == s_value.0
    }

    pub fn get_parent_data(&self) -> Option<FuzzyCard> {
        (self.0.0 < KING)
            .then(|| FuzzyCard { color: self.get_color().get_opposing_color(), value: Value(self.0.0 + 1) })
    }

    pub fn get_parents(&self) -> Option<(Card<'data>, Card<'data>)> {
        let Card(value, Pack(pack_type, _)) = *self;

        (value.0 < KING)
            .then(|| {
                let (fst_pack_type, snd_pack_type) = pack_type.get_opposing_pack_types();
                let fst_pack = Pack::from(fst_pack_type);
                let snd_pack = Pack::from(snd_pack_type);

                let parent_value = Value(value.0 + 1);

                (Card(parent_value, fst_pack), Card(parent_value, snd_pack))
            })
    }
}

pub type RawCard<'a> = (usize, &'a Pack);

impl<'a> std::fmt::Display for Card<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Card(value, Pack(_, pack)) = *self;

        f.write_fmt(format_args!("{}", pack[value.0 - 1]))
    }
}
