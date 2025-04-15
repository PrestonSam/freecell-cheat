use std::{cell::RefCell, rc::Rc};

use crate::utils::{TernaryVal, ThruplePartitionMap};

use super::{
    card::{Card, RawCard},
    card_depots::{
        CardMove, FindProxPair, Foundation, FoundationCardLocation, HoldsCard, PickableCard, PickableStack, Reserve, ReserveCardLocation, Tableau, TableauCardLocation, NUMBER_OF_COLUMNS_IN_TABLEAU
    }, error::GameError
};


#[derive(Debug)]
pub enum ParentLocations {
    HasParents(CardLocation, CardLocation),
    King,
}

impl ParentLocations {
    pub fn min_distance(&self) -> Option<usize> {
        match self {
            ParentLocations::HasParents(p0, p1) =>
                Some(p0.get_distance().min(p1.get_distance())),

            ParentLocations::King =>
                None,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum CardLocation {
    Reserve(ReserveCardLocation),
    Foundation(FoundationCardLocation),
    Tableau(TableauCardLocation),
}

impl CardLocation {
    pub fn get_distance(&self) -> usize {
        match self {
            CardLocation::Reserve(_) => 0,
            CardLocation::Foundation(foundation_card_location) => foundation_card_location.get_distance(),
            CardLocation::Tableau(tableau_card_location) => tableau_card_location.get_distance(),
        }
    }
}

impl Ord for CardLocation {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.get_distance()
            .cmp(&other.get_distance())
    }
}

impl PartialOrd for CardLocation {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

pub struct Game {
    tableau: Tableau,
    reserve: Reserve,
    foundation: Foundation,
}

fn get_mut_refs<T>(arr: &mut [T], fst_idx: usize, snd_idx: usize) -> Option<(&mut T, &mut T)> {
    let fst_pick_idx = fst_idx + 1;
    if arr.len() < fst_pick_idx { return None; }

    let (fst_slice, snd_slice) = arr.split_at_mut_checked(fst_pick_idx)?;
    let fst_val = fst_slice.last_mut()?;

    let snd_pick_idx = snd_idx - fst_idx - 1 /* Don't understand why decrement is necessary but code fails without it */;
    let (_, snd_slice) = snd_slice.split_at_mut_checked(snd_pick_idx)?;
    let snd_val = snd_slice.first_mut()?;

    Some((fst_val, snd_val))
}

impl Game {
    pub fn new(raw_columns: [Vec<RawCard>; NUMBER_OF_COLUMNS_IN_TABLEAU]) -> Self {
        Self {
            tableau: Tableau::from(raw_columns),
            reserve: Reserve::new(), 
            foundation: Foundation::new()
        }
    }
    
    pub fn find_parents_for_top_cards(&self) -> Vec<(Card, ParentLocations)> {
        self.tableau.top_cards()
            .into_iter()
            .map(|card| {
                let parents = self.find_parents(&card);

                (card, parents)
            })
            .collect()
    }

    // Produces None if is King (might be worth explicitly encoding that into the model)
    fn find_parents(&self, card: &Card) -> ParentLocations {
        let prox_card = match card.get_parent_data() {
            Some(prox_card) => prox_card,
            None => return ParentLocations::King,
        };

        let tableau_parents = self.tableau.find_prox_pair(&prox_card);
        let foundation_parents = self.foundation.find_prox_pair(&prox_card);
        let reserve_parents = self.reserve.find_prox_pair(&prox_card);

        let mut locations_iter = tableau_parents.into_iter().map(CardLocation::Tableau)
            .chain(foundation_parents.into_iter().map(CardLocation::Foundation))
            .chain(reserve_parents.into_iter().map(CardLocation::Reserve));

        match (locations_iter.next(), locations_iter.next()) {
            (Some(fst), Some(snd)) =>
                ParentLocations::HasParents(fst, snd),

            locations =>
                panic!("Unable to find two cards matching proximate card {prox_card:?}\nInstead found {locations:?}"),
        }
    }

    pub fn show_cards<'a>(&'a self, card_locations: &'a[&'a CardLocation]) -> GameShownCards<'a> {
        GameShownCards(self, card_locations)
    }

    pub fn get_valid_stack_picks(&self) -> Vec<PickableStack> {
        self.tableau.get_valid_stack_picks()
    }

    pub fn get_valid_card_picks(&self) -> Vec<PickableCard> {
        let mut out = self.reserve.get_valid_card_picks();
        out.append(&mut self.foundation.get_valid_card_picks());
        out.append(&mut self.tableau.get_valid_card_picks());
        out
    }

    pub fn get_valid_card_puts(&self, pick: &PickableCard) -> Vec<CardMove> {
        let mut out = self.reserve.get_valid_card_puts(&pick);
        out.append(&mut self.foundation.get_valid_card_puts(&pick));
        out.append(&mut self.tableau.get_valid_card_puts(&pick));
        out
    }

    fn get_card_holder(&self, location: &CardLocation) -> Rc<RefCell<dyn HoldsCard>> {
        match &location {
            CardLocation::Reserve(reserve_card_location) => self.reserve[&reserve_card_location.position()].clone(),
            CardLocation::Foundation(foundation_card_location) => self.foundation[&foundation_card_location.position()].clone(),
            CardLocation::Tableau(tableau_card_location) => self.tableau[&tableau_card_location.position()].clone(),
        }
    }

    pub fn move_card(&mut self, card_move: CardMove) -> Result<(), GameError> {
        let from = self.get_card_holder(&card_move.from);
        let to = self.get_card_holder(&card_move.to);

        to.borrow_mut()
            .take_card_from(&mut *from.borrow_mut())
    }

    pub fn move_stack(&mut self, pick: PickableStack) {
        // Pick stack
        // Put stack
        // Done
        // Might produce a game error
    }
}

impl<'a> std::fmt::Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Game { tableau, reserve, foundation } = self;

        f.write_fmt(format_args!("{reserve}  {foundation}\n\n{tableau}"))
    }
}

pub struct GameShownCards<'a>(&'a Game, &'a [&'a CardLocation]);

impl std::fmt::Display for GameShownCards<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(game, card_locations) = self;

        let (reserve_card_locations, foundation_card_locations, tableau_card_locations): (Vec<_>, Vec<_>, Vec<_>) = card_locations.into_iter()
            .thruple_partition_map(|v| match v {
                CardLocation::Reserve(reserve_card_location) => TernaryVal::Left(reserve_card_location),
                CardLocation::Foundation(foundation_card_location) => TernaryVal::Middle(foundation_card_location),
                CardLocation::Tableau(tableau_card_location) => {
                    // TODO this is a hack I threw together to get quick results
                    let mut location = tableau_card_location.get_child_locations();
                    location.push(tableau_card_location.clone());

                    TernaryVal::Right(location)
                },
            });

        let reserve_cards = game.reserve.show_cards(reserve_card_locations);
        let foundation_cards = game.foundation.show_cards(foundation_card_locations);
        // TODO this is a consequence of the hack. There are many allocations and I doubt I need them all
        let tableau_cards = game.tableau.show_cards(tableau_card_locations.iter().flatten().collect());

        f.write_fmt(format_args!("{}  {}\n\n{}", reserve_cards, foundation_cards, tableau_cards))
    }
}