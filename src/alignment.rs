use crate::amino_acids::AminoAcid;
use crate::amino_acids::AminoAcidMap;
use crate::amino_acids::AminoAcidModel;
use crate::fixed::{FixedIndex, FixedVec};
use crate::graph::Stats;
use crate::prelude::*;
use seq_io::fasta::Record;

define_index!(SequenceId);
define_index!(RawPositionIndex);
define_index!(PositionIndex);
define_index!(GroupIndex);

#[derive(Debug, Clone)]
pub struct Alignment {
    pub sequence_ids: FixedVec<SequenceId, String>,
    pub raw_positions: FixedVec<RawPositionIndex, RawPosition>,
    pub positions: FixedVec<PositionIndex, PositionData>,
    pub root_stats: Stats,
    pub other_stats: Stats,
    pub sequence_stats: FixedVec<SequenceId, Stats>,
}

#[derive(Debug, Clone)]
pub enum RawPosition {
    Standard(FixedVec<SequenceId, AminoAcid>),
    Simple(AminoAcid, FixedVec<SequenceId, AminoAcid>),
}

impl RawPosition {
    fn new(sequences: FixedVec<SequenceId, AminoAcid>, _index: usize) -> Self {
        let distinct_amino_acids = sequences.values().sorted().dedup().count();
        let repeated_amino_acid = sequences
            .values()
            .sorted()
            .group_by(|x| **x)
            .into_iter()
            .filter_map(|(x, c)| if c.count() > 1 { Some(x) } else { None })
            .exactly_one()
            .ok();

        if let Some(repeated) = repeated_amino_acid {
            if distinct_amino_acids <= 2
            /*  && (!sequences.values().any(|&x| x == AminoAcid::Gap) || AminoAcid::Gap == repeated)*/
            {
                return RawPosition::Simple(repeated, sequences);
            }
        }

        Self::Standard(sequences)
    }

    pub fn is_standard(&self) -> bool {
        matches!(self, RawPosition::Standard(_))
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PositionData {
    pub sequences: FixedVec<SequenceId, AminoAcid>,
    pub candidates: Vec<AminoAcid>,
    pub counts: AminoAcidMap<i32>,
}

impl PositionData {
    fn new(sequences: FixedVec<SequenceId, AminoAcid>) -> PositionData {
        let mut counts = AminoAcidMap::from_fn(|_| 0);
        for &amino_acid in sequences.values() {
            if amino_acid.is_amino_acid() || amino_acid == AminoAcid::Gap {
                counts[amino_acid] += 1;
            }
        }

        let candidates = counts
            .iter()
            .filter(|x| *x.1 > 1)
            .map(|x| x.0)
            .collect_vec();

        PositionData {
            sequences,
            candidates,
            counts,
        }
    }
}

pub fn read_alignment(read: impl std::io::Read, model: &AminoAcidModel) -> Result<Alignment> {
    let mut reader = seq_io::fasta::Reader::new(read);

    let records = itertools::process_results(reader.records(), |records| {
        records.into_iter().collect_vec()
    })?;

    let sequence_ids = FixedVec::<SequenceId, _>::from_raw(
        records
            .iter()
            .map(|record| record.id().map(|x| x.to_owned()))
            .collect::<Result<Vec<_>, _>>()?,
    );

    let sequences = FixedVec::from_raw(
        records
            .iter()
            .map(|seq| {
                seq.seq
                    .iter()
                    .copied()
                    .map(AminoAcid::from_u8)
                    .collect::<Result<Vec<AminoAcid>>>()
            })
            .collect::<Result<Vec<Vec<AminoAcid>>>>()?,
    );

    let raw_positions = FixedVec::from_raw(
        (0..records[0].seq.len())
            .map(|index| RawPosition::new(sequences.make_vec(|_, sequence| sequence[index]), index))
            .collect(),
    );

    let positions = FixedVec::from_raw(
        raw_positions
            .values()
            .filter_map(|x| match x {
                RawPosition::Standard(sequences) => Some(PositionData::new(sequences.clone())),
                _ => None,
            })
            .collect(),
    );

    let mut root_stats = Stats::default();
    let mut other_stats = Stats::default();
    let mut sequence_stats = sequence_ids.make_vec(|_, _| Stats::default());

    for raw in raw_positions.values() {
        match raw {
            RawPosition::Standard(_) => {}
            RawPosition::Simple(reference, sequences) => {
                if *reference == AminoAcid::Gap {
                    for (sequence_id, &amino_acid) in sequences.iter() {
                        let sequence_stats = &mut sequence_stats[sequence_id];
                        if amino_acid != *reference && amino_acid != AminoAcid::Unknown {
                            sequence_stats.inserts.record(true);
                            sequence_stats.insert_probability *= model.initial(amino_acid);
                        }
                    }
                } else {
                    root_stats.initial.record(true);
                    root_stats.insert_probability *= model.initial(*reference);

                    other_stats.inserts.record(false);
                    other_stats.record_transition(*reference, *reference);
                    other_stats.deletes.record(false);

                    for (sequence_id, &amino_acid) in sequences.iter() {
                        let sequence_stats = &mut sequence_stats[sequence_id];
                        if amino_acid == *reference {
                            sequence_stats.inserts.record(false);
                            sequence_stats.record_transition(*reference, amino_acid);
                            sequence_stats.deletes.record(false);
                        } else if amino_acid == AminoAcid::Gap {
                            sequence_stats.inserts.record(false);
                            sequence_stats.deletes.record(true);
                        } else if amino_acid != AminoAcid::Unknown {
                            sequence_stats.inserts.record(false);
                            sequence_stats.record_transition(*reference, amino_acid);
                            sequence_stats.deletes.record(false);
                        }
                    }
                }
            }
        }
    }

    Ok(Alignment {
        sequence_ids,
        raw_positions,
        positions,
        root_stats,
        other_stats,
        sequence_stats,
    })
}
