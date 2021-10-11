use std::collections::HashMap;
use std::time::{Instant, Duration};
use itertools::Itertools;

pub struct TimeSlice
{
    map: HashMap<&'static str, Duration>,
    name: &'static str,
    time: Instant,
}

impl TimeSlice
{
    pub fn new(start: &'static str) -> TimeSlice
    {
        TimeSlice { map: HashMap::new(), name: start, time: Instant::now() }
    }

    pub fn set(&mut self, new: &'static str)
    {
        let now = Instant::now();
        let dur = now - self.time;
        match self.map.get_mut(self.name) {
            None => {
                self.map.insert(self.name, dur);
            },
            Some(ref mut d) => {
                **d = **d + dur
            }
        }
        self.name = new;
        self.time = now;
    }

    pub fn stop(&mut self) {
        self.set("")
    }

    pub fn get(&self, key: &'static str) -> Option<Duration>
    {
        self.map.get(key).map(|x| *x)
    }

    pub fn show(&self)
    {
        for (key, value) in self.map.iter().sorted_by_key(|x| x.1).rev() {
            info!("  {:.<12}:{:>12.2?} ms", key, value.as_micros() as f32 / 1000f32);
        }
    }
}
