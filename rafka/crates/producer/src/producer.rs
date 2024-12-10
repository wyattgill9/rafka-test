use tonic::transport::Channel;
use rafka_core::proto::rafka::{
    broker_service_client::BrokerServiceClient,
    RegisterRequest, PublishRequest, ClientType,
};

pub struct Producer {
    id: String,
    clients: Vec<BrokerServiceClient<Channel>>,
    partition_count: u32,
}

impl Producer {
    pub async fn new(broker_addrs: &[String]) -> Result<Self, Box<dyn std::error::Error>> {
        let id = uuid::Uuid::new_v4().to_string();
        let mut clients = Vec::new();
        
        for addr in broker_addrs {
            let client = BrokerServiceClient::connect(format!("http://{}", addr)).await?;
            clients.push(client);
        }

        let producer = Self { 
            id, 
            clients,
            partition_count: broker_addrs.len() as u32,
        };
        
        producer.register_all().await?;
        Ok(producer)
    }

    pub async fn send(&mut self, topic: String, key: String, payload: Vec<u8>) -> Result<String, Box<dyn std::error::Error>> {
        let partition = self.get_partition(&key);
        let client = &self.clients[partition as usize];

        println!("Sending message with key '{}' to partition {}", key, partition);

        let request = PublishRequest {
            producer_id: key,  // Use the key for partition routing
            topic,
            payload,
        };

        let response = client.clone().publish(request).await?;
        let response = response.into_inner();
        Ok(response.message_id)
    }

    fn get_partition(&self, key: &str) -> u32 {
        // Consistent hash-based partitioning
        let hash: u32 = key.bytes().fold(0u32, |acc, b| acc.wrapping_add(b as u32));
        hash % self.partition_count
    }

    async fn register_all(&self) -> Result<(), Box<dyn std::error::Error>> {
        for client in &self.clients {
            let request = RegisterRequest {
                client_id: self.id.clone(),
                client_type: ClientType::Producer as i32,
            };
            client.clone().register(request).await?;
        }
        Ok(())
    }
}
