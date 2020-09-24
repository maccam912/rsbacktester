# RSBacktester

Now with more actors!

## Account
The `Account` keeps track of how much `cash: Ratio<i64>` is held and which `position: HashMap<String, Position>`s are open.

## Position
The `Position` notes the `asset: String`, number of lots `lots: i64`, and `costbasis: Ratio<i64>` of any position. Two positions can be combined as long as they have the same `asset: String`.

## Indicators
An `Indicator` is one of several subtypes: `MovingAverage` and `Momentum`, and has a `name: String`. In each case they have a `length: u64` of the number of bars back they look, and an `input: String` that is either `"price"` or the name of another indicator.

## Bar
A `Bar` is the standard candle, with an `open: Ratio<i64>`, `high: Ratio<i64>`, `low: Ratio<i64>`, and `close: Ratio<i64>`, as well as a `timestamp: DateTime<Utc>`.

## BarProducer
A `BarProducer` pushes `Bar`s to the `Engine`, either from a CSV in backtesting or live data in live trading mode.

## Engine
The `Engine` has a reference to an `Account`, a `HashMap<String, Indicator>` keeping track of all the indiactors set up, and a history of `Bar`s.
