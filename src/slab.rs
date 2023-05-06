use crate::prelude::*;
use serde::ser::SerializeMap;
use serde::ser::SerializeSeq;

pub type IndexType = u16;
pub trait SlabHandle: Copy + Clone + std::fmt::Debug + PartialEq {
    fn from_raw(index: IndexType) -> Self;
    fn into_raw(self) -> IndexType;
}

#[macro_export]
macro_rules! define_slab_handle {
    ($name:ident) => {
        #[derive(
            Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, reformation::Reformation,
        )]
        #[reformation("N{}")]
        pub struct $name(pub crate::slab::IndexType);
        impl crate::slab::SlabHandle for $name {
            fn from_raw(index: crate::slab::IndexType) -> Self {
                $name(index)
            }
            fn into_raw(self) -> crate::slab::IndexType {
                self.0
            }
        }

        impl serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                String::serialize(&format!("{}", self.0), serializer)
            }
        }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<$name, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let base: String = serde::Deserialize::deserialize(deserializer)?;
                let first: crate::slab::IndexType = base.parse().unwrap();
                Ok($name(first))
            }
        }
    };
}

#[derive(Clone)]
enum SlabItem<DataType> {
    Empty { next_free: Option<IndexType> },
    Full { data: DataType },
}

#[derive(Clone)]
pub struct Slab<HandleType, DataType> {
    items: Vec<SlabItem<DataType>>,
    free: Option<IndexType>,
    phantom: std::marker::PhantomData<HandleType>,
}

impl<HandleType: SlabHandle, DataType: PartialEq> std::cmp::PartialEq
    for Slab<HandleType, DataType>
{
    fn eq(&self, other: &Self) -> bool {
        self.iter().eq(other.iter())
    }
}

impl<HandleType: SlabHandle, DataType: std::fmt::Debug> std::fmt::Debug
    for Slab<HandleType, DataType>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

impl<HandleType: SlabHandle, DataType: std::default::Default> Default
    for Slab<HandleType, DataType>
{
    fn default() -> Self {
        Self::new()
    }
}

impl<HandleType: SlabHandle, DataType: std::default::Default> SlabMap<HandleType, DataType> {
    pub fn get_or_insert_mut(&mut self, handle: HandleType) -> &mut DataType {
        if self.get(handle).is_none() {
            self.insert(handle, DataType::default());
        }
        self.get_mut(handle).unwrap()
    }
}

impl<HandleType: SlabHandle, DataType: Send + Sync> Slab<HandleType, DataType> {
    pub fn par_values(&self) -> impl ParallelIterator<Item = &DataType> + '_ {
        self.items.par_iter().filter_map(|item| match item {
            SlabItem::Empty { .. } => None,
            SlabItem::Full { data, .. } => Some(data),
        })
    }
}

impl<HandleType: SlabHandle, DataType> Slab<HandleType, DataType> {
    pub fn new() -> Self {
        Slab {
            items: Vec::new(),
            free: None,
            phantom: std::marker::PhantomData::default(),
        }
    }

    pub fn from_map(data: FnvHashMap<HandleType, DataType>) -> Self {
        if let Some(last_index) = data.keys().map(|key| key.into_raw()).max() {
            let mut items = (0..last_index + 1)
                .map(|_| SlabItem::Empty { next_free: None })
                .collect_vec();

            for (key, data) in data {
                let index = key.into_raw();
                assert!(matches!(items[index as usize], SlabItem::Empty { .. }));
                items[index as usize] = SlabItem::Full { data }
            }

            let mut free = None;
            for (index, item) in items.iter_mut().enumerate() {
                if let SlabItem::Empty { next_free, .. } = item {
                    *next_free = free;
                    free = Some(index as u16)
                }
            }

            Slab {
                items,
                free,

                phantom: std::marker::PhantomData::default(),
            }
        } else {
            // if the map is empty then return an empty slab
            Self::new()
        }
    }

