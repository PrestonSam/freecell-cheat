use std::cell::RefCell;
use std::rc::Rc;
use std::{ops::Index, slice::Iter};
use std::fmt::Write;
use derive_more::From;
use itertools::Itertools;

use crate::{
    model::{card::{Card, ProximateCard, RawCard, CARD_BACK_CHAR, NUMBER_OF_CARDS_IN_PACK}, error::GameError, CardLocation},
    utils::{FlatTranspose, Ternary}
};
use super::pickables::{CardMove, FindProxPair, HoldsCard, HoldsStack, PickableCard, PickableStack, StackMove};



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
    position: TableauPosition,
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
    pub fn new(position: usize, raw_cards: Vec<RawCard>) -> Self {
        let cards_iter = raw_cards.into_iter()
            .rev()
            .map(|(value, pack)| Card::new(value, pack));
        
        let mut cards_vec = Vec::with_capacity(MAX_NUMBER_OF_CARDS_IN_COLUMN);
        cards_vec.splice(.., cards_iter);

        Self {
            position: position.into(),
            cards: cards_vec,
            state_token: StateToken::new()
        }
    }

    pub fn iter(&self) -> Iter<'_, Card>{
        self.cards.iter()
    }

    fn get_top_card(&self) -> Option<&Card> {
        self.cards.last()
    }

    pub fn get_card_pick(&self) -> Option<PickableCard> {
        self.get_top_card()
            .map(|c| {
                let loc = TableauCardLocation(
                    self.position,
                    ColumnDepth { depth: 0, column_size: self.len() }
                );

                PickableCard::new(c, loc)
            })
    }

    pub fn get_largest_stack_pick(&self) -> Option<PickableStack> {
        (1..=self.len())
            .map_while(|n| self.can_pick_stack(n))
            .last()
    } 

    pub(super) fn at_depth(&self, depth: ColumnDepth) -> &Card {
        self.cards
            .iter()
            .nth_back(depth.depth)
            .expect("Invalid depth for column")
    }

    pub fn len(&self) -> usize {
        self.cards.len()
    }

    fn is_playable_pair(parent: &Card, child: &Card) -> bool {
        return parent.is_complementary_pack(child)
            && parent.is_playable_pair_bigger(child);
    }

    fn make_depth(&self, depth: usize) -> ColumnDepth {
        ColumnDepth { column_size: self.len(), depth }
    }
}

impl HoldsCard for Column {
    fn try_get_card_pick(&self) -> Option<PickableCard> {
        self.get_top_card()
            .map(|card| PickableCard::new(card, TableauCardLocation(self.position, self.make_depth(0))))
    }
    
    fn try_get_card_move(&self, picked_card: &PickableCard) -> Option<CardMove> {
        self.get_top_card()
            .is_none_or(|last_card| Self::is_playable_pair(&last_card, &picked_card.0))
            .then(|| CardMove::new(picked_card, TableauCardLocation(self.position, self.make_depth(0))))
    }
    
    fn pick_card(&mut self) -> Result<Card, GameError> {
        self.cards
            .pop()
            .ok_or(GameError::TriedToPickEmptyColumn)
    }

    fn take_card_from(&mut self, from: &mut dyn HoldsCard) -> Result<(), GameError> {
        self.cards.push(from.pick_card()?);

        Ok(())
    }
}

impl HoldsStack for Column {
    fn can_pick_stack(&self, pick_size: usize) -> Option<PickableStack> {
        if self.cards.len() < pick_size { return None; }

        if pick_size == 1  {
            return self.get_top_card()
                .map(|last| PickableStack::new(last, TableauCardLocation(self.position, self.make_depth(0)), pick_size))
        }

        self.cards.iter()
            .rev()
            .take(pick_size)
            // Get last child, assuming all cards in stack are playable pairs with neighbours
            .tuple_windows()
            .map(|(child, parent)| Self::is_playable_pair(parent, child).then_some(child))
            .reduce(|prev, cur| { prev.and(cur) })?
            .map(|last| PickableStack::new(last, TableauCardLocation(self.position, self.make_depth(pick_size - 1)), pick_size))
    }
    
    fn can_put_stack(&self, picked_stack: &PickableStack) -> Option<StackMove> {
        self.get_top_card()
            .is_none_or(|last_card| Self::is_playable_pair(last_card, &picked_stack.deepest_card))
            .then(|| StackMove::new(picked_stack, TableauCardLocation(self.position, self.make_depth(0))))
    }

    fn pick_stack(&mut self, pick: PickableStack) -> Result<Vec<Card>, GameError> {
        let card_count = self.cards.len();

        if card_count < pick.size {
            return Err(GameError::InsufficientCardsForStackPick { stack_size: pick.size });
        }
    
        Ok(self.cards.split_off(card_count - pick.size))
    }

    fn take_stack_from<T: HoldsStack>(&mut self, from: &mut T, pick: PickableStack) -> Result<(), GameError> {
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



pub const NUMBER_OF_COLUMNS_IN_TABLEAU: usize = 8;


#[derive(Debug, Clone, Copy, PartialEq, Eq, From)]
pub struct TableauPosition(usize);

#[derive(PartialEq, Eq, Clone)]
pub struct TableauCardLocation(TableauPosition, ColumnDepth);

impl PartialOrd for TableauCardLocation {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.1.partial_cmp(&other.1)
    }
}

impl Ord for TableauCardLocation {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.1.cmp(&other.1)
    }
}

impl std::fmt::Debug for TableauCardLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("TableauCardLocation(Pos({}), Depth({}))", self.0.0, self.1.depth))
    }
}

