use crate::prelude::*;
use num::Zero;
use reformation::Reformation;
use num_derive::FromPrimitive;

pub const ACID_COUNT: usize = 20;

#[repr(u8)]
#[derive(
    Debug,
    Copy,
    Clone,
    Eq,
    PartialEq,
    Hash,
    FromPrimitive,
    Ord,
    PartialOrd,
    reformation::Reformation,
)]
pub enum AminoAcid {
    #[reformation("A")]
    Ala,
    #[reformation("R")]
    Arg,
    Asn,
    Asp,
    Cys,
    #[reformation("Q")]
    Gln,
    Glu,
    Gly,
    His,
    #[reformation("I")]
    Ile,
    Leu,
    #[reformation("K")]
    Lys,
    #[reformation("M")]
    Met,
    Phe,
    Pro,
    Ser,
    Thr,
    Trp,
    Tyr,
    #[reformation("V")]
    Val,

    Gap,
    Unknown,
}

impl std::str::FromStr for AminoAcid {
    type Err = reformation::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        AminoAcid::parse(s)
    }
}


impl Serialize for AminoAcid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_char(char::from(self.as_u8()))
    }
}

#[allow(dead_code)]
impl AminoAcid {
    pub fn from_u8(letter: u8) -> Result<AminoAcid> {
        Ok(match letter {
            b'A' => AminoAcid::Ala,
            b'R' => AminoAcid::Arg,
            b'N' => AminoAcid::Asn,
            b'D' => AminoAcid::Asp,
            b'C' => AminoAcid::Cys,
            b'Q' => AminoAcid::Gln,
            b'E' => AminoAcid::Glu,
            b'G' => AminoAcid::Gly,
            b'H' => AminoAcid::His,
            b'I' => AminoAcid::Ile,
            b'L' => AminoAcid::Leu,
            b'K' => AminoAcid::Lys,
            b'M' => AminoAcid::Met,
            b'F' => AminoAcid::Phe,
            b'P' => AminoAcid::Pro,
            b'S' => AminoAcid::Ser,
            b'T' => AminoAcid::Thr,
            b'W' => AminoAcid::Trp,
            b'Y' => AminoAcid::Tyr,
            b'V' => AminoAcid::Val,
            b'X' => AminoAcid::Unknown,
            b'-' => AminoAcid::Gap,
            _ => bail!("Unknown amino amino_acid: {}", char::from(letter)),
        })
    }

    pub fn as_u8(self) -> u8 {
        match self {
            AminoAcid::Ala => b'A',
            AminoAcid::Arg => b'R',
            AminoAcid::Asn => b'N',
            AminoAcid::Asp => b'D',
            AminoAcid::Cys => b'C',
            AminoAcid::Gln => b'Q',
            AminoAcid::Glu => b'E',
            AminoAcid::Gly => b'G',
            AminoAcid::His => b'H',
            AminoAcid::Ile => b'I',
            AminoAcid::Leu => b'L',
            AminoAcid::Lys => b'K',
            AminoAcid::Met => b'M',
            AminoAcid::Phe => b'F',
            AminoAcid::Pro => b'P',
            AminoAcid::Ser => b'S',
            AminoAcid::Thr => b'T',
            AminoAcid::Trp => b'W',
            AminoAcid::Tyr => b'Y',
            AminoAcid::Val => b'V',
            AminoAcid::Gap => b'-',
            AminoAcid::Unknown => b'X',
        }
    }

    pub fn is_amino_acid(self) -> bool {
        !matches!(self, AminoAcid::Gap | AminoAcid::Unknown)
    }

    pub fn as_index(self) -> Option<usize> {
        if self.is_amino_acid() || self == AminoAcid::Gap {
            Some(self as usize)
        } else {
            None
        }
    }

    pub fn from_index(index: usize) -> AminoAcid {
        if index <= ACID_COUNT {
            num::FromPrimitive::from_usize(index).unwrap()
        } else {
            panic!("invalid amino_acid index")
        }
    }

    pub fn iter() -> impl Iterator<Item = AminoAcid> {
        (0..ACID_COUNT).map(AminoAcid::from_index)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct AminoAcidSet(u32);

#[derive(Clone)]
pub struct AminoAcidSetIter(u32);

impl std::fmt::Debug for AminoAcidSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_set().entries(self.iter()).finish()
    }
}

impl std::iter::Iterator for AminoAcidSetIter {
    type Item = AminoAcid;

    fn next(&mut self) -> Option<Self::Item> {
        let next_index = self.0.trailing_zeros();
        if next_index < 20 {
            self.0 ^= 1 << next_index;
            Some(AminoAcid::from_index(next_index as usize))
        } else {
            None
        }
    }
}

