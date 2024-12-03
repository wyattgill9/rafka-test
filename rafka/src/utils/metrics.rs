use prometheus::{Counter, Gauge, Histogram, Registry};
use std::sync::Arc;

pub struct Metrics {
    registry: Registry,
    message_count: Counter,
    message_size: Histogram,
    broker_load: Gauge,
    consumer_lag: Gauge,
}

impl Metrics {
    pub fn new() -> Self {
        let registry = Registry::new();
        
        let message_count = Counter::new(
            "rafka_messages_total",
            "Total number of messages processed"
        ).unwrap();
        
        let message_size = Histogram::with_opts(
            HistogramOpts::new(
                "rafka_message_size_bytes",
                "Message size distribution in bytes"
            )
        ).unwrap();

        registry.register(Box::new(message_count.clone())).unwrap();
        registry.register(Box::new(message_size.clone())).unwrap();

        Self {
            registry,
            message_count,
            message_size,
            broker_load,
            consumer_lag,
        }
    }

    pub fn record_message(&self, size: usize) {
        self.message_count.inc();
        self.message_size.observe(size as f64);
    }
}