    pub fn capacity(&self) -> usize {
        self.items.len()
    }

    pub fn recover(&self, index: IndexType) -> Option<HandleType> {
        self.items.get(index as usize).and_then(|entry| {
            if let SlabItem::Full { .. } = entry {
                Some(HandleType::from_raw(index))
            } else {
                None
            }
        })
    }

    pub fn len(&self) -> usize {
        self.items
            .iter()
            .filter(|x| matches!(x, SlabItem::Full { .. }))
            .count()
    }

    pub fn is_empty(&self) -> bool {
        self.items
            .iter()
            .any(|x| matches!(x, SlabItem::Full { .. }))
    }

    pub fn insert(&mut self, data: DataType) -> HandleType {
        if let Some(free) = self.free {
            if let SlabItem::Empty { next_free } = self.items[usize::from(free)] {
                self.free = next_free;
                self.items[usize::from(free)] = SlabItem::Full { data };
                HandleType::from_raw(free)
            } else {
                panic!("invariant failed: free was not free");
            }
        } else {
            let handle = HandleType::from_raw(
                IndexType::try_from(self.items.len()).expect("ran out of indexes"),
            );

            self.items.push(SlabItem::Full { data });

            handle
        }
    }

    pub fn get(&self, handle: HandleType) -> Option<&DataType> {
        let handle_index = handle.into_raw();
        if let Some(SlabItem::Full { data }) = self.items.get(usize::from(handle_index)) {
            return Some(data);
        }
        None
    }

    pub fn contains(&self, handle: HandleType) -> bool {
        let handle_index = handle.into_raw();
        if let Some(SlabItem::Full { .. }) = self.items.get(usize::from(handle_index)) {
            return true;
        }
        false
    }

    pub fn get_mut(&mut self, handle: HandleType) -> Option<&mut DataType> {
        let handle_index = handle.into_raw();
        if let Some(SlabItem::Full { data }) = self.items.get_mut(usize::from(handle_index)) {
            return Some(data);
        }
        None
    }

    pub fn remove(&mut self, handle: HandleType) -> Option<DataType> {
        let handle_index = handle.into_raw();
        if let Some(item) = self.items.get_mut(usize::from(handle_index)) {
            if if let SlabItem::Full { .. } = item {
                true
            } else {
                false
            } {
                let mut tmp = SlabItem::Empty {
                    next_free: self.free,
                };
                std::mem::swap(&mut tmp, item);

                if let SlabItem::Full { data, .. } = tmp {
                    return Some(data);
                } else {
                    unreachable!();
                }
            }
        }
        None
    }

    pub fn iter(&self) -> impl Iterator<Item = (HandleType, &DataType)> + Clone {
        self.items
            .iter()
            .enumerate()
            .filter_map(|(index, item)| match item {
                SlabItem::Empty { .. } => None,
                SlabItem::Full { data } => Some((
                    HandleType::from_raw(IndexType::try_from(index).expect("index too large")),
                    data,
                )),
            })
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (HandleType, &mut DataType)> {
        self.items
            .iter_mut()
            .enumerate()
            .filter_map(|(index, item)| match item {
                SlabItem::Empty { .. } => None,
                SlabItem::Full { data } => Some((
                    HandleType::from_raw(IndexType::try_from(index).expect("index too large")),
                    data,
                )),
            })
    }

    pub fn values(&self) -> impl Iterator<Item = &DataType> + '_ {
        self.items.iter().filter_map(|item| match item {
            SlabItem::Empty { .. } => None,
            SlabItem::Full { data, .. } => Some(data),
        })
    }
    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut DataType> + '_ {
        self.items.iter_mut().filter_map(|item| match item {
            SlabItem::Empty { .. } => None,
            SlabItem::Full { data, .. } => Some(data),
        })
    }

    pub fn ids(&self) -> impl Iterator<Item = HandleType> + '_ + Clone {
        self.items
            .iter()
            .enumerate()
            .filter_map(|(index, item)| match item {
                SlabItem::Empty { .. } => None,
                SlabItem::Full { .. } => Some(HandleType::from_raw(
                    IndexType::try_from(index).expect("index too large"),
                )),
            })
    }

    pub fn first_id(&self) -> Option<HandleType> {
        self.items.iter().enumerate().find_map(|(index, item)| {
            if let SlabItem::Full { .. } = item {
                return Some(HandleType::from_raw(
                    IndexType::try_from(index).expect("index too large"),
                ));
            } else {
                None
            }
        })
    }

    pub fn next_id(&self, id: HandleType) -> Option<HandleType> {
        self.items
            .iter()
            .enumerate()
            .skip(id.into_raw() as usize + 1)
            .find_map(|(index, item)| {
                if let SlabItem::Full { .. } = item {
                    return Some(HandleType::from_raw(
                        IndexType::try_from(index).expect("index too large"),
                    ));
                } else {
                    None
                }
            })
    }
}

