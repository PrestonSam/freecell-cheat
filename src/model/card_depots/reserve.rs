use std::fmt::Write;

use itertools::Itertools;

use crate::{model::{card::{Card, ProximateCard, BLANK_CARD_CHAR, CARD_BACK_CHAR}, error::GameError}, utils::Ternary};
use super::{pickables::{HoldsCard, PickableCard}, FindProxPair};


#[derive(Debug, Default)]
pub struct ReserveSlot(Option<Card>);

impl ReserveSlot {
    fn matches_prox_card(&self, prox_card: &ProximateCard) -> bool {
        self.0.as_ref()
            .is_some_and(|c| c.matches_prox_card(prox_card))
    }
}

impl HoldsCard for ReserveSlot {
    fn can_pick_card(&self) -> Option<PickableCard> {
        self.0.as_ref()
            .map(|card| PickableCard { card: card.clone() })
    }
    
    fn can_put_card(&self, _: &PickableCard) -> bool {
        self.0.is_none()
    }

    fn pick_card(&mut self) -> Result<Card, GameError> {
        self.0
            .take()
            .ok_or(GameError::TriedToPickEmptyReserveSlot)
    }

    fn take_card_from<T: HoldsCard>(&mut self, from: &mut T) -> Result<(), GameError> {
        if self.0.is_some() {
            return Err(GameError::ReserveSlotIsOccupied);
        }

        self.0 = Some(from.pick_card()?);

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReservePosition(usize);

#[derive(Debug, PartialEq, Eq)]
pub struct ReserveCardLocation(ReservePosition);

#[derive(Debug)]
pub struct Reserve([ReserveSlot; 4]);

impl Reserve {
    pub fn new() -> Self {
        Self(Default::default())
    }

    pub(in crate::model) fn show_cards(&self, positions: Vec<&ReserveCardLocation>) {
        let positions: Box<[_]> = positions.iter()
            .map(|p| p.0.0)
            .collect();

        self.0.iter()
            .enumerate()
            .map(|(n, slot)|
                match (positions.contains(&n), slot.0.as_ref()) {
                    (true, Some(card)) => card.get_char(),
                    (true, None) => CARD_BACK_CHAR,
                    (false, _) => BLANK_CARD_CHAR,
                })
            .interleave_shortest(std::iter::repeat(' '))
            .for_each(|c| print!("{c}"));
    }
}

impl FindProxPair<ReserveCardLocation> for Reserve {
    fn find_prox_pair(&self, prox_card: &ProximateCard) -> Ternary<ReserveCardLocation> {
        let mut slots = self.0.iter()
            .enumerate()
            .filter_map(|(pos, slot)| slot.matches_prox_card(prox_card).then_some(pos));

        match (slots.next(), slots.next()) {
            (None, _) =>
                Ternary::None,

            (Some(fst_pos), None) =>
                Ternary::One(ReserveCardLocation(ReservePosition(fst_pos))),

            (Some(fst_pos), Some(snd_pos)) =>
                Ternary::Two(
                    ReserveCardLocation(ReservePosition(fst_pos)),
                    ReserveCardLocation(ReservePosition(snd_pos))),
        }
    }
}

impl std::fmt::Display for Reserve {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for maybe_card in self.0.iter() {
            if let ReserveSlot(Some(card)) = maybe_card {
                f.write_fmt(format_args!("{card} "))?;
            } else {
                f.write_char(BLANK_CARD_CHAR)?;
                f.write_char(' ')?;
            }
        }

        Ok(())
    }
}
