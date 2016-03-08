use num;
use std::fmt::Debug;

pub trait Float : num::Float + Debug { }

impl Float for f32 { }
impl Float for f64 { }
