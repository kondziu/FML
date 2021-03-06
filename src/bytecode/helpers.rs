use anyhow::*;

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


pub trait MapResult<I> {
    type IntoIter;
    fn into_result(self) -> Result<Self::IntoIter>;
}

impl<I, T> MapResult<I> for I where I: Iterator<Item=Result<T>> + Clone {
    type IntoIter = std::iter::Map<I, fn(Result<T>) -> T>;

    fn into_result(self) -> Result<Self::IntoIter> {
        let error = self.clone()
            .filter(|e| e.is_err())
            .take(1)
            .last();

        if let Some(error) = error { error?; }

        Ok(self.map(|e| e.unwrap()))
    }
}

// pub struct VecMap<K, T>(Vec<(K, T)>) where K: Eq;
// impl<K, T> VecMap<K, T> where K: Eq {
//     pub fn new() -> Self { VecMap(vec![]) }
//     pub fn contains(&self, key: &K) -> bool {
//         self.0.iter()
//             .any(|(resident_key, _)| resident_key == key)
//     }
//     pub fn get(&self, key: &K) -> Option<&T> {
//         self.0.iter()
//             .find(|(resident_key, _)| resident_key == key)
//             .map(|(_, resident_value)| resident_value)
//     }
//     pub fn insert(&mut self, key: K, value: T) -> Option<T> {
//         let position = self.0.iter()
//             .find_position(|(resident_key, _)| resident_key == &key)
//             .map(|(position, (_, _))| position);
//
//         if position.is_none() {
//             self.0.push((key, value));
//             return None
//         }
//
//         let position = position.unwrap();
//         let value = self.0.get(position).map(|(k, v)| v.clone());
//         self.0.insert(position, (key, value))
//
//     }
// }