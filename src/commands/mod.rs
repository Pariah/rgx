pub mod explain;
pub mod generate;
pub mod test;

use crate::error::Result;

pub trait Command {
    type Response: serde::de::DeserializeOwned;

    fn build_prompt(&self, input: &str) -> String;

    fn parse_response(&self, response: &str) -> Result<Self::Response> {
        Ok(serde_json::from_str(response)?)
    }
}
