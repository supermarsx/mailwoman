//! Version-two integer-millisecond histogram, dense while recording and sparse on disk.
use serde::{Deserialize, Deserializer, Serialize, Serializer, ser::SerializeSeq};

pub const LAST_FINITE_INDEX: usize = 2940;
pub const OVERFLOW_INDEX: usize = LAST_FINITE_INDEX + 1;
pub const BIN_COUNT: usize = OVERFLOW_INDEX + 1;

pub const fn upper_bound(index: usize) -> u64 {
    match index {
        0..=1000 => index as u64,
        1001..=1900 => 1000 + (index as u64 - 1000) * 10,
        1901..=2400 => 10000 + (index as u64 - 1900) * 100,
        _ => 60000 + (index as u64 - 2400) * 1000,
    }
}
const fn bounds() -> [u64; LAST_FINITE_INDEX + 1] {
    let mut values = [0; LAST_FINITE_INDEX + 1];
    let mut index = 0;
    while index <= LAST_FINITE_INDEX {
        values[index] = upper_bound(index);
        index += 1;
    }
    values
}
pub static BOUNDS_MS: [u64; LAST_FINITE_INDEX + 1] = bounds();
fn index(ms: u64) -> usize {
    match ms {
        0..=1000 => ms as usize,
        1001..=10000 => 1000 + (ms - 1000).div_ceil(10) as usize,
        10001..=60000 => 1900 + (ms - 10000).div_ceil(100) as usize,
        60001..=600000 => 2400 + (ms - 60000).div_ceil(1000) as usize,
        _ => OVERFLOW_INDEX,
    }
}

#[derive(Clone, Debug)]
pub enum Histogram {
    Dense(Vec<u64>),
    Sparse(Vec<(u16, u64)>),
}
impl PartialEq for Histogram {
    fn eq(&self, other: &Self) -> bool {
        self.entries().eq(other.entries())
    }
}
impl Eq for Histogram {}
impl Default for Histogram {
    fn default() -> Self {
        Self::Dense(Vec::new())
    }
}
impl Histogram {
    pub(super) fn entries(&self) -> Box<dyn Iterator<Item = (usize, u64)> + '_> {
        match self {
            Self::Dense(values) => Box::new(
                values
                    .iter()
                    .copied()
                    .enumerate()
                    .filter(|(_, count)| *count > 0),
            ),
            Self::Sparse(values) => Box::new(
                values
                    .iter()
                    .map(|(index, count)| (*index as usize, *count)),
            ),
        }
    }
    fn add(&mut self, index: usize, count: u64) {
        if let Self::Sparse(entries) = self {
            let mut dense = vec![0; BIN_COUNT];
            for (index, count) in entries {
                dense[*index as usize] = *count;
            }
            *self = Self::Dense(dense);
        }
        if let Self::Dense(values) = self {
            if values.is_empty() {
                values.resize(BIN_COUNT, 0);
            }
            values[index] = values[index].saturating_add(count);
        }
    }
    pub fn record(&mut self, ms: u64) {
        self.add(index(ms), 1);
    }
    pub fn merge(&mut self, other: &Self) {
        for (index, count) in other.entries() {
            self.add(index, count);
        }
    }
    pub fn compatible_max(&self, max: u64) -> bool {
        self.entries()
            .all(|(index, _)| index == 0 || upper_bound(index - 1) < max)
    }
    pub fn total(&self) -> Option<u64> {
        self.entries()
            .try_fold(0_u64, |sum, (_, count)| sum.checked_add(count))
    }
    pub fn percentile(&self, rank: u64, max: u64) -> Option<(u64, Option<u64>)> {
        let mut seen = 0_u64;
        for (index, count) in self.entries() {
            seen = seen.saturating_add(count);
            if seen >= rank {
                return Some(if index == OVERFLOW_INDEX {
                    (max, None)
                } else {
                    let resolution = match index {
                        0..=1000 => 1,
                        1001..=1900 => 10,
                        1901..=2400 => 100,
                        _ => 1000,
                    };
                    (upper_bound(index), Some(resolution))
                });
            }
        }
        None
    }
}
impl Serialize for Histogram {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(None)?;
        for (index, count) in self.entries() {
            seq.serialize_element(&(index as u16, count))?;
        }
        seq.end()
    }
}
impl<'de> Deserialize<'de> for Histogram {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let entries = Vec::<(u16, u64)>::deserialize(deserializer)?;
        if entries.len() > BIN_COUNT
            || entries
                .iter()
                .any(|(index, count)| *index as usize >= BIN_COUNT || *count == 0)
            || entries.windows(2).any(|pair| pair[0].0 >= pair[1].0)
        {
            return Err(serde::de::Error::custom(
                "invalid bounded latency histogram",
            ));
        }
        Ok(Self::Sparse(entries))
    }
}
