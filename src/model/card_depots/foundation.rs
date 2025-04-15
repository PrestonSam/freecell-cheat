use std::{cell::RefCell, ops::Index, rc::Rc};

use derive_more::From;
use itertools::Itertools;

use crate::{model::{card::{Card, ProximateCard, BLANK_CARD_CHAR}, error::GameError, CardLocation}, utils::Ternary};

use super::{pickables::{CardMove, HoldsCard, PickableCard}, FindProxPair};



#[derive(Debug)]
pub struct FoundationStack(FoundationPosition, Vec<Card>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct FoundationDepth {
    pub foundation_size: usize,
    pub depth: usize,
}

impl FoundationDepth {
    fn depth_index(&self) -> usize {
        self.foundation_size - self.depth
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, From)]
pub struct FoundationPosition(usize);

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FoundationCardLocation(FoundationPosition, FoundationDepth);

impl FoundationCardLocation {
    pub fn get_distance(&self) -> usize {
        self.1.depth
    }

    pub fn position(&self) -> FoundationPosition {
        self.0
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

impl From<FoundationCardLocation> for CardLocation {
    fn from(value: FoundationCardLocation) -> Self {
        Self::Foundation(value)
    }
}

impl FoundationStack {
    fn new(position: usize) -> Self {
        Self(position.into(), vec![])
    }

    fn get_top_card(&self) -> Option<&Card> {
        self.1.last()
    }

    fn find_card(&self, f_card: &ProximateCard) -> Option<FoundationDepth> {
        self.1.iter()
            .rev()
            .position(|card| card.matches_prox_card(f_card))
            .map(|depth| self.make_depth(depth))
    }

    fn find_equivalent_pair(&self, prox_card: &ProximateCard) -> Ternary<FoundationDepth> {
        match (self.find_card(prox_card), self.find_card(prox_card)) {
            (None, _) => Ternary::None,
            (Some(first_card), Some(second_card)) => Ternary::Two(first_card, second_card),
            (Some(first_card), None) => Ternary::One(first_card),
        }
    }

    fn is_playable_pair(parent: &Card, child: &Card) -> bool {
        return parent.is_same_pack(child)
            && parent.is_playable_pair_bigger(child);
    }

    pub fn len(&self) -> usize {
        self.1.len()
    }

    fn make_depth(&self, depth: usize) -> FoundationDepth {
        FoundationDepth { foundation_size: self.len(), depth }
    }
}

impl std::fmt::Display for FoundationStack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let FoundationStack(_, stack) = self;

        if let Some(card) = stack.last() {
            f.write_fmt(format_args!("{card} "))
        } else {
            f.write_fmt(format_args!("{BLANK_CARD_CHAR} "))
        }
    }
}

impl<'data> HoldsCard for FoundationStack {
    fn try_get_card_pick(&self) -> Option<PickableCard> {
        self.1.last()
            .map(|card| PickableCard::new(card, FoundationCardLocation(self.0, self.make_depth(0))))
    }
    
    fn try_get_card_move(&self, picked_card: &PickableCard) -> Option<CardMove> {
        self.1.last()
            .is_some_and(|last_card| Self::is_playable_pair(last_card, &picked_card.0))
            .then(|| CardMove::new(picked_card, FoundationCardLocation(self.0, self.make_depth(0))))
    }

    fn pick_card(&mut self) -> Result<Card, GameError> {
        self.1
            .pop()
            .ok_or(GameError::TriedToPickEmptyFoundationStack)
    }

    fn take_card_from(&mut self, from: &mut dyn HoldsCard) -> Result<(), GameError> {
        self.1.push(from.pick_card()?);

        Ok(())
    }
}

#[derive(Debug)]
pub struct Foundation([Rc<RefCell<FoundationStack>>; 4]);

impl Foundation {
    pub fn new() -> Self {
        Self([0,1,2,3].map(|pos| Rc::new(RefCell::new(FoundationStack::new(pos)))))
    }

    pub(in crate::model) fn show_cards<'a>(&'a self, positions: Vec<&'a FoundationCardLocation>) -> FoundationShownCards<'a> {
        FoundationShownCards(self, positions)
    }

    pub fn get_valid_card_picks(&self) -> Vec<PickableCard> {
        self.0.iter()
            .filter_map(|s|
                s.borrow().1.last().as_ref()
                    .map(|card|
                        PickableCard::new(
                            card,
                            FoundationCardLocation(s.borrow().0, s.borrow().make_depth(0)))))
            .collect_vec()
    }

    pub fn get_valid_card_puts(&self, pick: &PickableCard) -> Vec<CardMove>  {
        self.0.iter()
            .filter_map(|stack| stack.borrow().try_get_card_move(pick))
            .collect()
    }
}

impl Index<&FoundationPosition> for Foundation {
    type Output = Rc<RefCell<FoundationStack>>;

    fn index(&self, index: &FoundationPosition) -> &Self::Output {
        &self.0[index.0]
    }
}

impl FindProxPair<FoundationCardLocation> for Foundation {
    fn find_prox_pair(&self, prox_card: &ProximateCard) -> Ternary<FoundationCardLocation> {
        self.0.iter()
            .enumerate()
            .find_map(|(pos, stack)|
                match stack.borrow().find_equivalent_pair(prox_card) {
                    Ternary::None =>
                        None,

                    Ternary::One(fst_depth) =>
                        Some(Ternary::One(FoundationCardLocation(pos.into(), fst_depth))),

                    Ternary::Two(fst_depth, snd_depth) =>
                        Some(Ternary::Two(
                            FoundationCardLocation(pos.into(), fst_depth),
                            FoundationCardLocation(pos.into(), snd_depth))),
                })
            .unwrap_or(Ternary::None)
    }
}

impl std::fmt::Display for Foundation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for stack in self.0.iter() {
            stack.borrow().fmt(f)?;
        }

        Ok(())
    }
}

pub struct FoundationShownCards<'a>(&'a Foundation, Vec<&'a FoundationCardLocation>);

impl std::fmt::Display for FoundationShownCards<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(foundation, positions) = self;

        foundation.0.iter()
            .enumerate()
            .map(|(pos, slot)| {
                match positions.iter().find(|c| c.0.0 == pos) {
                    Some(card_location) => slot.borrow().1.iter()
                        .nth_back(card_location.1.depth)
                        .expect("Invalid depth for foundation stack")
                        .get_char(),
                    None => BLANK_CARD_CHAR,
                }
            })
            .interleave_shortest(std::iter::repeat(' '))
            .map(|c| f.write_fmt(format_args!("{c}")))
            .collect()
    }
}