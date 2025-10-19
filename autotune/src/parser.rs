use anyhow::{anyhow, Result};
use std::str::FromStr;

pub trait InputParser<T>
where
    T: FromStr + PartialOrd,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    fn parse(&self, content: &String) -> Result<T, anyhow::Error>;
}

pub struct ConstantParser {
    index: usize,
}

impl ConstantParser {
    pub fn new(index: usize) -> Self {
        Self { index }
    }
}

impl<T> InputParser<T> for ConstantParser
where
    T: FromStr + PartialOrd,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    fn parse(&self, input_content: &String) -> Result<T, anyhow::Error> {
        let tokens: Vec<&str> = input_content.split_whitespace().collect();
        if self.index >= tokens.len() {
            return Err(anyhow!("index out of range"));
        }
        let value = tokens[self.index].parse()?;
        Ok(value)
    }
}

#[test]
fn test_constant_parser() {
    let parser = ConstantParser::new(1);
    let value: i64 = parser.parse(&"10 20 30".to_string()).unwrap();
    assert_eq!(value, 20);
}
