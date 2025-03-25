use std::slice::Iter;

use itertools::Itertools;

use crate::{model::{card::{Card, ProximateCard, RawCard, NUMBER_OF_CARDS_IN_PACK}, error::GameError}, utils::Ternary};

use super::{pickables::{HoldsCard, HoldsStack, PickableCard, PickableStack}, FindProxPair};



pub const MAX_NUMBER_OF_CARDS_IN_COLUMN: usize = NUMBER_OF_CARDS_IN_PACK * 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StateToken(usize);

impl StateToken {
    fn new() -> Self {
        Self(0)
    }

    fn into_next(self) -> Self {
        Self(self.0 + 1)
    }
}

#[derive(Debug)]
pub struct Column {
    cards: Vec<Card>,
    state_token: StateToken,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord)]
pub struct ColumnDepth {
    pub column_size: usize,
    pub depth: usize,
}

impl PartialOrd for ColumnDepth {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.depth.partial_cmp(&other.depth)
    }
}

impl<'data> Column {
    pub fn new(raw_cards: Vec<RawCard>) -> Self {
        let cards_iter = raw_cards.into_iter()
            .rev()
            .map(|(value, pack)| Card::new(value, pack));
        
        let mut cards_vec = Vec::with_capacity(MAX_NUMBER_OF_CARDS_IN_COLUMN);
        cards_vec.splice(.., cards_iter);

        Self {
            cards: cards_vec,
            state_token: StateToken::new()
        }
    }

    pub fn iter(&self) -> Iter<'_, Card>{
        self.cards.iter()
    }

    pub fn get_largest_stack_pick(&self) -> Option<PickableStack<StateToken>> {
        (1..=self.len())
            .map_while(|n| self.can_pick_stack(n))
            .last()
    } 

    // First card
    pub fn first_card(&self) -> Option<&Card> {
        self.cards.last()
    }

    pub(super) fn at_depth(&self, depth: ColumnDepth) -> &Card {
        self.cards.iter().nth_back(depth.depth).expect("Invalid depth for column")
    }

    pub fn len(&self) -> usize {
        self.cards.len()
    }

    fn is_playable_pair(parent: &Card, child: &Card) -> bool {
        return parent.is_complementary_pack(child)
            && parent.is_playable_pair_bigger(child);
    }
}

impl HoldsCard for Column {
    fn can_pick_card(&self) -> Option<PickableCard> {
        self.cards
            .last()
            .map(|card| PickableCard { card: card.clone() })
    }
    
    fn can_put_card(&self, picked_card: &PickableCard) -> bool {
        self.cards.last()
            .is_none_or(|last_card| Self::is_playable_pair(last_card, &picked_card.card))
    }
    
    fn pick_card(&mut self) -> Result<Card, GameError> {
        self.cards
            .pop()
            .ok_or(GameError::TriedToPickEmptyColumn)
    }

    fn take_card_from<T: HoldsCard>(&mut self, from: &mut T) -> Result<(), GameError> {
        self.cards.push(from.pick_card()?);

        Ok(())
    }
}

impl HoldsStack<StateToken> for Column {
    fn can_pick_stack(&self, pick_size: usize) -> Option<PickableStack<StateToken>> {
        if self.cards.len() < pick_size { return None; }

        if pick_size == 1  {
            return self.cards.last()
                .map(|last| PickableStack {
                    deepest_card: last.clone(),
                    size: pick_size,
                    state_token: self.state_token
                })
        }

        self.cards.iter()
            .rev()
            .take(pick_size)
            // Get last child, assuming all cards in stack are playable pairs with neighbours
            .tuple_windows()
            .map(|(child, parent)| Self::is_playable_pair(parent, child).then_some(child))
            .reduce(|prev, cur| { prev.and(cur) })?
            .map(|last| PickableStack {
                deepest_card: last.clone(),
                size: pick_size,
                state_token: self.state_token,
            })
    }
    
    fn can_put_stack(&self, picked_stack: &PickableStack<StateToken>) -> bool {
        self.cards
            .iter()
            .last()
            .is_none_or(|last_card| Self::is_playable_pair(last_card, &picked_stack.deepest_card))
    }

    fn pick_stack(&mut self, pick: PickableStack<StateToken>) -> Result<Vec<Card>, GameError> {
        let card_count = self.cards.len();

        if card_count < pick.size {
            return Err(GameError::InsufficientCardsForStackPick { stack_size: pick.size });
        }
    
        Ok(self.cards.split_off(card_count - pick.size))
    }

    fn take_stack_from<T: HoldsStack<StateToken>>(&mut self, from: &mut T, pick: PickableStack<StateToken>) -> Result<(), GameError> {
        let mut picked = from.pick_stack(pick)?;

        self.cards.append(&mut picked);
        Ok(())
    }    
}

impl<'data> FindProxPair<ColumnDepth> for Column {
    fn find_prox_pair(&self, prox_pair: &ProximateCard) -> Ternary<ColumnDepth> {
        let column_size = self.cards.len();
        let mut cards = self.cards.iter()
            .rev()
            .enumerate()
            .filter(|(_, card)| prox_pair.matches(card))
            .map(|(depth, _)| ColumnDepth { depth, column_size });

        match (cards.next(), cards.next()) {
            (Some(first_card), Some(second_card)) => Ternary::Two(first_card, second_card),
            (Some(first_card), None) => Ternary::One(first_card),
            (None, _) => Ternary::None,
        }
    }
}