impl<HandleType: SlabHandle, DataType> std::ops::Index<HandleType> for Slab<HandleType, DataType> {
    type Output = DataType;
    fn index(&self, index: HandleType) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl<HandleType: SlabHandle, DataType> std::ops::IndexMut<HandleType>
    for Slab<HandleType, DataType>
{
    fn index_mut(&mut self, index: HandleType) -> &mut Self::Output {
        self.get_mut(index).unwrap()
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct SlabSet<HandleType> {
    items: Vec<bool>,
    phantom: std::marker::PhantomData<HandleType>,
}

impl<HandleType: SlabHandle + Serialize> Serialize for SlabSet<HandleType> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(None)?;
        for item in self.iter() {
            seq.serialize_element(&item)?;
        }
        seq.end()
    }
}

impl<HandleType: SlabHandle> std::default::Default for SlabSet<HandleType> {
    fn default() -> Self {
        Self::new()
    }
}

impl<HandleType: SlabHandle> SlabSet<HandleType> {
    pub fn new() -> Self {
        SlabSet {
            items: Vec::new(),
            phantom: std::marker::PhantomData::default(),
        }
    }

    pub fn new_with_capacity_of<X>(slab: &Slab<HandleType, X>) -> Self {
        SlabSet {
            items: Vec::with_capacity(slab.items.len()),
            phantom: std::marker::PhantomData::default(),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = HandleType> + '_ {
        self.items.iter().enumerate().filter_map(|(index, &item)| {
            if item {
                Some(HandleType::from_raw(IndexType::try_from(index).unwrap()))
            } else {
                None
            }
        })
    }

    pub fn insert(&mut self, handle: HandleType) -> bool {
        let index = handle.into_raw();
        if self.items.len() <= usize::from(index) {
            self.items.resize(usize::from(index) + 1, false);
        }
        let old = self.items[usize::from(index)];
        self.items[usize::from(index)] = true;
        !old
    }

    pub fn len(&self) -> usize {
        self.items.iter().filter(|&&x| x).count()
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn remove(&mut self, handle: HandleType) {
        let index = handle.into_raw();
        if let Some(target) = self.items.get_mut(usize::from(index)) {
            *target = false;
        }
    }

    pub fn toggle(&mut self, handle: HandleType) {
        if self.contains(handle) {
            self.remove(handle);
        } else {
            self.insert(handle);
        }
    }

    pub fn contains(&self, handle: HandleType) -> bool {
        let handle_index = handle.into_raw();
        self.items
            .get(usize::from(handle_index))
            .copied()
            .unwrap_or(false)
    }
}

impl<T: SlabHandle> std::fmt::Debug for SlabSet<T> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        fmt.debug_set().entries(self.iter()).finish()
    }
}

impl<T: SlabHandle> std::iter::FromIterator<T> for SlabSet<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut result = SlabSet::new();
        for item in iter {
            result.insert(item);
        }
        result
    }
}

