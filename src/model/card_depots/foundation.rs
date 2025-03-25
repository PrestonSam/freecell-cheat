use itertools::Itertools;

use crate::{model::{card::{Card, ProximateCard, BLANK_CARD_CHAR}, error::GameError}, utils::Ternary};

use super::{pickables::{HoldsCard, PickableCard}, FindProxPair};



#[derive(Debug, Default)]
pub struct FoundationStack(Vec<Card>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct FoundationDepth(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FoundationPosition(usize);

#[derive(Debug, PartialEq, Eq)]
pub struct FoundationCardLocation(FoundationPosition, FoundationDepth);

impl FoundationCardLocation {
    pub fn get_distance(&self) -> usize {
        self.1.0
    }
}

impl PartialOrd for FoundationCardLocation {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.1.partial_cmp(&other.1)
    }
}

impl Ord for FoundationCardLocation {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.1.cmp(&other.1)
    }
}

impl FoundationStack {
    fn find_card(&self, f_card: &ProximateCard) -> Option<FoundationDepth> {
        self.0.iter()
            .rev()
            .position(|card| card.matches_prox_card(f_card))
            .map(FoundationDepth)
    }

    fn find_equivalent_pair(&self, prox_card: &ProximateCard) -> Ternary<FoundationDepth> {
        match (self.find_card(prox_card), self.find_card(prox_card)) {
            (None, _) => Ternary::None,
            (Some(first_card), Some(second_card)) => Ternary::Two(first_card, second_card),
            (Some(first_card), None) => Ternary::One(first_card),
        }
    }
}

impl std::fmt::Display for FoundationStack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let FoundationStack(stack) = self;

        if let Some(card) = stack.last() {
            f.write_fmt(format_args!("{card} "))?;
        } else {
            f.write_fmt(format_args!("{BLANK_CARD_CHAR} "))?;
        }

        Ok(())
    }
}

impl<'data> HoldsCard for FoundationStack {
    fn can_pick_card(&self) -> Option<PickableCard> {
        self.0
            .last()
            .map(|card| PickableCard { card: card.clone() })
    }
    
    fn can_put_card(&self, picked_card: &PickableCard) -> bool {
        if let Some(last_card) = self.0.last() {
            return last_card.is_same_pack(&picked_card.card)
                && last_card.is_playable_pair_smaller(&picked_card.card);
        }

        true
    }

    fn pick_card(&mut self) -> Result<Card, GameError> {
        self.0
            .pop()
            .ok_or(GameError::TriedToPickEmptyFoundationStack)
    }

    fn take_card_from<T: HoldsCard>(&mut self, from: &mut T) -> Result<(), GameError> {
        self.0.push(from.pick_card()?);

        Ok(())
    }
}

#[derive(Debug)]
pub struct Foundation([FoundationStack; 4]);

impl Foundation {
    pub fn new() -> Self {
        Self(Default::default())
    }

    pub(in crate::model) fn show_cards(&self, positions: Vec<&FoundationCardLocation>) {
        self.0.iter()
            .enumerate()
            .map(|(pos, slot)| {
                match positions.iter().find(|c| c.0.0 == pos) {
                    Some(card_location) => slot.0.iter()
                        .nth_back(card_location.1.0)
                        .expect("Invalid depth for foundation stack")
                        .get_char(),
                    None => BLANK_CARD_CHAR,
                }
            })
            .interleave_shortest(std::iter::repeat(' '))
            .for_each(|c| print!("{c}"));
    }
}

impl FindProxPair<FoundationCardLocation> for Foundation {
    fn find_prox_pair(&self, prox_card: &ProximateCard) -> Ternary<FoundationCardLocation> {
        self.0
            .iter()
            .enumerate()
            .find_map(|(pos, stack)| {
                match stack.find_equivalent_pair(prox_card) {
                    Ternary::None =>
                        None,

                    Ternary::One(fst_depth) =>
                        Some(Ternary::One(FoundationCardLocation(FoundationPosition(pos), fst_depth))),

                    Ternary::Two(fst_depth, snd_depth) =>
                        Some(Ternary::Two(
                            FoundationCardLocation(FoundationPosition(pos), fst_depth),
                            FoundationCardLocation(FoundationPosition(pos), snd_depth))),
                }
            })
            .unwrap_or(Ternary::None)
    }
}

impl std::fmt::Display for Foundation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for stack in self.0.iter() {
            stack.fmt(f)?;
        }

        Ok(())
    }
}
