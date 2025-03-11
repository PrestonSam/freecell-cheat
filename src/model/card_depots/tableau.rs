use std::fmt::Write;
use itertools::Itertools;
use crate::{
    model::{card::{Card, ProximateCard, RawCard, CARD_BACK_CHAR}, CardLocation},
    utils::{FlatTranspose, Ternary}
};
use super::{commons::FindProxPair, Column, ColumnDepth};


pub const NUMBER_OF_COLUMNS_IN_TABLEAU: usize = 8;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TableauPosition(usize);

#[derive(PartialEq, Eq)]
pub struct TableauCardLocation(TableauPosition, ColumnDepth);

impl TableauCardLocation {
    pub fn get_distance(&self) -> usize {
        self.1.depth
    }
}

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
    fn matches_from_base(&self, pos: usize, inverted_depth: usize) -> bool {
        let Self(TableauPosition(cl_pos), ColumnDepth { depth: cl_depth, column_size }) = self;
        let max_inverted_depth = *column_size - 1;

        *cl_pos == pos
            && inverted_depth <= max_inverted_depth
            && *cl_depth == (max_inverted_depth - inverted_depth)
    }
}

pub struct Tableau([Column; NUMBER_OF_COLUMNS_IN_TABLEAU]);

impl Tableau {
    pub fn bottom_cards(&self) -> Vec<&Card> {
        self.0.iter()
            .filter_map(|col| col.first_card())
            .collect()
    }
    
    pub fn get_valid_picks(&self) -> Vec<CardLocation> {
        todo!()
    }

    pub(in crate::model) fn show_cards(&self, positions: Vec<&TableauCardLocation>) {
        // Need to condense the positions
        // Might be worth insisting in the model that such positions are nested
        // Which is probably for the best anyway, as that's how they're procured to begin with
        let mut col_iters = self.0
            .iter()
            .map(|col| col.iter())
            .collect::<Box<[_]>>();

        let grid_iter = col_iters
            .flat_transpose()
            .chunks(self.0.len());

        // Depth is back to front. This is a difficult issue to solve, as I have deliberately obscured the length of each iterator.
        for (depth, row_iter) in grid_iter.into_iter().enumerate() {
            print!(" ");

            for (pos, val) in row_iter.enumerate() {
                let is_in_positions = positions.iter().any(|l| l.matches_from_base(pos, depth));

                match (is_in_positions, val) {
                    (true, Some(card)) => print!("{card} "),
                    (false, Some(_)) => print!("{CARD_BACK_CHAR} "),
                    (_, None) => print!("  "),
                }
            }

            print!("\n")
        }
    }
}

impl FindProxPair<TableauCardLocation> for Tableau {
    fn find_prox_pair(&self, prox_card: &ProximateCard) -> Ternary<TableauCardLocation> {
        let mut cards_iter = self.0
            .iter()
            .enumerate()
            .flat_map(|(tableau_position, column)| {
                column.find_prox_pair(prox_card)
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
        Self(raw_columns.map(Column::new))
    }
}

impl std::fmt::Display for Tableau {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut col_iters = self.0
            .iter()
            .map(|col| col.iter())
            .collect_vec();

        let nested_iter = col_iters
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
