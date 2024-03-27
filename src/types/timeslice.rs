use itertools::Itertools;
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct TimeSlice {
    map: HashMap<String, Duration>,
    name: String,
    time: Instant,
}

impl TimeSlice {
    pub fn new(start: &str) -> TimeSlice {
        TimeSlice {
            map: HashMap::new(),
            name: start.to_string(),
            time: Instant::now(),
        }
    }

    pub fn set(&mut self, new: &str) {
        let now = Instant::now();
        let dur = now - self.time;
        match self.map.get_mut(&self.name) {
            None => {
                self.map.insert(self.name.clone(), dur);
            }
            Some(ref mut d) => **d += dur,
        }
        self.name = new.to_string();
        self.time = now;
    }

    pub fn stop(&mut self) {
        self.set("");
    }

    pub fn get(&self, key: &'static str) -> Option<Duration> {
        self.map.get(key).copied()
    }

    pub fn show(&self) {
        let width = self.map.iter().map(|x| x.0.len()).max().unwrap_or(0) + 4;
        for (key, value) in self.map.iter().sorted_by_key(|x| x.1).rev() {
            info!(
                "  {:.<width$}:{:>9.2?} ms",
                format!("{} ", key),
                value.as_micros() as f32 / 1000f32,
                width = width
            );
        }
        let total = self.map.iter().map(|x| x.1.as_micros()).sum::<u128>() as f32;
        info!("Total: {:>9.2} ms", total / 1000f32);
    }
}
