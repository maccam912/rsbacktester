use rust_decimal::prelude::*;
use crate::Engine;
use std::fmt::Debug;

pub trait Indicator: Debug {
    fn value(&self) -> anyhow::Result<f64>;
    fn update(&mut self, engine: &Engine);
}

#[derive(Debug)]
pub struct MovingAverage {
    length: i32,
    prices: Vec<Option<Decimal>>,
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

    fn update(self: &mut MovingAverage, engine: &Engine) {
        let bbo = engine.prices.ticks[engine.index as usize];
        let sum = bbo.ask.checked_add(bbo.bid).expect("Could not add ask and bid decimals");
        let avg = sum.checked_div(Decimal::new(2, 0)).expect("Could not divide bid+ask by 2");
        self.prices.push(Some(avg));
    }
}