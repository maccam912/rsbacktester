use rust_decimal::prelude::*;
use std::collections::VecDeque;
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub enum Indicator {
    MovingAverage(MovingAverage),
    Momentum(Momentum),
}

impl Indicator {
    pub fn get_input(self: &Self) -> String {
        match self {
            Indicator::MovingAverage(i) => i.get_input(),
            Indicator::Momentum(i) => i.get_input(),
        }
    }
    pub fn update(self: &mut Self, stepvalue: Option<f64>) {
        match self {
            Indicator::MovingAverage(i) => i.update(stepvalue),
            Indicator::Momentum(i) => i.update(stepvalue),
        }
    }
    pub fn value(self: &Self) -> Option<f64> {
        match self {
            Indicator::MovingAverage(i) => i.value(),
            Indicator::Momentum(i) => i.value(),
        }
    }
}

/// `MovingAverage` is defined by the `length` the MA should look back
/// and an `input: String` which can contain "price" to use the latest prices, or another string to give you
/// a Moving Average of another `Indicator`.
#[derive(Debug, Clone)]
pub struct MovingAverage {
    pub length: usize,
    pub input: String,
    operands: VecDeque<Option<f64>>,
}

impl MovingAverage {
    pub fn new(length: usize, input: String) -> Self {
        Self {
            length,
            input,
            operands: VecDeque::new(),
        }
    }
}

impl MovingAverage {
    fn value(self: &Self) -> Option<f64> {
        let mut sum: f64 = 0.;
        let mut count: f64 = 0.;
        for v in &self.operands {
            match v {
                Some(v) => {
                    sum += v.to_f64().expect("Could not convert price to f64");
                    count += 1.;
                }
                None => {}
            }
        }
        Some(sum / count)
    }

    fn get_input(self: &Self) -> String {
        (&self.input).to_string()
    }

    fn update(self: &mut Self, stepvalue: Option<f64>) {
        self.operands.push_front(stepvalue);
        self.operands.truncate(self.length);
    }
}

/// `Momentum` is defined by the `length` the Momentum indicator should look back
/// and an `input: String` which can contain "price" to use the latest prices, or another string to give you
/// a Moving Average of another `Indicator`.
#[derive(Debug, Clone)]
pub struct Momentum {
    pub length: usize,
    pub input: String,
    operands: VecDeque<Option<f64>>,
}

impl Momentum {
    pub fn new(length: usize, input: String) -> Self {
        Self {
            length,
            input,
            operands: VecDeque::new(),
        }
    }
}

impl Momentum {
    fn value(self: &Self) -> Option<f64> {
        let l = self.operands.len();
        if l == 0 {
            None
        } else {
            let a = self.operands[l - 1].expect("Last element of operands is `None`");
            let b = self.operands[0].expect("First element of operands is `None`");
            let diff = b - a;
            Some(diff)
        }
    }

    fn get_input(self: &Self) -> String {
        (&self.input).to_string()
    }

    fn update(self: &mut Self, stepvalue: Option<f64>) {
        self.operands.push_front(stepvalue);
        self.operands.truncate(self.length);
    }
}
