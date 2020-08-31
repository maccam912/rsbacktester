use rust_decimal::prelude::*;
use std::fmt::Debug;
use std::collections::VecDeque;

pub trait Indicator: Debug {
    fn value(&self) -> anyhow::Result<f64>;
    fn update(&mut self, stepvalue: Decimal);
}

#[derive(Debug)]
pub struct MovingAverage {
    pub length: usize,
    pub prices: VecDeque<Option<Decimal>>,
    pub input: String,
}

impl Indicator for MovingAverage {

    fn value(self: &MovingAverage) -> anyhow::Result<f64> {
        let mut sum: f64 = 0.;
        let mut count: f64 = 0.;
        for v in &self.prices {
            match v {
                Some(v) => {
                    sum += v.to_f64().expect("Could not convert price to f64");
                    count += 1.;
                },
                None => {},
            }
        }
        Ok(sum/count)
    }

    fn update(self: &mut MovingAverage, stepvalue: Decimal) {
        self.prices.push_front(Some(stepvalue));
        self.prices.truncate(self.length);

    }
}