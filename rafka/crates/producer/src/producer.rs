use tonic::transport::Channel;
use rafka_core::proto::rafka::{
    broker_service_client::BrokerServiceClient,
    RegisterRequest, PublishRequest, ClientType,
};
use chrono::{DateTime, Utc};

pub struct Producer {
    id: String,
    client: BrokerServiceClient<Channel>,
}

impl Producer {
    pub async fn new(broker_addr: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let id = uuid::Uuid::new_v4().to_string();
        let client = BrokerServiceClient::connect(format!("http://{}", broker_addr)).await?;
        let producer = Self { id, client };
        producer.register().await?;
        Ok(producer)
    }

    async fn register(&self) -> Result<(), Box<dyn std::error::Error>> {
        let request = RegisterRequest {
            client_id: self.id.clone(),
            client_type: ClientType::Producer as i32,
        };

        self.client
            .clone()
            .register(request)
            .await?;

        println!("Producer registered with ID: {}", self.id);
        Ok(())
    }

    pub async fn send(&mut self, topic: String, payload: Vec<u8>) -> Result<String, Box<dyn std::error::Error>> {
        let request = PublishRequest {
            producer_id: self.id.clone(),
            topic,
            payload,
        };

        let response = self.client
            .clone()
            .publish(request)
            .await?;

        let response = response.into_inner();
        println!("Message sent with ID: {}", response.message_id);
        Ok(response.message_id)
    }

    pub async fn send_benchmark(&mut self, topic: String, payload: Vec<u8>) -> Result<(String, DateTime<Utc>), Box<dyn std::error::Error>> {
        let sent_at = Utc::now();
        let request = PublishRequest {
            producer_id: self.id.clone(),
            topic,
            payload,
        };

        let response = self.client
            .clone()
            .publish(request)
            .await?;

        let response = response.into_inner();
        Ok((response.message_id, sent_at))
    }
}
