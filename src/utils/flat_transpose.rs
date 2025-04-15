use std::ops::DerefMut;

pub struct FlatTransposed<C, I, T>
where
    C: DerefMut<Target = [I]>,
    I: Iterator<Item = T>,
{
    container: C,
    index: usize,
    child_had_next: bool,
}

impl<C, I, T> FlatTransposed<C, I, T>
where
    C: DerefMut<Target = [I]>,
    I: Iterator<Item = T>
{
    fn new(container: C) -> Self {
        Self { container, index: 0, child_had_next: false }
    }
}

pub trait FlatTranspose<C, I, T> {
    /// Cycle through each child iterator, emitting the next value from each.
    /// Continue to cycle until every child iterator produces None.
    ///
    /// The item for this iterator is `Option<T>`, meaning the outputted values are `Option<Option<T>>`
    /// This is because some of the child iterators are expected to finish earlier than others.
    /// This iterator only produces `None` if every child iterator produces None.
    /// ```
    /// let input = vec![
    ///     vec![ 'T', 'i', 't', 't', 't', 'a'],
    ///     vec![ 'r', 't', 'h', 'e', 'e', 't'],
    ///     vec![ 'a', 'i', 'o', 'r', ' ', 'i'],
    ///     vec![ 'n', 'o', 'u', 'm', 'a', 'o'],
    ///     vec![ 's', 'n', 't', 'e', 'l', 'n'],
    ///     vec![ 'p', ' ', ' ', 'd', 'l', 's'],
    ///     vec![ 'o', 'w', 'i', 'i', 'o', '!'],
    ///     vec![ 's', 'i', 'n', 'a', 'c']
    /// ];
    ///
    /// let str: String = input.iter()
    ///     .map(|v| v.iter())
    ///     .flat_transpose()
    ///     .filter_map(|v| v)
    ///     .collect();
    ///
    /// println!("{str}"); // Prints "Transposition without intermediate allocations!"
    /// ```
    fn flat_transpose(self) -> FlatTransposed<C, I, T>
    where
        Self: Sized,
        C: DerefMut<Target = [I]>,
        I: Iterator<Item = T>;
}

impl<C, I, T> Iterator for FlatTransposed<C, I, T>
where 
    Self: Sized,
    C: DerefMut<Target = [I]>,
    I: Iterator<Item = T>,
{
    type Item = Option<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.container.len() == 0 {
            return None;
        }

        let iter_next = self.container[self.index].next();
        self.child_had_next |= iter_next.is_some();
        self.index = (self.index + 1) % self.container.len();

        if self.index == 0 {
            if !self.child_had_next {
                return None
            }

            self.child_had_next = false;
        }

        Some(iter_next)
    }
}

impl<I, SI, T> FlatTranspose<Vec<SI>, SI, T> for I
where
    I: Iterator<Item = SI>,
    SI: Iterator<Item = T>,
{
    fn flat_transpose(self) -> FlatTransposed<Vec<SI>, SI, T>
    where
        Self: Sized,
        SI: Iterator<Item = T>
    {
        FlatTransposed::new(self.collect())
    }
}

#[test]
pub fn flat_transpose_iterates_until_all_columns_exhausted() {
    let input = [
        vec![ 'Y', ' ', 'e'           ],
        vec![ 'o', 's', ' ', 't', '!' ],
        vec![ 'u', 'e', 'i', '!'      ],
    ];

    let expected_output = [
        vec![ Some('Y'), Some('o'), Some('u') ],
        vec![ Some(' '), Some('s'), Some('e') ],
        vec![ Some('e'), Some(' '), Some('i') ],
        vec![ None,      Some('t'), Some('!') ],
        vec![ None,      Some('!'), None      ],
    ];

    let slice = input.into_iter()
        .map(|v| v.into_iter());

    for (found, expected) in slice.flat_transpose().zip(expected_output.into_iter().flatten()) {
        assert!(found == expected)
    }
}

#[test]
pub fn flat_transpose_iterates_when_some_columns_are_blank() {
    let input = [
        vec![                     ],
        vec![ 1, 2, 3, 4, 5       ],
        vec![ 7, 8, 9, 10, 11, 12 ],
        vec![                     ],
        vec![                     ],
        vec![                     ],
    ];

    let expected_output = [
        vec![ None, Some(1), Some(7),  None, None, None, ],
        vec![ None, Some(2), Some(8),  None, None, None, ],
        vec![ None, Some(3), Some(9),  None, None, None, ],
        vec![ None, Some(4), Some(10), None, None, None, ],
        vec![ None, Some(5), Some(11), None, None, None, ],
        vec![ None, None,    Some(12), None, None, None, ],
    ];

    let slice = input.into_iter()
        .map(|v| v.into_iter());
    
    for (found, expected) in slice.flat_transpose().zip(expected_output.into_iter().flatten()) {
        assert!(found == expected, "{found:?} is not {expected:?}")
    }
}