#[derive(Clone)]
pub struct SlabMap<HandleType, DataType> {
    items: Vec<Option<DataType>>,
    phantom: std::marker::PhantomData<HandleType>,
}

impl<HandleType: SlabHandle + Serialize, DataType: Serialize> Serialize
    for SlabMap<HandleType, DataType>
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_map(None)?;
        for (k, v) in self.iter() {
            seq.serialize_entry(&k, v)?;
        }
        seq.end()
    }
}

impl<HandleType: SlabHandle, DataType> std::iter::FromIterator<(HandleType, DataType)>
    for SlabMap<HandleType, DataType>
{
    fn from_iter<T: IntoIterator<Item = (HandleType, DataType)>>(iter: T) -> Self {
        let mut map = SlabMap::new();
        for (key, value) in iter {
            map.insert(key, value);
        }
        map
    }
}

impl<HandleType: SlabHandle, DataType: std::fmt::Debug> std::fmt::Debug
    for SlabMap<HandleType, DataType>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

impl<HandleType: SlabHandle, DataType> std::default::Default for SlabMap<HandleType, DataType> {
    fn default() -> Self {
        Self::new()
    }
}

impl<HandleType: SlabHandle, DataType> SlabMap<HandleType, DataType> {
    pub fn new() -> Self {
        SlabMap {
            items: Vec::new(),
            phantom: std::marker::PhantomData::default(),
        }
    }
    pub fn get_or_insert_with_mut(
        &mut self,
        handle: HandleType,
        f: impl FnOnce() -> DataType,
    ) -> &mut DataType {
        if self.get(handle).is_none() {
            self.insert(handle, f());
        }
        self.get_mut(handle).unwrap()
    }

    pub fn from_map(data: FnvHashMap<HandleType, DataType>) -> Self {
        let mut result = Self::new();
        for (key, value) in data {
            result.insert(key, value);
        }
        result
    }

    pub fn with_capacity(capacity: usize) -> Self {
        SlabMap {
            items: Vec::with_capacity(capacity),
            phantom: std::marker::PhantomData::default(),
        }
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }

    pub fn insert(&mut self, handle: HandleType, data: DataType) {
        let index = handle.into_raw();
        if self.items.len() <= usize::from(index) {
            self.items.resize_with(usize::from(index) + 1, || None);
        }
        self.items[usize::from(index)] = Some(data);
    }

    #[inline(always)]
    pub fn get(&self, handle: HandleType) -> Option<&DataType> {
        let handle_index = handle.into_raw();
        if let Some(Some(data)) = self.items.get(usize::from(handle_index)) {
            return Some(data);
        }
        None
    }

    pub fn get_mut(&mut self, handle: HandleType) -> Option<&mut DataType> {
        let handle_index = handle.into_raw();
        if let Some(Some(data)) = self.items.get_mut(usize::from(handle_index)) {
            return Some(data);
        }
        None
    }

    pub fn remove(&mut self, handle: HandleType) -> Option<DataType> {
        let handle_index = handle.into_raw();
        if let Some(shelf) = self.items.get_mut(usize::from(handle_index)) {
            return shelf.take();
        }
        None
    }

    pub fn values(&self) -> impl Iterator<Item = &DataType> + Clone {
        self.items.iter().filter_map(|item| item.as_ref())
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut DataType> {
        self.items.iter_mut().filter_map(|item| item.as_mut())
    }

    pub fn into_values(self) -> impl Iterator<Item = DataType> {
        self.items.into_iter().filter_map(|item| item)
    }

    pub fn iter(&self) -> impl Iterator<Item = (HandleType, &DataType)> + '_ + Clone {
        self.items.iter().enumerate().filter_map(|(index, item)| {
            item.as_ref().map(|item| {
                (
                    HandleType::from_raw(IndexType::try_from(index).unwrap()),
                    item,
                )
            })
        })
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (HandleType, &mut DataType)> + '_ {
        self.items
            .iter_mut()
            .enumerate()
            .filter_map(|(index, item)| {
                item.as_mut().map(|item| {
                    (
                        HandleType::from_raw(IndexType::try_from(index).unwrap()),
                        item,
                    )
                })
            })
    }
}

