#![allow(dead_code)]
use chrono::prelude::*;
use hashbrown::HashMap;
use rust_decimal::prelude::*;
use serde::Deserialize;
use std::path::Path;

pub mod indicators;
pub mod position;
pub mod account;

use account::Account;

#[derive(Debug, Clone)]
pub struct Tick {
    pub timestamp: DateTime<Utc>,
    pub asset: String,
    pub bid: Decimal,
    pub ask: Decimal,
}

/// `TS` is a time series of `Tick`s.
/// ```
/// use rsbacktester::{Tick, TS};
/// use rust_decimal::Decimal;
/// use chrono::Utc;
///
/// let t = Tick{timestamp: Utc::now(), asset: "AAPL".to_string(), bid: Decimal::new(202, 2), ask: Decimal::new(203, 1)};
/// let ts = TS{ticks: vec![t]};
/// # assert!(ts.ticks.len() == 1);
/// ```
#[derive(Debug, Clone)]
pub struct TS {
    pub ticks: Vec<Tick>,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Engine {
    pub acct: account::Account,
    pub time: DateTime<Utc>,
    pub prices: TS,
    pub index: i64,
    pub indicators: hashbrown::HashMap<String, indicators::Indicator>,
    pub last_price: hashbrown::HashMap<String, Decimal>,
}

unsafe impl Send for Engine {}
unsafe impl Sync for Engine {}

#[derive(Debug)]
pub struct Signal {
    pub asset: String,
    pub direction_up: bool,
    pub magnitude: Option<f32>,
    pub duration: Option<i32>,
}

impl Engine {
    pub fn step(self: &mut Engine) {
        let iv = self.indicator_values();
        self.update_indicators(iv);
        let _signals = self.check_signals();
        let tick = &self.prices.ticks[self.index as usize];
        self.last_price.insert(tick.asset.clone(), (tick.ask.checked_add(tick.bid)).unwrap().checked_div(Decimal::new(2,0)).unwrap());
        self.index += 1;
    }

    pub fn check_signals(self: &Engine) -> Vec<Signal> {
        let s = Signal {
            asset: "SPY".to_string(),
            direction_up: true,
            magnitude: None,
            duration: None,
        };
        vec![s]
    }

    pub fn register_indicator(
        self: &mut Engine,
        name: String,
        indicator: indicators::Indicator,
    ) {
        self.indicators.insert(name, indicator);
    }

    fn indicator_values(self: &Engine) -> HashMap<String, Option<f64>> {
        let mut values = HashMap::new();
        for (name, ind) in &self.indicators {
            let v = ind.value();
            values.insert(name.to_string(), v);
        }
        values
    }

    pub fn update_indicators(self: &mut Engine, ind_values: HashMap<String, Option<f64>>) {
        for (_, indicator) in &mut self.indicators {
            if &indicator.get_input() == "price" {
                let stepvaluetick = self.prices.ticks[self.index as usize].clone();
                let stepvaluesum = stepvaluetick.ask.checked_add(stepvaluetick.bid);
                let stepvalue = stepvaluesum.unwrap().checked_div(Decimal::new(2, 0));
                let v: Option<f64> = stepvalue.expect("Decimal was screwy").to_f64();
                indicator.update(v);
            } else {
                let v = ind_values[&indicator.get_input()];
                indicator.update(v);
            }
        }
    }

    pub fn reset(self: &mut Engine, cash: f64) {
        self.acct.cash = Decimal::from_f64(cash).unwrap();
        self.acct.portfolio = HashMap::new();
        self.index = 0;
        for i in self.indicators.values_mut() {
            i.reset();
        }
    }

