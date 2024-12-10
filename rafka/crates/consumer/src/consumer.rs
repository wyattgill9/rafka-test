use tonic::transport::Channel;
use futures::StreamExt;
use rafka_core::proto::rafka::{
    broker_service_client::BrokerServiceClient,
    RegisterRequest, SubscribeRequest, ConsumeRequest, ClientType,
};
use rafka_core::message::BenchmarkMetrics;
use chrono::{Utc, TimeZone};

pub struct Consumer {
    id: String,
    client: BrokerServiceClient<Channel>,
    partition_id: u32,
}

impl Consumer {
    pub async fn new(broker_addr: &str, partition_id: u32) -> Result<Self, Box<dyn std::error::Error>> {
        let id = uuid::Uuid::new_v4().to_string();
        let client = BrokerServiceClient::connect(format!("http://{}", broker_addr)).await?;
        let consumer = Self { id, client, partition_id };
        consumer.register().await?;
        Ok(consumer)
    }

    async fn register(&self) -> Result<(), Box<dyn std::error::Error>> {
        let request = RegisterRequest {
            client_id: self.id.clone(),
            client_type: ClientType::Consumer as i32,
        };

        self.client
            .clone()
            .register(request)
            .await?;

        println!("Consumer registered with ID: {}", self.id);
        Ok(())
    }

    pub async fn subscribe(&self, topic: String) -> Result<(), Box<dyn std::error::Error>> {
        let request = SubscribeRequest {
            consumer_id: self.id.clone(),
            topic,
        };

        self.client
            .clone()
            .subscribe(request)
            .await?;

        Ok(())
    }

    pub async fn start_consuming(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let request = ConsumeRequest {
            consumer_id: self.id.clone(),
        };

        let mut stream = self.client
            .clone()
            .consume(request)
            .await?
            .into_inner();

        println!("Consumer {} started consuming messages on partition {}", 
                self.id, self.partition_id);

        while let Some(message) = stream.next().await {
            match message {
                Ok(msg) => {
                    println!(
                        "Partition {} received message: {} on topic {} with payload: {}",
                        self.partition_id,
                        msg.message_id,
                        msg.topic,
                        String::from_utf8_lossy(&msg.payload)
                    );
                }
                Err(e) => eprintln!("Error receiving message: {}", e),
            }
        }

        Ok(())
    }

    pub async fn receive_benchmark(&mut self) -> Result<BenchmarkMetrics, Box<dyn std::error::Error>> {
        let request = ConsumeRequest {
            consumer_id: self.id.clone(),
        };

        let mut stream = self.client
            .clone()
            .consume(request)
            .await?
            .into_inner();

        if let Some(msg) = stream.next().await {
            let msg = msg?;
            let received_at = Utc::now();
            let sent_at = msg.sent_at
                .map(|ts| Utc.timestamp_opt(ts.seconds, ts.nanos as u32).unwrap())
                .unwrap_or_else(|| received_at);

            Ok(BenchmarkMetrics {
                message_id: msg.message_id,
                sent_at,
                received_at,
                latency_ms: (received_at - sent_at).num_milliseconds()
            })
        } else {
            Err("No message received".into())
        }
    }
}
