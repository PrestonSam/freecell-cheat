pub enum Ternary<T> {
    None,
    One(T),
    Two(T, T),
}

impl<T> Ternary<T> {
    pub fn and(self, other: Self) -> Self {
        let mut all_vals = self.into_iter()
            .chain(other.into_iter());

        match (all_vals.next(), all_vals.next()) {
            (None, _) => Ternary::None,
            (Some(fst), None) => Ternary::One(fst),
            (Some(fst), Some(snd)) => Ternary::Two(fst, snd),
        }
    }

    pub fn and_then(self, other: fn() -> Self) -> Self {
        if matches!(self, Ternary::Two(_, _)) {
            self.and(other())
        } else {
            self
        }
    }
}

pub struct IntoIter<T> {
    fst: Option<T>,
    snd: Option<T>, 
    pointer: usize,
}

impl<T> IntoIter<T> {
    fn new_none() -> Self {
        IntoIter {
            fst: None,
            snd: None,
            pointer: 0,
        }
    }

    fn new_one(fst: T) -> Self {
        IntoIter {
            fst: Some(fst),
            snd: None,
            pointer: 0,
        }
    }

    fn new_two(fst: T, snd: T) -> Self {
        IntoIter {
            fst: Some(fst),
            snd: Some(snd),
            pointer: 0,
        }
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            IntoIter { pointer: 0, fst: Some(_), .. } => { self.pointer = 1; self.fst.take() },
            IntoIter { pointer: 1, snd: Some(_), .. } => { self.pointer = 2; self.snd.take() },
            _ => None
        }
    }
}

impl<T> IntoIterator for Ternary<T> {
    type Item = T;

    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Ternary::None => IntoIter::new_none(),
            Ternary::One(fst) => IntoIter::new_one(fst),
            Ternary::Two(fst, snd) => IntoIter::new_two(fst, snd),
        }
    }
}

impl<T> std::fmt::Debug for Ternary<T>
where T: std::fmt::Debug
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "Ternary::None"),
            Self::One(arg0) => f.write_fmt(format_args!("Ternary::One({:?})", arg0)),
            Self::Two(arg0, arg1) => f.debug_tuple("Ternary::Two").field(arg0).field(arg1).finish(),
        }
    }
}
