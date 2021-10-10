use pest::Parser;
use pest_derive::Parser;
#[derive(Parser)]
#[grammar = "format/sbt.pest"]
pub struct SbtParser;
