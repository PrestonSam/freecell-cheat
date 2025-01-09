use model::{Game, ACE, CLUBS, DIAMONDS, HEARTS, JACK, KING, QUEEN, SPADES};

mod model;

fn main() {
    let H = &HEARTS;
    let C = &CLUBS;
    let D = &DIAMONDS;
    let S = &SPADES;

    let mut game = Game::new([
        vec![ (02,    H), (10,    C), (QUEEN, D), (JACK,  C), (06,    C), (03,    H), (03,    D), ],
        vec![ (06,    S), (04,    C), (03,    C), (07,    S), (09,    D), (08,    H), (ACE,   H), ],
        vec![ (ACE,   C), (09,    S), (QUEEN, H), (KING,  H), (02,    D), (02,    S), (04,    H), ],
        vec![ (06,    D), (05,    S), (10,    D), (QUEEN, C), (07,    D), (05,    H), (10,    H), ],
        vec![             (06,    H), (KING,  S), (07,    H), (07,    C), (05,    D), (11,    S), ],
        vec![             (10,    S), (ACE,   S), (03,    S), (KING,  D), (JACK,  D), (JACK,  H), ],
        vec![             (05,    C), (KING,  C), (09,    C), (04,    D), (QUEEN, S), (08, D),    ],
        vec![             (04,    S), (08,    C), (08,    S), (02,    C), (ACE,   D), (09, H),    ],
    ]);

    println!("Initial game\n{game}");

    game.from_column_to_column().unwrap();
    println!("Moved stack from column to column\n{game}");

    game.from_column_to_reserve().unwrap();
    println!("Moved card from column to reserve\n{game}");

    game.from_reserve_to_column().unwrap();
    println!("Moved card from reserve to column\n{game}");

    game.from_column_to_foundation().unwrap();
    println!("Moved card from column to foundation\n{game}")
}