    pub fn equity(self: &Self) -> Decimal {
        let mut total_equity = Decimal::new(0, 0);
        for (asset, pos) in &self.acct.portfolio {
            let equity = self.last_price[asset].checked_mul(Decimal::new(pos.lots, 0)).unwrap_or(Decimal::new(0,0));
            total_equity = total_equity.checked_add(equity).unwrap();
        }
        total_equity = total_equity.checked_add(self.acct.cash).unwrap();
        total_equity
    }
}

#[derive(Debug, Deserialize)]
struct Record {
    #[serde(rename = "Date")]
    pub date: String,
    #[serde(rename = "Time")]
    pub time: String,
    #[serde(rename = "Asset")]
    pub asset: String,
    #[serde(rename = "Bid")]
    pub bid: String,
    #[serde(rename = "Ask")]
    pub ask: String,
}

fn init_acct(cash: i64) -> Account {
    Account {
        cash: Decimal::from(cash),
        portfolio: HashMap::new(),
        trades: vec![],
    }
}

fn record_to_tick(r: &Record) -> anyhow::Result<Tick> {
    let d = NaiveDate::parse_from_str(&r.date, "%Y/%m/%d")?;
    let t = NaiveTime::parse_from_str(&r.time, "%H:%M:%S%.f")?;
    let dt = NaiveDateTime::new(d, t);
    let utc_dt: DateTime<Utc> = DateTime::from_utc(dt, Utc);

    let ask: Decimal = Decimal::from_str(&r.ask)?;
    let bid: Decimal = Decimal::from_str(&r.bid)?;
    Ok(Tick {
        timestamp: utc_dt,
        asset: r.asset.clone(),
        ask: ask,
        bid: bid,
    })
}

fn init_prices<P: AsRef<Path>>(path: &P) -> anyhow::Result<TS> {
    let mut rdr = csv::Reader::from_path(path)?;
    let mut ticks = vec![];
    for result in rdr.deserialize() {
        let record: Record = result?;
        let tick = record_to_tick(&record)?;
        ticks.push(tick);
    }

    Ok(TS { ticks })
}

pub fn init_engine<P: AsRef<Path>>(path: &P, cash: i64) -> Engine {
    let prices: TS = init_prices(path).expect("could not load prices");
    let t1 = prices.ticks[0].timestamp;
    Engine {
        acct: init_acct(cash),
        time: t1,
        prices: prices,
        index: 0,
        indicators: HashMap::new(),
        last_price: HashMap::new(),
    }
}

#[cfg(test)]
mod tests {
    use crate::{indicators, indicators::Indicator, init_engine, Tick, TS, account::Account, position::Position};
    use hashbrown::HashMap;
    use chrono::prelude::*;
    use rust_decimal::Decimal;
    use std::path::Path;

    #[test]
    fn test_tick() {
        let t = Tick {
            timestamp: Utc::now(),
            asset: "AAPL".to_string(),
            bid: Decimal::new(202, 2),
            ask: Decimal::new(203, 1),
        };
        let twenty = Decimal::new(20, 0);
        let thirty = Decimal::new(30, 0);
        assert!(t.ask.ge(&twenty));
        assert!(t.bid.le(&thirty));
        assert!(t.timestamp <= Utc::now());
    }

    #[test]
    fn test_ts() {
        let t = Tick {
            timestamp: Utc::now(),
            asset: "AAPL".to_string(),
            bid: Decimal::new(202, 2),
            ask: Decimal::new(203, 1),
        };
        let ts = TS { ticks: vec![t] };
        assert!(ts.ticks.len() == 1);
    }

    #[test]
    fn test_engine() {
        let path = Path::new("test_resources/ticks.csv");
        let _engine = init_engine(&path, 10000);
    }

    #[test]
    fn test_init_engine() {
        let _ = init_engine(&"test_resources/ticks.csv", 10000);
    }

    #[test]
    #[ignore]
    fn test_large_dataframe() {
        let _ = init_engine(&"test_resources/mgcticks.csv", 10000);
    }

    #[test]
    fn test_moving_average() {
        let mut engine = init_engine(&"test_resources/ticks.csv", 10000);
        let i = indicators::MovingAverage::new(10, "price".to_string());
        engine.register_indicator("ind1".to_string(), Indicator::MovingAverage(i));
        engine.step();
        engine.step();
        assert!(
            engine.indicators["ind1"]
                .value()
                .expect("Could not get MA value")
                == 0.5
        );
    }

