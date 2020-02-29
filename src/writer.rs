use std::str::FromStr;

#[derive(Clone, Copy, Debug)]
pub enum QuantificationMethod {
    Count,
}

impl FromStr for QuantificationMethod {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "count" => Ok(Self::Count),
            _ => Err(()),
        }
    }
}