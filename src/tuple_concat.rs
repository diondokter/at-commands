#![allow(unused_attributes)]
#[rustfmt::skip]

pub trait TupleConcat<C> {
    type Out;
    fn tup_cat(self, c: C) -> Self::Out;
}

impl<C> TupleConcat<C> for () {
    type Out = (C,);
    fn tup_cat(self, c: C) -> Self::Out {
        (c,)
    }
}

impl<T0, C> TupleConcat<C> for (T0,) {
    type Out = (T0, C);
    fn tup_cat(self, c: C) -> Self::Out {
        (self.0, c)
    }
}

impl<T0, T1, C> TupleConcat<C> for (T0, T1) {
    type Out = (T0, T1, C);
    fn tup_cat(self, c: C) -> Self::Out {
        (self.0, self.1, c)
    }
}

impl<T0, T1, T2, C> TupleConcat<C> for (T0, T1, T2) {
    type Out = (T0, T1, T2, C);
    fn tup_cat(self, c: C) -> Self::Out {
        (self.0, self.1, self.2, c)
    }
}

impl<T0, T1, T2, T3, C> TupleConcat<C> for (T0, T1, T2, T3) {
    type Out = (T0, T1, T2, T3, C);
    fn tup_cat(self, c: C) -> Self::Out {
        (self.0, self.1, self.2, self.3, c)
    }
}

impl<T0, T1, T2, T3, T4, C> TupleConcat<C> for (T0, T1, T2, T3, T4) {
    type Out = (T0, T1, T2, T3, T4, C);
    fn tup_cat(self, c: C) -> Self::Out {
        (self.0, self.1, self.2, self.3, self.4, c)
    }
}

impl<T0, T1, T2, T3, T4, T5, C> TupleConcat<C> for (T0, T1, T2, T3, T4, T5) {
    type Out = (T0, T1, T2, T3, T4, T5, C);
    fn tup_cat(self, c: C) -> Self::Out {
        (self.0, self.1, self.2, self.3, self.4, self.5, c)
    }
}

impl<T0, T1, T2, T3, T4, T5, T6, C> TupleConcat<C> for (T0, T1, T2, T3, T4, T5, T6) {
    type Out = (T0, T1, T2, T3, T4, T5, T6, C);
    fn tup_cat(self, c: C) -> Self::Out {
        (self.0, self.1, self.2, self.3, self.4, self.5, self.6, c)
    }
}

impl<T0, T1, T2, T3, T4, T5, T6, T7, C> TupleConcat<C> for (T0, T1, T2, T3, T4, T5, T6, T7) {
    type Out = (T0, T1, T2, T3, T4, T5, T6, T7, C);
    fn tup_cat(self, c: C) -> Self::Out {
        (self.0, self.1, self.2, self.3, self.4, self.5, self.6, self.7, c)
    }
}

impl<T0, T1, T2, T3, T4, T5, T6, T7, T8, C> TupleConcat<C> for (T0, T1, T2, T3, T4, T5, T6, T7, T8) {
    type Out = (T0, T1, T2, T3, T4, T5, T6, T7, T8, C);
    fn tup_cat(self, c: C) -> Self::Out {
        (self.0, self.1, self.2, self.3, self.4, self.5, self.6, self.7, self.8, c)
    }
}

impl<T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, C> TupleConcat<C> for (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9) {
    type Out = (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, C);
    fn tup_cat(self, c: C) -> Self::Out {
        (self.0, self.1, self.2, self.3, self.4, self.5, self.6, self.7, self.8, self.9, c)
    }
}

impl<T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, C> TupleConcat<C> for (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10) {
    type Out = (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, C);
    fn tup_cat(self, c: C) -> Self::Out {
        (self.0, self.1, self.2, self.3, self.4, self.5, self.6, self.7, self.8, self.9, self.10, c)
    }
}

impl<T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, C> TupleConcat<C> for (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11) {
    type Out = (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, C);
    fn tup_cat(self, c: C) -> Self::Out {
        (self.0, self.1, self.2, self.3, self.4, self.5, self.6, self.7, self.8, self.9, self.10, self.11, c)
    }
}