impl TableauCardLocation {
    pub fn get_distance(&self) -> usize {
        self.1.depth
    }

    fn matches_from_base(&self, pos: usize, inverted_depth: usize) -> bool {
        let Self(TableauPosition(cl_pos), ColumnDepth { depth: cl_depth, column_size }) = self;
        let max_inverted_depth = *column_size - 1;

        *cl_pos == pos
            && inverted_depth <= max_inverted_depth
            && *cl_depth == (max_inverted_depth - inverted_depth)
    }

    pub fn get_child_locations(&self) -> Vec<TableauCardLocation> {
        (0..self.1.depth)
            .map(|depth| TableauCardLocation(self.0, ColumnDepth { column_size: self.1.column_size, depth }))
            .collect()
    }

    pub fn position(&self) -> TableauPosition {
        self.0
    }
}

impl From<TableauCardLocation> for CardLocation {
    fn from(value: TableauCardLocation) -> Self {
        Self::Tableau(value)
    }
}

pub struct Tableau([Rc<RefCell<Column>>; NUMBER_OF_COLUMNS_IN_TABLEAU]);

impl Tableau {
    pub fn top_cards(&self) -> Vec<Card> {
        self.0.iter()
            .filter_map(|col| col.borrow().get_top_card().cloned())
            .collect()
    }

    pub fn get_valid_card_picks(&self) -> Vec<PickableCard> {
        self.0.iter()
            .filter_map(|c| c.borrow().get_card_pick())
            .collect_vec()
    }

    pub fn get_valid_card_puts(&self, pick: &PickableCard) -> Vec<CardMove>  {
        self.0.iter()
            .filter_map(|column| column.borrow().try_get_card_move(pick))
            .collect()
    }

    pub fn get_valid_stack_picks(&self) -> Vec<PickableStack> {
        self.0.iter()
            .filter_map(|c| c.borrow().get_largest_stack_pick())
            .collect_vec()
    }

    pub(in crate::model) fn show_cards<'a>(&'a self, positions: Vec<&'a TableauCardLocation>) -> TableauShownCards<'a> {
        TableauShownCards(self, positions)
    }
}

impl Index<&TableauPosition> for Tableau {
    type Output = Rc<RefCell<Column>>;

    fn index(&self, index: &TableauPosition) -> &Self::Output {
        &self.0[index.0]
    }
}

impl FindProxPair<TableauCardLocation> for Tableau {
    fn find_prox_pair(&self, prox_card: &ProximateCard) -> Ternary<TableauCardLocation> {
        let mut cards_iter = self.0
            .iter()
            .enumerate()
            .flat_map(|(tableau_position, column)| {
                column.borrow().find_prox_pair(prox_card)
                    .into_iter()
                    .map(move |depth| TableauCardLocation(TableauPosition(tableau_position), depth))
            });

        match (cards_iter.next(), cards_iter.next()) {
            (None, _) => Ternary::None,
            (Some(fst_card), Some(snd_card)) => Ternary::Two(fst_card, snd_card),
            (Some(fst_card), None) => Ternary::One(fst_card),
        }
    }
}

impl<'a> From<[Vec<RawCard>; NUMBER_OF_COLUMNS_IN_TABLEAU]> for Tableau {
    fn from(raw_columns: [Vec<RawCard>; NUMBER_OF_COLUMNS_IN_TABLEAU]) -> Self {
        let mut position = 0;

        Self(raw_columns.map(|c| {
            let column = Rc::new(RefCell::new(Column::new(position, c)));
            position += 1;
            column
        }))
    }
}

impl std::fmt::Display for Tableau {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let nested_iter = self.0
            .iter()
            .map(|col| col.borrow().iter().cloned().collect_vec().into_iter()) // TODO this was a workaround due to introduction of RefCell
            .flat_transpose()
            .chunks(self.0.len());

        for sub_iter in nested_iter.into_iter() {
            f.write_char(' ')?;

            for val in sub_iter {
                match val {
                    Some(card) => f.write_fmt(format_args!("{card} "))?,
                    None => f.write_fmt(format_args!("  "))?,
                }
            }

            f.write_char('\n')?;
        }

        Ok(())
    }
}

pub struct TableauShownCards<'a>(&'a Tableau, Vec<&'a TableauCardLocation>);

impl std::fmt::Display for TableauShownCards<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(tableau, positions) = self;
        // Need to condense the positions
        // Might be worth insisting in the model that such positions are nested
        // Which is probably for the best anyway, as that's how they're procured to begin with
        let grid_iter = tableau.0
            .iter()
            .map(|col| col.borrow().iter().cloned().collect_vec().into_iter()) // TODO this was a workaround due to introduction of RefCell
            .flat_transpose()
            .chunks(tableau.0.len());

        // Depth is back to front. This is a difficult issue to solve, as I have deliberately obscured the length of each iterator.
        for (depth, row_iter) in grid_iter.into_iter().enumerate() {
            print!(" ");

            for (pos, val) in row_iter.enumerate() {
                let is_in_positions = positions.iter().any(|l| (*l).matches_from_base(pos, depth));

                match (is_in_positions, val) {
                    (true, Some(card)) => f.write_fmt(format_args!("{card} ")),
                    (false, Some(_)) => f.write_fmt(format_args!("{CARD_BACK_CHAR} ")),
                    (_, None) => f.write_fmt(format_args!("  ")),
                }?;
            }

            f.write_str("\n")?;
        }

        Ok(())
    }
}