pub struct SlabMapIntoIterator<HandleType: SlabHandle, DataType>(
    std::iter::Enumerate<std::vec::IntoIter<Option<DataType>>>,
    std::marker::PhantomData<HandleType>,
);

impl<HandleType: SlabHandle, DataType> std::iter::Iterator
    for SlabMapIntoIterator<HandleType, DataType>
{
    type Item = (HandleType, DataType);

    fn next(&mut self) -> Option<Self::Item> {
        for (index, next) in &mut self.0 {
            if let Some(data) = next {
                return Some((
                    HandleType::from_raw(IndexType::try_from(index).unwrap()),
                    data,
                ));
            }
        }
        None
    }
}

impl<HandleType: SlabHandle, DataType> IntoIterator for SlabMap<HandleType, DataType> {
    type Item = (HandleType, DataType);

    type IntoIter = SlabMapIntoIterator<HandleType, DataType>;

    fn into_iter(self) -> Self::IntoIter {
        SlabMapIntoIterator(
            self.items.into_iter().enumerate(),
            std::marker::PhantomData::default(),
        )
    }
}

impl<HandleType: SlabHandle, DataType> std::ops::Index<HandleType>
    for SlabMap<HandleType, DataType>
{
    type Output = DataType;
    fn index(&self, index: HandleType) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl<HandleType: SlabHandle, DataType> std::ops::IndexMut<HandleType>
    for SlabMap<HandleType, DataType>
{
    fn index_mut(&mut self, index: HandleType) -> &mut Self::Output {
        self.get_mut(index).unwrap()
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[derive(Copy, Clone, Debug, PartialEq)]
    struct FoobarHandle(IndexType);
    impl SlabHandle for FoobarHandle {
        fn from_raw(index: IndexType) -> Self {
            FoobarHandle(index)
        }
        fn into_raw(self) -> IndexType {
            self.0
        }
    }

    #[test]
    fn test_insert_get() {
        let mut slab: Slab<FoobarHandle, &'static str> = Slab::new();
        let foobar: FoobarHandle = slab.insert("cow");
        assert_eq!(slab.get(foobar), Some(&"cow"));
    }

    #[test]
    fn test_insert_delete_get() {
        let mut slab: Slab<FoobarHandle, &'static str> = Slab::new();
        let foobar: FoobarHandle = slab.insert("cow");
        slab.remove(foobar);
        assert_eq!(slab.get(foobar), None);
    }

    #[test]
    fn test_insert_mixed() {
        let mut slab: Slab<FoobarHandle, &'static str> = Slab::new();
        let foobar: FoobarHandle = slab.insert("cow");
        let goat: FoobarHandle = slab.insert("goat");
        slab.remove(goat);
        let horse: FoobarHandle = slab.insert("horse");
        assert_eq!(slab.get(foobar), Some(&"cow"));
        assert_eq!(slab.get(goat), None);
        assert_eq!(slab.get(horse), Some(&"horse"));
    }

    #[test]
    fn test_set() {
        let mut slab: Slab<FoobarHandle, &'static str> = Slab::new();
        let foobar: FoobarHandle = slab.insert("cow");
        let goat: FoobarHandle = slab.insert("goat");
        let horse: FoobarHandle = slab.insert("horse");
        let mut set = SlabSet::new();
        set.insert(foobar);
        set.insert(goat);
        assert!(set.contains(foobar));
        assert!(set.contains(goat));
        assert!(!set.contains(horse));
    }
}
