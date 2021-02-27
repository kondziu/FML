use anyhow::*;
use anyhow::Context;

#[macro_export]
macro_rules! bail_if {
    ($condition:expr, $format:expr, $($arguments:expr),*) => {
        if $condition { anyhow::bail!($format$(, $arguments)*) }
    }
}

#[macro_export]
macro_rules! veccat {
    ($a:expr, $b:expr) => { $a.into_iter().chain($b.into_iter()).collect() };
    ($a:expr, $b:expr, $c:expr) => { $a.into_iter().chain($b.into_iter()).chain($c.into_iter()).collect() };
}

pub trait Pairable<T, I> where T: Copy + Default {
    fn pairs(self) -> PairIterator<T, I>;
}

impl<T, I> Pairable<T, I> for I where I: Iterator<Item=T>, T: Copy + Default {
    fn pairs(self) -> PairIterator<T, I> {
        PairIterator { previous: T::default(), iter: self }
    }
}

pub struct PairIterator<T, I> {
    previous: T,
    iter: I,
}

impl<T, I> Iterator for PairIterator<T, I> where I: Iterator<Item=T>, T: Copy {
    type Item = (T, T);
    fn next(&mut self) -> Option<Self::Item> {
        let next = self.iter.next().map(|current| (self.previous, current));
        if let Some((_, current)) = &next {
            self.previous = *current;
        }
        next
    }
}