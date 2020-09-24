use actix::prelude::*;
use hashbrown::HashMap;

pub struct Indicators(HashMap<String, Indicator>);

impl Actor for Indicators{
    type Context = Context<Self>;
}

pub struct Indicator {
    name: String,
    input: String,
    length: u64,
}