#[allow(unused)]
impl AminoAcidSet {
    pub fn empty() -> AminoAcidSet {
        AminoAcidSet(0)
    }

    pub fn byte_hash(self) -> u8 {
        self.0.to_ne_bytes().iter().copied().fold(0, |x, y| x ^ y)
    }

    pub fn len(self) -> u32 {
        self.0.count_ones()
    }

    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub fn insert(&mut self, amino_acid: AminoAcid) {
        let index = amino_acid.as_index().unwrap();
        self.0 |= 1 << index;
    }

    pub fn maybe_insert(&mut self, amino_acid: AminoAcid) {
        if let Some(index) = amino_acid.as_index() {
            self.0 |= 1 << index;
        }
    }

    pub fn remove(&mut self, amino_acid: AminoAcid) {
        let index = amino_acid.as_index().unwrap();
        self.0 &= !(1 << index);
    }

    pub fn contains(self, amino_acid: AminoAcid) -> bool {
        match amino_acid.as_index() {
            Some(index) => self.0 & 1 << index != 0,
            None => false,
        }
    }

    pub fn singleton(amino_acid: AminoAcid) -> AminoAcidSet {
        let mut result = Self::empty();
        result.insert(amino_acid);
        result
    }

    pub fn maybe_singleton(amino_acid: AminoAcid) -> AminoAcidSet {
        let mut result = Self::empty();
        result.maybe_insert(amino_acid);
        result
    }

    pub fn union_with(&mut self, other: AminoAcidSet) {
        self.0 |= other.0;
    }

    pub fn intersection(self, other: AminoAcidSet) -> AminoAcidSet {
        AminoAcidSet(self.0 & other.0)
    }

    pub fn difference(self, other: AminoAcidSet) -> AminoAcidSet {
        AminoAcidSet(self.0 & !other.0)
    }

    pub fn complement(self) -> AminoAcidSet {
        AminoAcidSet(((1 << 21) - 1) ^ self.0)
    }

    pub fn iter(self) -> AminoAcidSetIter {
        AminoAcidSetIter(self.0)
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct AminoAcidMap<V>([V; 21]);

impl<V> AminoAcidMap<V> {
    pub fn from_fn(f: impl Fn(AminoAcid) -> V) -> Self {
        AminoAcidMap(array_ext::Array::from_fn(|index| {
            f(AminoAcid::from_index(index))
        }))
    }

    pub fn iter(&self) -> impl Iterator<Item = (AminoAcid, &V)> {
        self.0
            .iter()
            .enumerate()
            .map(|(index, value)| (AminoAcid::from_index(index), value))
    }

    #[allow(unused)]
    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.0.iter()
    }
}

impl<V> std::ops::Index<AminoAcid> for AminoAcidMap<V> {
    type Output = V;

    fn index(&self, index: AminoAcid) -> &Self::Output {
        &self.0[index.as_index().unwrap()]
    }
}

impl<V> std::ops::IndexMut<AminoAcid> for AminoAcidMap<V> {
    fn index_mut(&mut self, index: AminoAcid) -> &mut Self::Output {
        &mut self.0[index.as_index().unwrap()]
    }
}

#[derive(Clone, Copy)]
pub struct AminoAcidModel {
    pub rate_matrix:
        nalgebra::Matrix<f64, nalgebra::U20, nalgebra::U20, nalgebra::ArrayStorage<f64, 20, 20>>,
    pub initial_probabilities: [FixedLog; 20],
}

pub struct ParameterizedAminoAcidModel {
    pub matrix:
        nalgebra::Matrix<Log, nalgebra::U20, nalgebra::U20, nalgebra::ArrayStorage<Log, 20, 20>>,
    pub parameter: R64,
}

impl ParameterizedAminoAcidModel {
    pub fn likelihood(
        &self,
        transitions: &nalgebra::Matrix<
            i32,
            nalgebra::U20,
            nalgebra::U20,
            nalgebra::ArrayStorage<i32, 20, 20>,
        >,
    ) -> Log {
        /* 
        dbg!(transitions[(AminoAcid::Ile.as_index().unwrap(), AminoAcid::Ile.as_index().unwrap())]);
        dbg!(transitions[(AminoAcid::Val.as_index().unwrap(), AminoAcid::Ile.as_index().unwrap())]);
        dbg!(self.matrix[(AminoAcid::Ile.as_index().unwrap(), AminoAcid::Ile.as_index().unwrap())]);
        dbg!(self.matrix[(AminoAcid::Val.as_index().unwrap(), AminoAcid::Ile.as_index().unwrap())]);*/
        /* 
        for amino_acid in AminoAcid::iter() {
            for rhs in AminoAcid::iter() {
                eprintln!("{:?} => {:?} @ {:?}", amino_acid, rhs, self.matrix[(amino_acid.as_index().unwrap(), rhs.as_index().unwrap())]);
            }
        }*/
        

        self.matrix
            .zip_fold(transitions, Log::one(), |accum, transition, count| {
                if count.is_zero() {
                    accum
                } else {
                    accum * transition.powi(count)
                }
            })
    }
}

impl AminoAcidModel {
    pub fn initial(&self, amino_acid: AminoAcid) -> FixedLog {
        self.initial_probabilities[amino_acid.as_index().unwrap()]
    }

