use std::fmt::Write;

use super::{card::{Card, FuzzyCard, RawCard}, error::GameError, stacks::{Column, ColumnDepth, EquivalentPairColumnDepths, Foundation, FoundationDepth, FoundationPosition, Reserve, ReservePosition, MAX_NUMBER_OF_CARDS_IN_COLUMN}};

const NUMBER_OF_COLUMNS_IN_TABLEAU: usize = 8;

#[derive(Debug, Clone, Copy)]
pub struct TableauPosition(usize);

pub struct Tableau<'data>([Column<'data>; NUMBER_OF_COLUMNS_IN_TABLEAU]);

impl<'data> Tableau<'data> {
    pub fn new(columns: [Column<'data>; NUMBER_OF_COLUMNS_IN_TABLEAU]) -> Self {
        Self(columns)
    }

    pub fn at(&mut self, column: &TableauPosition) -> Result<&mut Column<'data>, GameError> {
        self.0
            .get_mut(column.0)
            .ok_or_else(|| GameError::NoSuchColumn(column.0))
    }

    fn find_equivalent_pair(&self, f_card: &FuzzyCard) -> Option<EquivalentPairLocations> {
        self.0
            .iter()
            .enumerate()
            .find_map(|(tableau_position, column)| {
                let pair_depths = match column.find_equivalent_pair(f_card)? {
                    EquivalentPairColumnDepths::Two(fst_depth, snd_depth) =>
                        EquivalentPairLocations::Two(
                            CardLocation::Tableau { position: TableauPosition(tableau_position), depth: fst_depth },
                            CardLocation::Tableau { position: TableauPosition(tableau_position), depth: snd_depth }),
                    
                    EquivalentPairColumnDepths::One(depth) =>
                        EquivalentPairLocations::One(
                            CardLocation::Tableau { position: TableauPosition(tableau_position), depth }),
                };
                
                Some(pair_depths)
            })
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
// #[derive(Debug)]



enum EquivalentPairLocations {
    One(CardLocation),
    Two(CardLocation, CardLocation),
}

enum CardLocation {
    Reserve { position: ReservePosition }, // Thinking about it, permanent references to the reserve etc are going to influence how we can make references later. Better to store the index only
    Foundation { position: FoundationPosition , depth: FoundationDepth},
    Tableau { position: TableauPosition, depth: ColumnDepth },
}

// #[derive(Debug)]
// struct CardIndex {
//     clubs: [PlacedCard; NUMBER_OF_CARDS_IN_PACK],
//     spades: [PlacedCard; NUMBER_OF_CARDS_IN_PACK],
//     hearts: [PlacedCard; NUMBER_OF_CARDS_IN_PACK],
//     diamonds: [PlacedCard; NUMBER_OF_CARDS_IN_PACK],
// }

// impl CardIndex {
//     // You 100% on the index idea?
//     // You'll have to make sure that the index is continually updated
//     // Then, on the other hand, it'll just be swaps won't it
//     // If you move a stack it's quite a number of updates...
//     // Although it's a headache, it seems more efficient than searching through the entire game every time.
//     // Although that's true, I don't think that the effect is as pronounced as you claim.
//     // Your computer is obviously going to chew through that work as though it's nothing.
//     // Alright for the time being we'll use a search with no index, then, as it'll get me to a v1 sooner.
//     fn find_parents<'data>(child: &Card<'data>) -> (PlacedCard, PlacedCard) {
//         todo!()
//     }
// }

pub struct Game<'data> {
    tableau: Tableau<'data>,
    reserve: Reserve<'data>,
    foundation: Foundation<'data>,
}

fn get_mut_refs<'a, T>(arr: &'a mut [T], fst_idx: usize, snd_idx: usize) -> Option<(&'a mut T, &'a mut T)> {
    let fst_pick_idx = fst_idx + 1;
    if arr.len() < fst_pick_idx { return None; }

    let (fst_slice, snd_slice) = arr.split_at_mut_checked(fst_pick_idx)?;
    let fst_val = fst_slice.last_mut()?;

    let snd_pick_idx = snd_idx - fst_idx - 1 /* Don't understand why decrement is necessary but code fails without it */;
    let (_, snd_slice) = snd_slice.split_at_mut_checked(snd_pick_idx)?;
    let snd_val = snd_slice.first_mut()?;

    Some((fst_val, snd_val))
}

impl<'data> Game<'data> {
    pub fn new(raw_columns: [Vec<RawCard<'data>>; NUMBER_OF_COLUMNS_IN_TABLEAU]) -> Self {
        Self {
            tableau: Tableau::from(raw_columns),
            reserve: Reserve::new(), 
            foundation: Foundation::new()
        }
    }

    fn find_parents(&self, card: &Card<'data>) -> Option<CardLocation> {
        let f_card = card.get_parent_data()?;
        
        // TODO move "Color, Value" into a descriptive struct
        let maybe_card_locations_in_tableau = self.tableau.find_equivalent_pair(&f_card);
        let maybe_card_locations_in_foundation = self.foundation.find_equivalent_pair(&f_card);
        let maybe_card_locations_in_reserve = self.reserve.find_equivalent_pair(&f_card);

        todo!()
    }
}

impl<'a> std::fmt::Display for Game<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Game { tableau, reserve, foundation } = self;
        f.write_fmt(format_args!("{}  {}\n\n{}", reserve, foundation, tableau))
    }
}
