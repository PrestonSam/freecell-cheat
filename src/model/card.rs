
pub struct Pack(PackType, [char; 14]);

#[derive(Debug)]
pub enum PackType {
    Spades,
    Clubs,
    Hearts,
    Diamonds,
}

#[derive(PartialEq, Eq)]
enum Color {
    Red,
    Black,
}

impl PackType {
    pub fn is_opposing_color(&self, other: &PackType) -> bool {
        fn get_colour(pack: &PackType) -> Color {
            match pack {
                PackType::Spades | PackType::Clubs => Color::Black,
                PackType::Hearts | PackType::Diamonds => Color::Red,
            }
        }

        get_colour(self) != get_colour(other)
    }
}

pub const ACE: usize = 1;

pub const JACK: usize = 11;

pub const QUEEN: usize = 12;

pub const KING: usize = 13;

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

pub struct Card<'a>(usize, &'a Pack);

impl<'a> std::fmt::Debug for Card<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Card")
            .field(&self.0)
            .field(&self.1.0)
            .finish()
    }
}

impl<'data> Card<'data> {
    pub fn new(value: usize, pack: &'data Pack) -> Self {
        Self(value, pack)
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

        s_value + 1 == o_value
    }

    pub fn is_playable_pair_bigger(&self, other: &Card<'data>) -> bool {
        let Card(s_value, _) = *self;
        let Card(o_value, _) = *other;

        o_value + 1 == s_value
    }
}

pub type RawCard<'a> = (usize, &'a Pack);

impl<'a> std::fmt::Display for Card<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Card(value, Pack(_, pack)) = *self;

        f.write_fmt(format_args!("{}", pack[value - 1]))
    }
}
