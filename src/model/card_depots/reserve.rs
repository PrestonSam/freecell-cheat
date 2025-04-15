use std::{cell::RefCell, fmt::Write, ops::Index, rc::Rc};

use derive_more::From;
use itertools::Itertools;

use crate::{model::{card::{Card, ProximateCard, BLANK_CARD_CHAR, CARD_BACK_CHAR}, error::GameError, CardLocation}, utils::Ternary};
use super::{pickables::{CardMove, HoldsCard, PickableCard}, FindProxPair};


#[derive(Debug)]
pub struct ReserveSlot(ReservePosition, Option<Card>);

impl ReserveSlot {
    fn new(position: usize) -> Self {
        Self(position.into(), None)
    }

    fn matches_prox_card(&self, prox_card: &ProximateCard) -> bool {
        self.1.as_ref()
            .is_some_and(|c| c.matches_prox_card(prox_card))
    }
}

impl HoldsCard for ReserveSlot {
    fn try_get_card_pick(&self) -> Option<PickableCard> {
        self.1.as_ref()
            .map(|card| PickableCard::new(card, ReserveCardLocation(self.0)))
    }
    
    fn try_get_card_move(&self, pick: &PickableCard) -> Option<CardMove> {
        self.1.is_none()
            .then(|| CardMove::new(pick, ReserveCardLocation(self.0)))
    }

    fn pick_card(&mut self) -> Result<Card, GameError> {
        self.1
            .take()
            .ok_or(GameError::TriedToPickEmptyReserveSlot)
    }

    fn take_card_from(&mut self, from: &mut dyn HoldsCard) -> Result<(), GameError> {
        if self.1.is_some() {
            return Err(GameError::ReserveSlotIsOccupied);
        }

        self.1 = Some(from.pick_card()?);

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, From)]
pub struct ReservePosition(usize);

#[derive(Debug, PartialEq, Eq, Clone, From)]
pub struct ReserveCardLocation(ReservePosition);

impl ReserveCardLocation {
    fn new(position: usize) -> Self {
        Self(ReservePosition(position))
    }

    pub fn position(&self) -> ReservePosition {
        self.0
    }
}

impl From<ReserveCardLocation> for CardLocation {
    fn from(value: ReserveCardLocation) -> Self {
        CardLocation::Reserve(value)
    }
}

#[derive(Debug)]
pub struct Reserve([Rc<RefCell<ReserveSlot>>; 4]);

impl Reserve {
    pub fn new() -> Self {
        Self([0,1,2,3].map(|pos| Rc::new(RefCell::new(ReserveSlot::new(pos)))))
    }

    pub(in crate::model) fn show_cards<'a>(&'a self, positions: Vec<&'a ReserveCardLocation>) -> ReserveShownCards<'a> {
        ReserveShownCards(self, positions)
    }

    pub fn get_valid_card_picks(&self) -> Vec<PickableCard> {
        self.0.iter()
            .filter_map(|c|
                c.borrow().1
                    .as_ref()
                    .map(|card| PickableCard::new(card, ReserveCardLocation::new(c.borrow().0.0))))
            .collect()
    }

    pub fn get_valid_card_puts(&self, pick: &PickableCard) -> Vec<CardMove>  {
        self.0.iter()
            .filter_map(|c| c.borrow().try_get_card_move(pick))
            .collect()
    }
}

impl Index<&ReservePosition> for Reserve {
    type Output = Rc<RefCell<ReserveSlot>>;

    fn index(&self, index: &ReservePosition) -> &Self::Output {
        &self.0[index.0]
    }
}

impl FindProxPair<ReserveCardLocation> for Reserve {
    fn find_prox_pair(&self, prox_card: &ProximateCard) -> Ternary<ReserveCardLocation> {
        let mut slots = self.0.iter()
            .enumerate()
            .filter_map(|(pos, slot)| slot.borrow().matches_prox_card(prox_card).then_some(pos));

        match (slots.next(), slots.next()) {
            (None, _) =>
                Ternary::None,

            (Some(fst_pos), None) =>
                Ternary::One(ReserveCardLocation::new(fst_pos)),

            (Some(fst_pos), Some(snd_pos)) =>
                Ternary::Two(
                    ReserveCardLocation::new(fst_pos),
                    ReserveCardLocation::new(snd_pos)),
        }
    }
}

impl std::fmt::Display for Reserve {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for maybe_card in self.0.iter() {
            if let Some(card) = &maybe_card.borrow().1 {
                f.write_fmt(format_args!("{card} "))?;
            } else {
                f.write_char(BLANK_CARD_CHAR)?;
                f.write_char(' ')?;
            }
        }

        Ok(())
    }
}

pub struct ReserveShownCards<'a>(&'a Reserve, Vec<&'a ReserveCardLocation>);

impl std::fmt::Display for ReserveShownCards<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(reserve, positions) = self;

        let positions: Vec<_> = positions.iter()
            .map(|p| p.0.0)
            .collect();

        reserve.0.iter()
            .enumerate()
            .map(|(n, slot)|
                match (positions.contains(&n), slot.borrow().1.as_ref()) {
                    (true, Some(card)) => card.get_char(),
                    (true, None) => CARD_BACK_CHAR,
                    (false, _) => BLANK_CARD_CHAR,
                })
            .interleave_shortest(std::iter::repeat(' '))
            .map(|c| f.write_fmt(format_args!("{c}")))
            .collect()
    }
}