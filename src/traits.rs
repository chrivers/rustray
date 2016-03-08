use num;
use std::fmt::Debug;
use std::fmt::Display;

pub trait Float : num::Float + Debug + Display { }

impl Float for f32 { }
impl Float for f64 { }
