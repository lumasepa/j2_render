use crate::destination::Destination;
use crate::error::{ToWrapErrorResult, WrapError};
use crate::pairs::Pairs;

#[derive(Debug)]
pub struct Output {
    destination: Destination,
    format: Option<String>,
}

impl Output {
    pub fn try_from_pairs(pairs: Pairs) -> Result<Output, WrapError> {
        let destination = wrap_result!(
            Destination::try_from_pairs(&pairs),
            "Error parsing source from pairs : {}",
            pairs
        )?;
        let format = pairs.get("format").or(pairs.get("f"));
        return Ok(Output { destination, format });
    }
}
