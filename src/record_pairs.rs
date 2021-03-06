mod pair_position;

pub use self::pair_position::PairPosition;

use std::{
    collections::{hash_map::Drain, HashMap},
    convert::TryFrom,
    io,
};

use log::warn;
use noodles_bam as bam;

type RecordKey = (
    Vec<u8>,
    PairPosition,
    Option<i32>,
    Option<i32>,
    Option<i32>,
    Option<i32>,
    i32,
);

pub struct RecordPairs<I> {
    records: I,
    buf: HashMap<RecordKey, bam::Record>,
    primary_only: bool,
}

impl<I> RecordPairs<I>
where
    I: Iterator<Item = io::Result<bam::Record>>,
{
    pub fn new(records: I, primary_only: bool) -> RecordPairs<I> {
        RecordPairs {
            records,
            buf: HashMap::new(),
            primary_only,
        }
    }

    fn next_pair(&mut self) -> Option<io::Result<(bam::Record, bam::Record)>> {
        loop {
            let record = match self.records.next() {
                Some(result) => match result {
                    Ok(r) => r,
                    Err(e) => return Some(Err(e)),
                },
                None => {
                    if !self.buf.is_empty() {
                        warn!("{} records are singletons", self.buf.len());
                    }

                    return None;
                }
            };

            if self.primary_only && is_not_primary(&record) {
                continue;
            }

            let mate_key = match mate_key(&record) {
                Ok(k) => k,
                Err(e) => return Some(Err(e)),
            };

            if let Some(mate) = self.buf.remove(&mate_key) {
                return match mate_key.1 {
                    PairPosition::First => Some(Ok((mate, record))),
                    PairPosition::Second => Some(Ok((record, mate))),
                };
            }

            let key = match key(&record) {
                Ok(k) => k,
                Err(e) => return Some(Err(e)),
            };

            self.buf.insert(key, record.clone());
        }
    }

    pub fn singletons(&mut self) -> Singletons {
        Singletons {
            drain: self.buf.drain(),
        }
    }
}

impl<I> Iterator for RecordPairs<I>
where
    I: Iterator<Item = io::Result<bam::Record>>,
{
    type Item = io::Result<(bam::Record, bam::Record)>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_pair()
    }
}

fn is_not_primary(record: &bam::Record) -> bool {
    let flags = record.flags();
    flags.is_secondary() || flags.is_supplementary()
}

fn key(record: &bam::Record) -> io::Result<RecordKey> {
    Ok((
        record
            .read_name()
            .map(|s| s.to_bytes().to_vec())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
        PairPosition::try_from(record)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
        record.reference_sequence_id().map(i32::from),
        record.position().map(i32::from),
        record.mate_reference_sequence_id().map(i32::from),
        record.mate_position().map(i32::from),
        record.template_length(),
    ))
}

fn mate_key(record: &bam::Record) -> io::Result<RecordKey> {
    Ok((
        record
            .read_name()
            .map(|s| s.to_bytes().to_vec())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
        PairPosition::try_from(record)
            .map(|p| p.mate())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
        record.mate_reference_sequence_id().map(i32::from),
        record.mate_position().map(i32::from),
        record.reference_sequence_id().map(i32::from),
        record.position().map(i32::from),
        -record.template_length(),
    ))
}

pub struct Singletons<'a> {
    drain: Drain<'a, RecordKey, bam::Record>,
}

impl<'a> Iterator for Singletons<'a> {
    type Item = bam::Record;

    fn next(&mut self) -> Option<Self::Item> {
        self.drain.next().map(|(_, r)| r)
    }
}

#[cfg(test)]
mod tests {
    use noodles_sam as sam;

    use super::*;

    fn build_record_pair() -> Result<(bam::Record, bam::Record), Box<dyn std::error::Error>> {
        use sam::record::{Flags, Position, ReadName, ReferenceSequenceName};

        let read_name: ReadName = "r0".parse()?;
        let reference_sequence_name: ReferenceSequenceName = "sq0".parse()?;
        let position = Position::try_from(8)?;
        let mate_reference_sequence_name: ReferenceSequenceName = "sq1".parse()?;
        let mate_position = Position::try_from(13)?;

        let reference_sequences = vec![
            (
                String::from("sq0"),
                sam::header::ReferenceSequence::new(String::from("sq0"), 8),
            ),
            (
                String::from("sq1"),
                sam::header::ReferenceSequence::new(String::from("sq1"), 13),
            ),
        ]
        .into_iter()
        .collect();

        let s1 = sam::Record::builder()
            .set_read_name(read_name.clone())
            .set_flags(Flags::PAIRED | Flags::READ_1)
            .set_reference_sequence_name(reference_sequence_name.clone())
            .set_position(position)
            .set_mate_reference_sequence_name(mate_reference_sequence_name.clone())
            .set_mate_position(mate_position)
            .set_template_length(144)
            .build();

        let r1 = bam::Record::try_from_sam_record(&reference_sequences, &s1)?;

        let s2 = sam::Record::builder()
            .set_read_name(read_name)
            .set_flags(Flags::PAIRED | Flags::READ_2)
            .set_reference_sequence_name(mate_reference_sequence_name)
            .set_position(mate_position)
            .set_mate_reference_sequence_name(reference_sequence_name)
            .set_mate_position(position)
            .set_template_length(-144)
            .build();

        let r2 = bam::Record::try_from_sam_record(&reference_sequences, &s2)?;

        Ok((r1, r2))
    }

    #[test]
    fn test_key() -> Result<(), Box<dyn std::error::Error>> {
        let (r1, _) = build_record_pair()?;

        let actual = key(&r1)?;
        let expected = (
            b"r0".to_vec(),
            PairPosition::First,
            Some(0),
            Some(8),
            Some(1),
            Some(13),
            144,
        );

        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn test_mate_key() -> Result<(), Box<dyn std::error::Error>> {
        let (r1, _) = build_record_pair()?;

        let actual = mate_key(&r1)?;
        let expected = (
            b"r0".to_vec(),
            PairPosition::Second,
            Some(1),
            Some(13),
            Some(0),
            Some(8),
            -144,
        );

        assert_eq!(actual, expected);

        Ok(())
    }
}
