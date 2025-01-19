use itertools::Itertools;

use super::{card::{Card, Color, FuzzyCard, RawCard, Value, NUMBER_OF_CARDS_IN_PACK}, error::GameError};

pub const MAX_NUMBER_OF_CARDS_IN_COLUMN: usize = NUMBER_OF_CARDS_IN_PACK * 2;

#[derive(Debug)]
pub struct Column<'data> {
    cards: Vec<Card<'data>>,
}

#[derive(Debug, Clone, Copy)]
pub struct ColumnDepth(usize);

#[derive(Debug, Clone, Copy)]
pub enum EquivalentPairColumnDepths {
    One(ColumnDepth),
    Two(ColumnDepth, ColumnDepth),
}

#[derive(Debug)]
pub struct PickableCard<'data> { card: Card<'data> }

// Interesting idea, but you don't actually confirm that the pick matches the column or the game state at the time.
// You could probably bind those together with this move cleverly, but presently I'm not able to figure that out.
#[derive(Debug)]
pub struct PickableStack<'data> { deepest_card: Card<'data>, size: usize }

pub trait HoldsCard<'data> {
    fn can_pick_card(&self) -> Option<PickableCard<'data>>;

    fn can_put_card(&self, picked_card: &PickableCard<'data>) -> bool;

    fn pick_card(&mut self) -> Result<Card<'data>, GameError>;

    fn take_card_from<T: HoldsCard<'data>>(&mut self, from: &mut T) -> Result<(), GameError>;
}

pub trait HoldsStack<'data> {
    fn can_pick_stack(&self, pick_size: usize) -> Option<PickableStack<'data>>;

    fn can_put_stack(&self, picked_stack: &PickableStack<'data>) -> bool;

    fn pick_stack(&mut self, pick: PickableStack<'data>) -> Result<Vec<Card<'data>>, GameError>;

    fn take_stack_from<T: HoldsStack<'data>>(&mut self, from: &mut T, pick: PickableStack<'data>) -> Result<(), GameError>;
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

    // Created for <Tableau as Display>
    pub(super) fn get_card(&self, index: usize) -> Option<&Card<'data>> {
        self.cards
            .get(index)
    }

    pub fn get_largest_stack_pick(&self) -> Option<PickableStack<'data>> {
        (1..=self.len())
            .map_while(|n| self.can_pick_stack(n))
            .last()
    }

    fn find_card(&self, f_card: &FuzzyCard) -> Option<ColumnDepth> {
        self.cards.iter()
            .rev()
            .position(|card|
                card.get_color() == f_card.color
                    && card.get_value() == f_card.value)
            .map(ColumnDepth)
    }

    pub fn find_equivalent_pair(&self, f_card: &FuzzyCard) -> Option<EquivalentPairColumnDepths> {
        self.find_card(f_card)
            .map(|first_card| {
                match self.find_card(f_card) {
                    Some(second_card) => EquivalentPairColumnDepths::Two(first_card, second_card),
                    None => EquivalentPairColumnDepths::One(first_card),
                }
            })
    }

    pub fn len(&self) -> usize {
        self.cards.len()
    }

    fn is_playable_pair(parent: &Card<'data>, child: &Card<'data>) -> bool {
        return parent.is_complementary_pack(child)
            && parent.is_playable_pair_bigger(child);
    }
}

impl<'data> HoldsCard<'data> for Column<'data> {
    fn can_pick_card(&self) -> Option<PickableCard<'data>> {
        self.cards
            .last()
            .map(|card| PickableCard { card: card.clone() })
    }
    
    fn can_put_card(&self, picked_card: &PickableCard<'data>) -> bool {
        self.cards.last()
            .is_none_or(|last_card| Self::is_playable_pair(last_card, &picked_card.card))
    }
    
    fn pick_card(&mut self) -> Result<Card<'data>, GameError> {
        self.cards
            .pop()
            .ok_or(GameError::TriedToPickEmptyColumn)
    }

    fn take_card_from<T: HoldsCard<'data>>(&mut self, from: &mut T) -> Result<(), GameError> {
        self.cards.push(from.pick_card()?);

        Ok(())
    }
}

