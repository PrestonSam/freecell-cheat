use std::fmt::Write;

use super::{card::RawCard, error::GameError, stacks::{Column, Foundation, HoldsCard, HoldsStack, Reserve, MAX_NUMBER_OF_CARDS_IN_COLUMN}};

const NUMBER_OF_COLUMNS_IN_TABLEAU: usize = 8;

pub struct Tableau<'data>([Column<'data>; NUMBER_OF_COLUMNS_IN_TABLEAU]);

impl<'data> Tableau<'data> {
    pub fn new(columns: [Column<'data>; NUMBER_OF_COLUMNS_IN_TABLEAU]) -> Self {
        Self(columns)
    }

    pub fn at(&mut self, column: usize) -> Result<&mut Column<'data>, GameError> {
        self.0
            .get_mut(column)
            .ok_or_else(|| GameError::NoSuchColumn(column))
    }
}

impl<'a> From<[Vec<RawCard<'a>>; NUMBER_OF_COLUMNS_IN_TABLEAU]> for Tableau<'a> {
    fn from(raw_columns: [Vec<RawCard<'a>>; NUMBER_OF_COLUMNS_IN_TABLEAU]) -> Self {
        Self(raw_columns.map(Column::new))
    }
}

impl std::fmt::Display for Tableau<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Tableau(columns) = self;

        for row in 0..MAX_NUMBER_OF_CARDS_IN_COLUMN {
            let mut something_written = false;
            f.write_str(" ")?;

            for stack in columns.iter() {
                if let Some(card) = stack.get_card(row) {
                    something_written = true;

                    f.write_fmt(format_args!("{card} "))?;
                } else {
                    f.write_str("  ")?;
                }
            }

            if !something_written {
                return Ok(());
            }
            
            f.write_char('\n')?;
        }

        Ok(())
    }
}

// Build an index of all the cards
// Should probably just be a set of two arrays
// One for red cards, the other black.
// The cards should be stored in pairs, based on their value.
// Thus, each array should contain 28 cards, for at total of 54. Ace has a value of 0, while jack has 11, queen 12 and king 13
// Each entry of the array should be a struct containing the card, the index of the column in which the card is stored and a reference to the column itself
// To get the references right without using mutation the order should be as follows:
// Cards (already constructed in the parameters)
// Columns (immutable references to cards without destructuring parameter)
// Index (reference to the columns, their positions in the tableau and ownership of the cards)

// Hold up you've forgotten about the reserve and the foundation
// Agh
// Alrighty enums I guess
enum PlacedCard {
    Reserve { position: usize }, // Thinking about it, permanent references to the reserve etc are going to influence how we can make references later. Better to store the index only
    Foundation { position: usize },
}

pub struct Game<'data> {
    tableau: Tableau<'data>,
    reserve: Reserve<'data>,
    foundation: Foundation<'data>,
}

impl<'a> Game<'a> {
    pub fn new(raw_columns: [Vec<RawCard<'a>>; NUMBER_OF_COLUMNS_IN_TABLEAU]) -> Self {
        Self {
            tableau: Tableau::from(raw_columns),
            reserve: Reserve::new(), 
            foundation: Foundation::new()
        }
    }

    pub fn from_column_to_column<'pick>(&'pick mut self) -> Result<(), GameError> {
        let ([first_col, ..], [second_col, ..]) = self.tableau.0.split_at_mut(1) else { panic!() };

        second_col.take_stack_from(first_col, 4)
    }

    pub fn from_column_to_reserve<'pick>(&'pick mut self) -> Result<(), GameError> {
        let column = self.tableau.at(0)?;
        let reserve_slot = self.reserve.at(2)?;

        reserve_slot.take_card_from(column)
    }

    pub fn from_reserve_to_column<'pick>(&'pick mut self) -> Result<(), GameError> {
        let reserve_slot = self.reserve.at(2)?;
        let column = self.tableau.at(3)?;

        column.take_card_from(reserve_slot)
    }

    pub fn from_column_to_foundation(&mut self) -> Result<(), GameError> {
        let column = self.tableau.at(1)?;
        let foundation = self.foundation.at(2)?;

        foundation.take_card_from(column)
    }
}

impl<'a> std::fmt::Display for Game<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Game { tableau, reserve, foundation } = self;
        f.write_fmt(format_args!("{}  {}\n\n{}", reserve, foundation, tableau))
    }
}
