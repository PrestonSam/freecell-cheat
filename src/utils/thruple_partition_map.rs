pub enum TernaryVal<L, M, R> {
    Left(L),
    Middle(M),
    Right(R),
}

pub trait ThruplePartitionMap: Iterator {
    fn thruple_partition_map<A, B, C, F, L, M, R>(self, mut chooser: F) -> (A, B, C)
    where
        Self: Sized,
        F: FnMut(Self::Item) -> TernaryVal<L, M, R>,
        A: Default + Extend<L>,
        B: Default + Extend<M>,
        C: Default + Extend<R>,
    {
        let mut left = A::default();
        let mut middle = B::default();
        let mut right = C::default();

        for val in self {
            match chooser(val) {
                TernaryVal::Left(v) => left.extend(Some(v)),
                TernaryVal::Middle(v) => middle.extend(Some(v)),
                TernaryVal::Right(v) => right.extend(Some(v)),
            }
        }

        (left, middle, right)
    }
}

impl<T> ThruplePartitionMap for T where T: Iterator {}
