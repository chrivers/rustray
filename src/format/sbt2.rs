use std::marker::PhantomData;
use crate::lib::result::{RResult, Error};
use std::collections::HashMap;
use std::str::FromStr;

use pest::iterators::{Pair, Pairs};
use pest_derive::Parser;
use pest::Parser;

use super::sbt::SbtVersion;

use crate::lib::float::Float;

#[derive(Parser)]
#[grammar = "format/sbt2.pest"]
pub struct SbtParser2<F: Float> {
    _p: PhantomData<F>
}

#[derive(Debug)]
pub struct SbtProgram<F: Float> {
    version: SbtVersion,
    blocks: Vec<SbtBlock<F>>,
}

#[derive(Debug)]
pub struct SbtBlock<F: Float> {
    name: String,
    value: SbtValue<F>,
}

#[derive(Debug)]
pub enum SbtValue<F: Float> {
    Int(i64),
    Float(F),
    Str(String),
    Dict(HashMap<String, SbtValue<F>>),
    TupleI3([i64; 3]),
    Tuple(Vec<SbtValue<F>>),
    Block(Box<SbtBlock<F>>),
    Bool(bool),
}

impl<F: Float> SbtParser2<F> {

    pub fn new() -> Self
    {
        Self {_p: PhantomData{}}
    }

    pub fn dump(&self, pr: Pairs<Rule>) -> RResult<()>
    {
        pub fn _dump(pr: Pair<Rule>, lvl: usize) -> RResult<()>
        {
            print!(
                "{}{:?}",
                "    ".repeat(lvl),
                pr.as_rule(),
            );
            match pr.as_rule() {
                Rule::ident => print!(" '{}'", pr.as_span().as_str()),
                Rule::int   => print!(" {}", { pr.as_span().as_str() } ),
                _other      => print!(""),
            }
            println!();
            for p in pr.into_inner() {
                _dump(p, lvl + 1)?;
            }
            Ok(())
        }

        for p in pr {
            _dump(p, 0)?;
        }

        Ok(())
    }

    pub fn parse_dict(&self, pr: Pairs<Rule>) -> RResult<SbtValue<F>>
    {
        let mut hash = HashMap::new();
        let mut key = "";
        for p in pr {
            match p.as_rule() {
                Rule::ident => {
                    key = p.as_span().as_str()
                }
                _value => {
                    hash.insert(key.to_string(), self.parse_value(p)?);
                }
            }
        }
        Ok(SbtValue::Dict(hash))
    }

    pub fn parse_tuple(&self, pr: Pairs<Rule>) -> RResult<SbtValue<F>>
    {
        let mut tuple = vec![];

        /* Manual iteration is significantly faster than map()+collect() */
        for p in pr {
            tuple.push(self.parse_value(p)?)
        }

        Ok(SbtValue::Tuple(tuple))
    }

    pub fn parse_tuple_i3(&self, pr: Pairs<Rule>) -> RResult<SbtValue<F>>
    {
        let it = pr.into_iter().collect::<Vec<_>>();
        Ok(SbtValue::TupleI3(
            [
                it[0].as_span().as_str().trim().parse()?,
                it[1].as_span().as_str().trim().parse()?,
                it[2].as_span().as_str().trim().parse()?,
            ]
        ))
    }

    pub fn parse_float(&self, pr: Pair<Rule>) -> RResult<SbtValue<F>>
    {
        Ok(SbtValue::Float(F::from_f64(pr.as_span().as_str().trim().parse::<f64>()?)))
    }

    pub fn parse_int(&self, pr: Pair<Rule>) -> RResult<SbtValue<F>>
    {
        Ok(SbtValue::Int(pr.as_span().as_str().trim().parse()?))
    }

    pub fn parse_boolean(&self, pr: Pair<Rule>) -> RResult<SbtValue<F>>
    {
        match pr.as_span().as_str() {
            "true"  => Ok(SbtValue::Bool(true)),
            "false" => Ok(SbtValue::Bool(false)),
            _       => panic!("internal parser error"),
        }
    }

    pub fn parse_string(&self, pr: Pair<Rule>) -> RResult<SbtValue<F>>
    {
        let val = pr.as_span().as_str();
        Ok(SbtValue::Str(val[1..val.len()-1].to_string()))
    }

    pub fn parse_value(&self, pr: Pair<Rule>) -> RResult<SbtValue<F>>
    {
        let value = match pr.as_rule() {
            Rule::tuple   => self.parse_tuple(pr.into_inner())?,
            Rule::float   => self.parse_float(pr)?,
            Rule::int     => self.parse_int(pr)?,
            Rule::dict    => self.parse_dict(pr.into_inner())?,
            Rule::string  => self.parse_string(pr)?,
            Rule::group   => self.parse_tuple(pr.into_inner())?,
            Rule::boolean => self.parse_boolean(pr)?,
            Rule::block   => SbtValue::Block(box self.parse_block(pr.into_inner())?),
            Rule::tuple_i3 => self.parse_tuple_i3(pr.into_inner())?,
            other => return Err(Error::ParseUnsupported(format!("{:?}", other)))
        };
        Ok(value)
    }

    pub fn parse_block(&self, pr: Pairs<Rule>) -> RResult<SbtBlock<F>>
    {
        let mut it = pr.into_iter();
        let name = it.next().unwrap().as_str().to_owned();
        let value = self.parse_value(it.next().unwrap())?;
        Ok(SbtBlock { name, value })
    }

    pub fn ast(&self, pr: Pairs<Rule>) -> RResult<SbtProgram<F>>
    {
        let mut prog = SbtProgram { version: SbtVersion::Sbt1_0, blocks: vec![] };
        let mut name = "";
        for p in pr {
            match p.as_rule() {
                Rule::VERSION => prog.version = SbtVersion::from_str(p.as_str())?,
                Rule::block => prog.blocks.push(self.parse_block(p.into_inner())?),
                Rule::ident => name = p.as_span().as_str(),
                Rule::dict  => {
                    /* warn!("Workaround for malformed file"); */
                    prog.blocks.push(SbtBlock {
                        name: name.to_string(),
                        value: self.parse_dict(p.into_inner())?
                    })
                }
                Rule::EOI => break,
                other => return Err(Error::ParseUnsupported(format!("{:?}", other)))
            }
        }
        Ok(prog)
    }

}
