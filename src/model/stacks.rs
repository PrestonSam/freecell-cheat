use std::iter::once;

use itertools::Itertools;

use super::{card::{Card, RawCard}, error::GameError};

pub const MAX_NUMBER_OF_CARDS_IN_COLUMN: usize = 14 * 2;

#[derive(Debug)]
pub struct Column<'data> {
    cards: Vec<Card<'data>>,
}

pub trait HoldsCard<'data> {
    fn can_pick_card(&self) -> bool;

    fn can_put_card(&self, card: &Card<'data>) -> bool;

    fn pick_card(&mut self) -> Result<Card<'data>, GameError>;

    fn take_card_from<T: HoldsCard<'data>>(&mut self, from: &mut T) -> Result<(), GameError>;
}

pub trait HoldsStack<'data> {
    fn can_pick_stack(&self, pick_size: usize) -> bool;

    fn can_put_stack(&self, stack: &Vec<&Card<'data>>) -> bool;

    fn pick_stack(&mut self, pick_size: usize) -> Result<Vec<Card<'data>>, GameError>;

    fn take_stack_from<T: HoldsStack<'data>>(&mut self, from: &mut T, pick_size: usize) -> Result<(), GameError>;
}


impl<'data> Column<'data> {
    pub fn new(raw_cards: Vec<RawCard<'data>>) -> Self {
        let cards_iter = raw_cards.into_iter()
            .rev()
            .map(|(value, pack)| Card::new(value, pack));
        
        let mut cards_vec = Vec::with_capacity(MAX_NUMBER_OF_CARDS_IN_COLUMN);
        cards_vec.splice(.., cards_iter);

        Self { cards: cards_vec }
    }

    // Exists for <Tableau as Display>
    pub(super) fn get_card(&self, index: usize) -> Option<&Card<'data>> {
        self.cards
            .get(index)
    }

    fn is_playable_pair(parent: &Card<'data>, child: &Card<'data>) -> bool {
        return parent.is_complementary_pack(child)
            && parent.is_playable_pair_bigger(child);
    }
}

impl<'data> HoldsCard<'data> for Column<'data> {
    fn can_pick_card(&self) -> bool {
        !(self.cards.is_empty())
    }
    
    fn can_put_card(&self, card: &Card<'data>) -> bool {
        self.cards.last()
            .is_none_or(|last_card| Column::is_playable_pair(last_card, card))
    }
    
    
    fn pick_card(&mut self) -> Result<Card<'data>, GameError> {
        self.cards
            .pop()
            .ok_or(GameError::TriedToPickEmptyColumn)
    }

    fn take_card_from<T: HoldsCard<'data>>(&mut self, from: &mut T) -> Result<(), GameError> {
        Ok(self.cards.push(from.pick_card()?))
    }
}

impl<'data> HoldsStack<'data> for Column<'data> {
    fn can_pick_stack(&self, pick_size: usize) -> bool {
        if pick_size < self.cards.len() {
            return false;
        }

        self.cards.iter()
            .rev()
            .take(pick_size)
            .tuple_windows()
            .all(|(child, parent)| Self::is_playable_pair(parent, child))
    }
    
    fn can_put_stack(&self, stack: &Vec<&Card<'data>>) -> bool {
        if let Some(last_card) = self.cards.iter().last() {
            return once(last_card)
                .chain(stack.iter().map(|v| *v))
                .tuple_windows()
                .all(|(parent, child)| Self::is_playable_pair(parent, child))
        }

        return true;
    }

    fn pick_stack(&mut self, pick_size: usize) -> Result<Vec<Card<'data>>, GameError> {
        let card_count = self.cards.len();

        if card_count < pick_size {
            return Err(GameError::InsufficientCardsForStackPick { stack_size: pick_size });
        }
    
        Ok(self.cards.split_off(card_count - pick_size))
    }

    fn take_stack_from<T: HoldsStack<'data>>(&mut self, from: &mut T, pick_size: usize) -> Result<(), GameError> {
        let mut picked = from.pick_stack(pick_size)?;

        Ok(self.cards.append(&mut picked))
    }    
}

#[derive(Debug)]
pub struct ReserveSlot<'data>(Option<Card<'data>>);

impl<'data> Default for ReserveSlot<'data> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<'data> HoldsCard<'data> for ReserveSlot<'data> {
    fn can_pick_card(&self) -> bool {
        self.0.is_some()
    }
    
    fn can_put_card(&self, _card: &Card<'data>) -> bool {
        self.0.is_none()
    }

    fn pick_card(&mut self) -> Result<Card<'data>, GameError> {
        self.0
            .take()
            .ok_or(GameError::TriedToPickEmptyReserveSlot)
    }

    fn take_card_from<T: HoldsCard<'data>>(&mut self, from: &mut T) -> Result<(), GameError> {
        if self.0.is_some() {
            return Err(GameError::ReserveSlotIsOccupied);
        }

        self.0 = Some(from.pick_card()?);

        Ok(())
    }    
}

#[derive(Debug)]
pub struct Reserve<'data>([ReserveSlot<'data>; 4]);

impl<'data> Reserve<'data> {
    pub fn new() -> Self {
        Self(Default::default())
    }

    pub fn at(&mut self, slot: usize) -> Result<&mut ReserveSlot<'data>, GameError> {
        self.0
            .get_mut(slot)
            .ok_or_else(|| GameError::NoSuchReserveSlot(slot))
    }
}

impl std::fmt::Display for Reserve<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for maybe_card in self.0.iter() {
            if let ReserveSlot(Some(card)) = maybe_card {
                f.write_fmt(format_args!("{card} "))?;
            } else {
                f.write_str("🂠 ")?;
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct FoundationStack<'data>(Vec<Card<'data>>);

impl<'data> Default for FoundationStack<'data> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl std::fmt::Display for FoundationStack<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let FoundationStack(stack) = self;

        if let Some(card) = stack.last() {
            f.write_fmt(format_args!("{card} "))?;
        } else {
            f.write_str("🂠 ")?;
        }

        Ok(())
    }
}

impl<'data> HoldsCard<'data> for FoundationStack<'data> {
    fn can_pick_card(&self) -> bool {
        !(self.0.is_empty())
    }
    
    fn can_put_card(&self, card: &Card<'data>) -> bool {
        if let Some(last_card) = self.0.last() {
            return last_card.is_same_pack(card)
                && last_card.is_playable_pair_smaller(card);
        }

        return true;
    }

    fn pick_card(&mut self) -> Result<Card<'data>, GameError> {
        self.0
            .pop()
            .ok_or(GameError::TriedToPickEmptyFoundationStack)
    }

    fn take_card_from<T: HoldsCard<'data>>(&mut self, from: &mut T) -> Result<(), GameError> {
        Ok(self.0.push(from.pick_card()?))
    }
}

#[derive(Debug)]
pub struct Foundation<'data>([FoundationStack<'data>; 4]);

impl<'data> Foundation<'data> {
    pub fn new() -> Self {
        Self(Default::default())
    }

    pub fn at(&mut self, stack: usize) -> Result<&mut FoundationStack<'data>, GameError> {
        self.0
            .get_mut(stack)
            .ok_or_else(|| GameError::NoSuchFoundationStack(stack))
    }
}

impl std::fmt::Display for Foundation<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for stack in self.0.iter() {
            stack.fmt(f)?;
        }

        Ok(())
    }
}
