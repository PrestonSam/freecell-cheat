pub const NUMBER_OF_CARDS_IN_PACK: usize = 14;

pub const ACE: usize = 1;

pub const JACK: usize = 11;

pub const QUEEN: usize = 12;

pub const KING: usize = 13;

pub const BLANK_CARD_CHAR: char = '🃟';

pub const CARD_BACK_CHAR: char = '🂠';


#[derive(PartialEq, Eq, Copy, Clone, PartialOrd, Ord)]
pub struct Rank(usize);

impl std::fmt::Debug for Rank {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            1 => f.write_str("Ace"),
            11 => f.write_str("Jack"),
            12 => f.write_str("Queen"),
            13 => f.write_str("King"),
            n => f.write_fmt(format_args!("{n}")),
        }
    }
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Suit {
    Spades,
    Clubs,
    Hearts,
    Diamonds,
}

impl Suit {
    pub fn get_colour(&self) -> Color {
        match &self {
            Suit::Spades | Suit::Clubs => Color::Black,
            Suit::Hearts | Suit::Diamonds => Color::Red,
        }
    }

    pub fn is_opposing_color(&self, other: &Suit) -> bool {
        self.get_colour() != other.get_colour()
    }

    pub fn get_opposing_suits(&self) -> (Suit, Suit) {
        match self.get_colour() {
            Color::Red => (Suit::Spades, Suit::Clubs),
            Color::Black => (Suit::Hearts, Suit::Diamonds),
        }
    }
}




struct Pack(Suit, [char; 14]);

impl Pack {
    fn get(pack_type: &Suit) -> &'static Self {
        match pack_type {
            Suit::Spades => &SPADES,
            Suit::Clubs => &CLUBS,
            Suit::Hearts => &HEARTS,
            Suit::Diamonds => &DIAMONDS,
        }
    }
}


const SPADES: Pack
    = Pack(Suit::Spades, [ '🂡', '🂢', '🂣', '🂤', '🂥', '🂦', '🂧', '🂨', '🂩', '🂪', '🂫', '🂬', '🂭', '🂮', ]);

const CLUBS: Pack
    = Pack(Suit::Clubs, [ '🃑', '🃒', '🃓', '🃔', '🃕', '🃖', '🃗', '🃘', '🃙', '🃚', '🃛', '🃜', '🃝', '🃞', ]);

const HEARTS: Pack
    = Pack(Suit::Hearts, [ '🂱', '🂲', '🂳', '🂴', '🂵', '🂶', '🂷', '🂸', '🂹', '🂺', '🂻', '🂼', '🂽', '🂾', ]);

const DIAMONDS: Pack
    = Pack(Suit::Diamonds, [ '🃁', '🃂', '🃃', '🃄', '🃅', '🃆', '🃇', '🃈', '🃉', '🃊', '🃋', '🃌', '🃍', '🃎', ]);

impl Pack {
    fn get_card_char(&self, value: usize) -> char {
        self.1[value]
    }
}

impl std::fmt::Debug for Pack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Pack")
            .field(&self.0)
            .finish()
    }
}



#[derive(Clone)]
pub struct ProximateCard {
    pub color: Color,
    pub rank: Rank,
}

impl ProximateCard {
    pub fn matches(&self, card: &Card) -> bool {
        card.get_color() == self.color
            && card.get_value() == self.rank
    }
}

impl std::fmt::Debug for ProximateCard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("ProximateCard(A {:?} {:?})", self.color, self.rank))
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Card(Rank, Suit);

impl std::fmt::Debug for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Card({} of {:?})", self.0.0, self.1))
    }
}

impl<'data> Card {
    pub fn new(rank: usize, suit: Suit) -> Self {
        Self(Rank(rank), suit)
    }

    pub fn get_color(&self) -> Color {
        self.1.get_colour()
    }

    pub fn get_value(&self) -> Rank {
        self.0
    }

    pub fn get_char(&self) -> char {
        Pack::get(&self.1)
            .get_card_char(self.0.0)
    }

    pub fn is_same_pack(&self, other: &Card) -> bool {
        let Card(_, s_pack) = self;
        let Card(_, o_pack) = other;

        std::ptr::eq(s_pack, o_pack)
    }

    pub fn is_complementary_pack(&self, other: &Card) -> bool {
        let Card(_, s_type) = self;
        let Card(_, o_type) = other;

        s_type.is_opposing_color(o_type)
    }

    pub fn is_playable_pair_smaller(&self, other: &Card) -> bool {
        let Card(s_rank, _) = *self;
        let Card(o_rank, _) = *other;

        s_rank.0 + 1 == o_rank.0
    }

    pub fn is_playable_pair_bigger(&self, other: &Card) -> bool {
        let Card(s_rank, _) = *self;
        let Card(o_rank, _) = *other;

        o_rank.0 + 1 == s_rank.0
    }

    pub fn matches_prox_card(&self, prox_card: &ProximateCard) -> bool {
        self.get_color() == prox_card.color
            && self.get_value() == prox_card.rank
    }

    pub fn get_parent_data(&self) -> Option<ProximateCard> {
        (self.0.0 < KING)
            .then(|| ProximateCard { color: self.get_color().get_opposing_color(), rank: Rank(self.0.0 + 1) })
    }

    pub fn get_parents(&self) -> Option<(Card, Card)> {
        let Card(rank, suit) = self;

        (rank.0 < KING)
            .then(|| {
                let (fst_suit, snd_suit) = suit.get_opposing_suits();
                let parent_value = Rank(rank.0 + 1);

                (Card(parent_value, fst_suit), Card(parent_value, snd_suit))
            })
    }
}

pub type RawCard = (usize, Suit);

impl<'a> std::fmt::Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Card(rank, suit) = self;

        f.write_fmt(format_args!("{}", Pack::get(&suit).get_card_char(rank.0 - 1)))
    }
}
