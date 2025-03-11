use std::ops::DerefMut;

pub struct FlatTransposed<'a, C, I, T>
where
    C: DerefMut<Target = [I]>,
    I: Iterator<Item = T>,
{
    container: &'a mut C,
    index: usize,
    child_had_next: bool,
}

impl<'a, C, I, T> FlatTransposed<'a, C, I, T>
where
    C: DerefMut<Target = [I]>,
    I: Iterator<Item = T>
{
    fn new(container: &'a mut C) -> Self {
        Self { container, index: 0, child_had_next: false }
    }
}

pub trait FlatTranspose<'a, C, I, T> {
    fn flat_transpose(&'a mut self) -> FlatTransposed<'a, C, I, T>
    where
        Self: Sized,
        C: DerefMut<Target = [I]>,
        I: Iterator<Item = T>;
}

impl<'a, C, I, T> Iterator for FlatTransposed<'a, C, I, T>
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

impl<'a, C, I, T> FlatTranspose<'a, C, I, T> for C
where
    C: DerefMut<Target = [I]>,
    I: Iterator<Item = T>,
{
    /// Sequence ends after every child sequence in row produces None.
    fn flat_transpose(&'a mut self) -> FlatTransposed<'a, C, I, T> {
        FlatTransposed::new(self)
    }
}

#[test]
pub fn flat_transpose_iterates_until_all_columns_exhausted() {
    let input = [
        vec![ 'Y', ' ', 'e' ],
        vec![ 'o', 's', ' ', 't', '!' ],
        vec![ 'u', 'e', 'i', '!' ],
    ];

    let expected_output = [
        vec![ Some(&'Y'), Some(&'o'), Some(&'u') ],
        vec![ Some(&' '), Some(&'s'), Some(&'e') ],
        vec![ Some(&'e'), Some(&' '), Some(&'i') ],
        vec![ None,       Some(&'t'), Some(&'!') ],
        vec![ None,       Some(&'!'), None       ],
    ];

    let mut slice = input.iter()
        .map(|v| v.iter())
        .collect::<Box<[_]>>();

    for (found, expected) in slice.flat_transpose().zip(expected_output.into_iter().flatten()) {
        assert!(found == expected)
    }
}

#[test]
pub fn flat_transpose_iterates_when_some_columns_are_blank() {
    let input = [
        vec![],
        vec![ 1, 2, 3, 4, 5 ],
        vec![ 7, 8, 9, 10, 11, 12 ],
        vec![],
        vec![],
        vec![],
    ];

    let expected_output = [
        vec![ None, Some(&1), Some(&7),  None, None, None, ],
        vec![ None, Some(&2), Some(&8),  None, None, None, ],
        vec![ None, Some(&3), Some(&9),  None, None, None, ],
        vec![ None, Some(&4), Some(&10), None, None, None, ],
        vec![ None, Some(&5), Some(&11), None, None, None, ],
        vec![ None, None,     Some(&12), None, None, None, ],
    ];

    let mut slice = input.iter()
        .map(|v| v.iter())
        .collect::<Box<[_]>>();
    
    for (found, expected) in slice.flat_transpose().zip(expected_output.into_iter().flatten()) {
        assert!(found == expected, "{found:?} is not {expected:?}")
    }
}