impl<'data> HoldsStack<'data> for Column<'data> {
    fn can_pick_stack(&self, pick_size: usize) -> Option<PickableStack<'data>> {
        if self.cards.len() < pick_size { return None; }
        if pick_size == 1  {
            return self.cards.last().map(|last| PickableStack { deepest_card: last.clone(), size: pick_size } )
        }

        self.cards.iter()
            .rev()
            .take(pick_size)
            // Get last child, assuming all cards in stack are playable pairs with neighbours
            .tuple_windows()
            .map(|(child, parent)| Self::is_playable_pair(parent, child).then_some(child))
            .reduce(|prev, cur| { prev.and(cur) })?
            .map(|last| PickableStack { deepest_card: last.clone(), size: pick_size })
    }
    
    fn can_put_stack(&self, picked_stack: &PickableStack<'data>) -> bool {
        self.cards
            .iter()
            .last()
            .is_none_or(|last_card| Self::is_playable_pair(last_card, &picked_stack.deepest_card))
    }

    fn pick_stack(&mut self, pick: PickableStack<'data>) -> Result<Vec<Card<'data>>, GameError> {
        let card_count = self.cards.len();

        if card_count < pick.size {
            return Err(GameError::InsufficientCardsForStackPick { stack_size: pick.size });
        }
    
        Ok(self.cards.split_off(card_count - pick.size))
    }

    fn take_stack_from<T: HoldsStack<'data>>(&mut self, from: &mut T, pick: PickableStack<'data>) -> Result<(), GameError> {
        let mut picked = from.pick_stack(pick)?;

        self.cards.append(&mut picked);
        Ok(())
    }    
}

#[derive(Debug, Default)]
pub struct ReserveSlot<'data>(Option<Card<'data>>);

impl<'data> HoldsCard<'data> for ReserveSlot<'data> {
    fn can_pick_card(&self) -> Option<PickableCard<'data>> {
        self.0.as_ref()
            .map(|card| PickableCard { card: card.clone() })
    }
    
    fn can_put_card(&self, _: &PickableCard<'data>) -> bool {
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

#[derive(Debug, Clone, Copy)]
pub struct ReservePosition(usize);

#[derive(Debug)]
pub struct Reserve<'data>([ReserveSlot<'data>; 4]);

impl<'data> Reserve<'data> {
    pub fn new() -> Self {
        Self(Default::default())
    }

    pub fn at(&mut self, slot: ReservePosition) -> Result<&mut ReserveSlot<'data>, GameError> {
        self.0
            .get_mut(slot.0)
            .ok_or_else(|| GameError::NoSuchReserveSlot(slot.0))
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

#[derive(Debug, Default)]
pub struct FoundationStack<'data>(Vec<Card<'data>>);

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
    fn can_pick_card(&self) -> Option<PickableCard<'data>> {
        self.0
            .last()
            .map(|card| PickableCard { card: card.clone() })
    }
    
    fn can_put_card(&self, picked_card: &PickableCard<'data>) -> bool {
        if let Some(last_card) = self.0.last() {
            return last_card.is_same_pack(&picked_card.card)
                && last_card.is_playable_pair_smaller(&picked_card.card);
        }

        true
    }

    fn pick_card(&mut self) -> Result<Card<'data>, GameError> {
        self.0
            .pop()
            .ok_or(GameError::TriedToPickEmptyFoundationStack)
    }

    fn take_card_from<T: HoldsCard<'data>>(&mut self, from: &mut T) -> Result<(), GameError> {
        self.0.push(from.pick_card()?);

        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FoundationPosition(usize);

#[derive(Debug, Clone, Copy)]
pub struct FoundationDepth(usize);

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

    fn find_equivalent_pair(&self) -> Option<>
}

impl std::fmt::Display for Foundation<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for stack in self.0.iter() {
            stack.fmt(f)?;
        }

        Ok(())
    }
}
