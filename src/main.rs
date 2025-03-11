use itertools::Itertools;
use model::{Game, ParentLocations, Suit, ACE, JACK, KING, QUEEN};

mod model;
mod utils;

fn main() {
    let (H, C, D, S) = (Suit::Hearts, Suit::Clubs, Suit::Diamonds, Suit::Spades);

    let game = Game::new([
        vec![ (02,    H), (10,    C), (QUEEN, D), (JACK,  C), (06,    C), (03,    H), (03,    D), ],
        vec![ (06,    S), (04,    C), (03,    C), (07,    S), (09,    D), (08,    H), (ACE,   H), ],
        vec![ (ACE,   C), (09,    S), (QUEEN, H), (KING,  H), (02,    D), (02,    S), (04,    H), ],
        vec![ (06,    D), (05,    S), (10,    D), (QUEEN, C), (07,    D), (05,    H), (10,    H), ],
        vec![             (06,    H), (KING,  S), (07,    H), (07,    C), (05,    D), (11,    S), ],
        vec![             (10,    S), (ACE,   S), (03,    S), (KING,  D), (JACK,  D), (JACK,  H), ],
        vec![             (05,    C), (KING,  C), (09,    C), (04,    D), (QUEEN, S), (08,    D), ],
        vec![             (04,    S), (08,    C), (08,    S), (02,    C), (ACE,   D), (09,    H), ],
    ]);

    println!("GAME\n{game}\nGAME\n\n");

    let sorted_cards_and_parents = game.find_parents_for_bottom_cards()
        .into_iter()
        .sorted_by_key(|(_ ,parents)| parents.min_distance());

    for card_and_parents in sorted_cards_and_parents {
        if let (card, ParentLocations::HasParents(p0, p1)) = card_and_parents {
            println!("Consider grabbing {card}  and moving it as follows");

            let best_parent = [p0, p1].into_iter()
                .min_by_key(|p| p.get_distance())
                .unwrap();

            game.show_cards(&[&best_parent]);
        } else {
            println!("Oh it's a king look {}", card_and_parents.0)
        }
    }
}
