use crate::destination::Destination;
use crate::pairs::Pairs;
use crate::error::{WrapError, ToWrapErrorResult};

#[derive(Debug)]
pub struct Output {
    destination: Destination,
    format: Option<String>,
}

impl Output {
    pub fn try_from_pairs(pairs: Pairs) -> Result<Output, WrapError> {
        let destination = Destination::try_from_pairs(&pairs).wrap("Error parsing source from pairs")?;
        let format = pairs.get("format").or(pairs.get("f"));
        return Ok(Output { destination, format });
    }
}
