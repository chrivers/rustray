use std::fmt::Debug;
use std::str::FromStr;
use std::marker::PhantomData;

use cgmath::Vector4;

use pest::iterators::Pair;
use pest_derive::Parser;

use crate::lib::{RResult, Error::ParseError};
use crate::{Vector, Point, Float, Color, point};

#[derive(Parser)]
#[grammar = "format/sbt.pest"]
pub struct SbtParser<F> {
    _p: PhantomData<F>
}

#[derive(Copy, Clone)]
pub enum SbtVersion {
    Sbt0_9,
    Sbt1_0,
}

impl FromStr for SbtVersion {
    type Err = crate::lib::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "0.9" => Ok(SbtVersion::Sbt0_9),
            "1.0" => Ok(SbtVersion::Sbt1_0),
            _ => panic!("impossible"),
        }
    }
}

impl<F> SbtParser<F>
where
    F: Float + FromStr,
{
    pub fn parse_bool(p: Pair<Rule>) -> bool {
        match p.into_inner().next().map(|p| p.as_str()) {
            Some("false") => false,
            Some("true") => true,
            _ => panic!("internal parser error"),
        }
    }

    pub fn parse_num1(m: Pair<Rule>) -> F {
        m.as_str().parse().unwrap_or(F::ZERO)
    }

    pub fn parse_val1(p: Pair<Rule>) -> RResult<F>
    {
        let mut m = p.into_inner();
        m.next().unwrap().as_str().trim().parse().or(Err(ParseError()))
    }

    pub fn parse_val2(p: Pair<Rule>) -> Point<F> {
        let m = p.into_inner();
        let v = m.map(
            |x| x.as_str().trim().parse().unwrap_or(F::ZERO)
        ).collect::<Vec<_>>();
        point!(v[0], v[1])
    }

    pub fn parse_int3(p: Pair<Rule>) -> [usize; 3] {
        let m = p.into_inner();
        let v = m.map(
            |x| x.as_str().trim().parse().unwrap_or(0)
        ).collect::<Vec<usize>>();
        [v[0], v[1], v[2]]
    }

    pub fn parse_val3(p: Pair<Rule>) -> Vector<F> {
        Self::parse_val3b(p.into_inner().next().unwrap())
    }

    pub fn parse_val3b(p: Pair<Rule>) -> Vector<F> {
        let m = p.into_inner();
        let v = m.map(
            |x| x.as_str().trim().parse().unwrap_or(F::ZERO)
        ).collect::<Vec<F>>();
        Vector::new(v[0], v[1], v[2])
    }

    pub fn parse_val4(p: Pair<Rule>) -> Vector4<F> {
        let m = p.into_inner();
        let v = m.map(
            |x| x.as_str().trim().parse().unwrap_or(F::ZERO)
        ).collect::<Vec<F>>();
        Vector4::new(v[0], v[1], v[2], v[3])
    }

    pub fn parse_color(p: Pair<Rule>) -> Color<F>
    {
        let m = p.into_inner().next().unwrap().into_inner();
        let v = m.map(
            |x| x.as_str().trim().parse().unwrap_or(F::ZERO)
        ).collect::<Vec<F>>();
        Color::new(v[0], v[1], v[2])
    }
}
