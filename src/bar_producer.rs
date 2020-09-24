use actix::prelude::*;

use crate::engine::Engine;

pub struct BarProducer {
    engine: Addr<Engine>,
}

impl Actor for BarProducer {
    type Context = Context<Self>;
}