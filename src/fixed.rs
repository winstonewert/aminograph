use crate::prelude::*;

pub type IndexType = u16;

pub trait FixedIndex: Copy + Clone + std::fmt::Debug {
    fn from_raw(index: IndexType) -> Self;
    fn into_raw(self) -> IndexType;
}

#[derive(Clone, PartialOrd, Ord, PartialEq, Eq)]
pub struct FixedVec<T, D> {
    items: Vec<D>,
    phantom: std::marker::PhantomData<T>,
}

impl<T, D: Serialize> serde::Serialize for FixedVec<T, D> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        Vec::<D>::serialize(&self.items, serializer)
    }
}

impl<'de, T, V: Deserialize<'de>> serde::Deserialize<'de> for FixedVec<T, V> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(FixedVec {
            items: serde::Deserialize::deserialize(deserializer)?,
            phantom: std::marker::PhantomData::default(),
        })
    }
}

impl<T, D: std::fmt::Debug> std::fmt::Debug for FixedVec<T, D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.items, f)
    }
}

#[allow(unused)]
impl<T: FixedIndex + Send, D: Send + Sync> FixedVec<T, D> {
    pub fn par_iter(&self) -> impl ParallelIterator<Item = (T, &D)> {
        self.items
            .par_iter()
            .enumerate()
            .map(|(index, data)| (T::from_raw(IndexType::try_from(index).unwrap()), data))
    }

    pub fn par_make_vec<R: Send>(&self, f: impl Fn(T, &D) -> R + Sync + Send) -> FixedVec<T, R> {
        FixedVec {
            items: self.par_iter().map(|(x, y)| f(x, y)).collect(),
            phantom: std::marker::PhantomData::default(),
        }
    }
}

#[allow(dead_code)]
impl<T: FixedIndex, D> FixedVec<T, D> {
    pub fn from_raw(items: Vec<D>) -> Self {
        FixedVec {
            items,
            phantom: std::default::Default::default(),
        }
    }

    pub fn ids(&self) -> impl Iterator<Item = T> + Clone {
        (0..self.items.len()).map(|index| T::from_raw(IndexType::try_from(index).unwrap()))
    }

    pub fn make_vec<R>(&self, mut f: impl FnMut(T, &D) -> R) -> FixedVec<T, R> {
        FixedVec {
            items: self.iter().map(|(x, y)| f(x, y)).collect(),
            phantom: std::marker::PhantomData::default(),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (T, &D)> + Clone {
        self.items
            .iter()
            .enumerate()
            .map(|(index, data)| (T::from_raw(IndexType::try_from(index).unwrap()), data))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (T, &mut D)> {
        self.items
            .iter_mut()
            .enumerate()
            .map(|(index, data)| (T::from_raw(IndexType::try_from(index).unwrap()), data))
    }

    pub fn values(&self) -> impl Iterator<Item = &D> {
        self.items.iter()
    }

    pub fn into_values(self) -> impl Iterator<Item = D> {
        self.items.into_iter()
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut D> {
        self.items.iter_mut()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn recover(&self, index: IndexType) -> Option<T> {
        if (index as usize) < self.items.len() {
            Some(T::from_raw(index))
        } else {
            None
        }
    }
}

impl<T: FixedIndex, D> std::ops::Index<T> for FixedVec<T, D> {
    type Output = D;
    fn index(&self, index: T) -> &Self::Output {
        &self.items[usize::from(index.into_raw())]
    }
}

impl<T: FixedIndex, D> std::ops::IndexMut<T> for FixedVec<T, D> {
    fn index_mut(&mut self, index: T) -> &mut Self::Output {
        &mut self.items[usize::from(index.into_raw())]
    }
}
pub struct FixedIndexes<T> {
    size: IndexType,
    phantom: std::marker::PhantomData<T>,
}

#[allow(dead_code)]
impl<T: FixedIndex> FixedIndexes<T> {
    pub fn new(size: usize) -> Self {
        FixedIndexes {
            size: IndexType::try_from(size).expect("insufficient index space"),
            phantom: std::marker::PhantomData::default(),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = T> + Clone {
        (0..self.size).map(T::from_raw)
    }

    pub fn make_vec<D>(&self, f: impl FnMut(T) -> D) -> FixedVec<T, D> {
        FixedVec {
            items: self.iter().map(f).collect(),
            phantom: std::marker::PhantomData::default(),
        }
    }
}

#[macro_export]
macro_rules! define_index {
    ($name:ident) => {
        #[derive(
            Debug,
            Copy,
            Clone,
            Ord,
            PartialOrd,
            Eq,
            PartialEq,
            Hash,
            Serialize,
            Deserialize,
            reformation::Reformation,
        )]
        #[reformation("{}")]
        pub struct $name(pub crate::fixed::IndexType);

        impl FixedIndex for $name {
            fn from_raw(index: crate::fixed::IndexType) -> Self {
                $name(index)
            }
            fn into_raw(self) -> crate::fixed::IndexType {
                self.0
            }
        }
    };
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Debug, Copy, Clone, PartialEq)]
    struct MyIndex(IndexType);
    impl FixedIndex for MyIndex {
        fn from_raw(index: IndexType) -> Self {
            MyIndex(index)
        }
        fn into_raw(self) -> IndexType {
            self.0
        }
    }

    #[test]
    fn test_new_iter() {
        let indexes: FixedIndexes<MyIndex> = FixedIndexes::new(5);

        assert_eq!(
            vec![MyIndex(0), MyIndex(1), MyIndex(2), MyIndex(3), MyIndex(4)],
            indexes.iter().collect_vec()
        );
    }

    #[test]
    fn test_make_vec() {
        let indexes: FixedIndexes<MyIndex> = FixedIndexes::new(5);
        let vec = indexes.make_vec(|index| format!("{:?}", index));

        assert_eq!(vec[MyIndex(2)], "MyIndex(2)");
    }
}