    pub fn parameterize(&self, parameter: R64) -> ParameterizedAminoAcidModel {
        ParameterizedAminoAcidModel {
            matrix: (self.rate_matrix * parameter.raw()).exp().map(Log::from),
            parameter,
        }
    }
}

pub fn read_paml_matrix(read: impl std::io::Read) -> Result<AminoAcidModel> {
    use std::io::BufRead;

    let mut read = std::io::BufReader::new(read);
    let mut rate_matrix = nalgebra::Matrix::<
        f64,
        nalgebra::U20,
        nalgebra::U20,
        nalgebra::ArrayStorage<f64, 20, 20>,
    >::zeros();

    let mut line = String::new();
    for row in 1..20 {
        line.clear();
        read.read_line(&mut line)?;

        let parts = line.split_whitespace().collect_vec();
        if parts.len() != row {
            return Err(eyre!("Expected {} numbers on line {}", row, row));
        }

        for (column, part) in parts.into_iter().enumerate() {
            let parsed: f64 = part.parse()?;
            rate_matrix[(row, column)] = parsed;
            rate_matrix[(column, column)] -= parsed;
            rate_matrix[(row, row)] -= parsed;
            rate_matrix[(column, row)] = parsed;
        }
    }

    line.clear();
    read.read_line(&mut line)?;
    if !line.trim().is_empty() {
        return Err(eyre!("Expected blank line, got: {}", line));
    }

    line.clear();
    read.read_line(&mut line)?;
    let parts = line
        .trim_end_matches(";\n")
        .split_whitespace()
        .collect_vec();
    if parts.len() != 20 {
        return Err(eyre!("Expected 20 items"));
    }

    let mut initial_probabilities = [FixedLog::one(); 20];

    for (index, value) in parts.into_iter().enumerate() {
        let parsed: f64 = value.parse()?;
        if parsed.is_zero() {
            initial_probabilities[index] = FixedLog::smallest();
        } else {
            initial_probabilities[index] = FixedLog::from(parsed);
        }
    }
    dbg!(&initial_probabilities);

    Ok(AminoAcidModel {
        rate_matrix,
        initial_probabilities,
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_as_index() {
        assert_eq!(AminoAcid::Ala.as_index(), Some(0));
        assert_eq!(AminoAcid::Val.as_index(), Some(19));
    }

    #[test]
    fn test_index() {
        assert_eq!(AminoAcid::from_index(0), AminoAcid::Ala);
        assert_eq!(AminoAcid::from_index(19), AminoAcid::Val);
    }

    #[test]
    #[should_panic(expected = "invalid amino_acid index")]
    fn test_index_invalid() {
        AminoAcid::from_index(254);
    }

    #[test]
    #[should_panic(expected = "invalid amino_acid index")]
    fn test_index_invalid2() {
        AminoAcid::from_index(655);
    }

    #[test]
    fn test_from_u8() -> Result<()> {
        assert_eq!(AminoAcid::from_u8(b'A')?, AminoAcid::Ala);
        assert_eq!(AminoAcid::from_u8(b'W')?, AminoAcid::Trp);

        Ok(())
    }

    #[test]
    fn test_to_u8() -> Result<()> {
        assert_eq!(b'A', AminoAcid::Ala.as_u8());
        assert_eq!(b'W', AminoAcid::Trp.as_u8());

        Ok(())
    }

    quickcheck! {
        fn converts_back_and_forth(letter: u8) -> bool {
            if let Ok(amino_acid) = AminoAcid::from_u8(letter) {
                letter == amino_acid.as_u8()
            } else {
                true
            }
        }
    }

    #[test]
    fn test_remove_bit() {
        let mut set = AminoAcidSet::empty();

        set.insert(AminoAcid::Ala);
        set.insert(AminoAcid::Glu);

        assert_eq!(
            set.iter().collect_vec(),
            vec![AminoAcid::Ala, AminoAcid::Glu]
        );

        set.remove(AminoAcid::Glu);

        assert_eq!(set.iter().collect_vec(), vec![AminoAcid::Ala,]);
    }
}