    #[test]
    fn test_momentum() {
        let mut engine = init_engine(&"test_resources/ticks.csv", 10000);
        println!("Engine initialized");
        let i = indicators::MovingAverage::new(4, "price".to_string());
        engine.register_indicator("ind2".to_string(), Indicator::MovingAverage(i));
        let mom = indicators::Momentum::new(3, "ind2".to_string());
        engine.register_indicator("mom".to_string(), Indicator::Momentum(mom));
        for _ in 0..engine.prices.ticks.len() {
            engine.step();
        }
        assert!(engine.indicators["ind2"].value().unwrap() == 27.5);
        assert!(engine.indicators["mom"].value().unwrap() == 2.0);
    }

    #[test]
    fn acct_open_position() {
        let mut acct = Account{cash: Decimal::new(10000, 0), portfolio: HashMap::new(), trades: vec![]};
        let pos = Position{asset: "AAPL".to_string(), lots: 1, cost_basis: Decimal::new(1000, 0)};
        let resp = acct.order(pos);
        assert!(resp.is_ok());
        assert!(acct.portfolio.len() == 1);
        assert!(acct.cash.eq(&Decimal::new(9000, 0)));
        assert!(acct.portfolio["AAPL"].lots == 1);

        let p2 = Position{asset: "MSFT".to_string(), lots: 1, cost_basis: Decimal::new(2000, 0)};
        let resp = acct.order(p2);
        assert!(resp.is_ok());
        assert!(acct.portfolio.len() == 2);
        assert!(acct.cash.eq(&Decimal::new(7000, 0)));
        assert!(acct.portfolio["MSFT"].lots == 1);


        let p3 = Position{asset: "AAPL".to_string(), lots: 2, cost_basis: Decimal::new(250, 0)};
        let resp = acct.order(p3);
        assert!(resp.is_ok());
        assert!(acct.portfolio.len() == 2);
        assert!(acct.cash.eq(&Decimal::new(6500, 0)));
        assert!(acct.portfolio.get("AAPL").unwrap().cost_basis == Decimal::new(500, 0));
        assert!(acct.portfolio["AAPL"].lots == 3);


        let p4 = Position{asset: "AAPL".to_string(), lots: 1, cost_basis: Decimal::new(10000, 0)};
        let resp = acct.order(p4);
        assert!(resp.is_err());
        assert!(acct.cash.eq(&Decimal::new(6500, 0)));
        assert!(acct.portfolio.get("AAPL").unwrap().lots == 3);
    }

    #[test]
    fn acct_open_close() {
        let mut acct = Account{cash: Decimal::new(10000, 0), portfolio: HashMap::new(), trades: vec![]};
        let pos = Position{asset: "AAPL".to_string(), lots: 1, cost_basis: Decimal::new(1000, 0)};
        let resp = acct.order(pos);
        assert!(resp.is_ok());
        assert!(acct.portfolio.len() == 1);
        assert!(acct.cash.eq(&Decimal::new(9000, 0)));

        let pos = Position{asset: "AAPL".to_string(), lots: -1, cost_basis: Decimal::new(3000, 0)};
        let resp = acct.order(pos);
        assert!(resp.is_ok());
        assert!(acct.portfolio.len() == 0);
        assert!(acct.cash.eq(&Decimal::new(12000, 0)));
    }

    #[test]
    fn total_equity() {
        let mut e = init_engine(&"test_resources/ticks.csv", 10000);
        assert!(e.equity() == Decimal::new(10000, 0));
        let result = e.acct.order(Position{asset: "AAPL".to_string(), lots: 1, cost_basis: Decimal::new(2, 0)});
        assert!(result.is_ok());
        e.step();
        assert!(e.equity() == Decimal::new(9998+0, 0));
        e.step();
        assert!(e.equity() == Decimal::new(9998+1, 0));
        e.step();
        assert!(e.equity() == Decimal::new(9998+2, 0));
        e.step();
        assert!(e.equity() == Decimal::new(9998+3, 0));
    }
}
