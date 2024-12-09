use std::sync::Arc;
use tokio::net::TcpStream;
use rafka_core::{Message, Result, Error, NetworkMessage};
use crate::network::NetworkLayer;
use crate::storage::Storage;
use crate::metrics::MetricsCollector;
use crate::partition::PartitionManager;

pub struct StatelessBroker {
    pub node_id: String,
    pub config: Arc<rafka_core::Config>,
    pub storage: Arc<Storage>,
    pub network: Arc<NetworkLayer>,
    pub metrics: Arc<MetricsCollector>,
    pub partition_manager: Arc<PartitionManager>,
}

impl StatelessBroker {
    pub fn new(config: rafka_core::Config) -> Self {
        let node_id = uuid::Uuid::new_v4().to_string();
        
        Self {
            node_id: node_id.clone(),
            config: Arc::new(config),
            storage: Arc::new(Storage::new()),
            network: Arc::new(NetworkLayer::new()),
            metrics: Arc::new(MetricsCollector::new()),
            partition_manager: Arc::new(PartitionManager::new(node_id)),
        }
    }

    pub async fn route_message(&self, message: Message) -> Result<()> {
        let partition = self.partition_manager.get_partition_for_message(&message)?;
        
        // Store message
        self.storage.store_message(partition, message.clone()).await?;

        // Replicate to backup nodes
        self.network.replicate_message(partition, &message).await?;

        Ok(())
    }

    pub async fn start_network(self: Arc<Self>) -> Result<()> {
        let addr = format!("0.0.0.0:{}", self.config.broker_port);
        let listener = tokio::net::TcpListener::bind(&addr).await?;
        tracing::info!("Broker listening on {}", addr);

        loop {
            match listener.accept().await {
                Ok((socket, addr)) => {
                    tracing::info!("New connection from {}", addr);
                    let broker = self.clone();
                    tokio::spawn(async move {
                        if let Err(e) = broker.handle_connection(socket).await {
                            tracing::error!("Error handling connection: {}", e);
                        }
                    });
                }
                Err(e) => {
                    tracing::error!("Error accepting connection: {}", e);
                }
            }
        }
    }

    async fn handle_connection(&self, socket: TcpStream) -> Result<()> {
        let mut buf = [0u8; 1];
        socket.readable().await?;
        socket.try_read(&mut buf)?;

        match buf[0] {
            1 => self.handle_producer(socket).await,
            2 => self.handle_consumer(socket).await,
            _ => Err(Error::InvalidInput("Unknown connection type".into())),
        }
    }

    async fn handle_producer(&self, socket: TcpStream) -> Result<()> {
        let producer_id = uuid::Uuid::new_v4().to_string(); 
        self.network.add_producer(producer_id, socket).await?;
        Ok(())
    }

    async fn handle_consumer(&self, socket: TcpStream) -> Result<()> {
        let consumer_id = uuid::Uuid::new_v4().to_string();
        self.network.add_consumer(consumer_id, socket).await?;
        Ok(())
    }

    async fn handle_fetch(&self, _message: Message) -> Result<()> {
        // TODO: Implement fetch handling
        Ok(())
    }

    async fn handle_partition_transfer(&self, _msg: NetworkMessage) -> Result<()> {
        // TODO: Implement partition transfer
        Ok(())
    }

    async fn handle_node_join(&self, _msg: NetworkMessage) -> Result<()> {
        // TODO: Implement node join
        Ok(())
    }

    async fn handle_node_leave(&self, _msg: NetworkMessage) -> Result<()> {
        // TODO: Implement node leave
        Ok(())
    }

    async fn handle_heartbeat(&self, _msg: NetworkMessage) -> Result<()> {
        // TODO: Implement heartbeat
        Ok(())
    }

    async fn handle_join_group(&self, _msg: NetworkMessage) -> Result<()> {
        // TODO: Implement join group
        Ok(())
    }

    async fn handle_metadata_request(&self, _msg: NetworkMessage) -> Result<()> {
        // TODO: Implement metadata request
        Ok(())
    }
}
