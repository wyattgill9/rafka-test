use rafka_core::{Message, NetworkMessage, MessageType, Result, Config};
use tokio::net::TcpStream;
use tokio::sync::broadcast;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::collections::HashMap;

pub struct Consumer {
    node_id: String,
    stream: TcpStream,
    assigned_partitions: HashMap<u32, PartitionConsumer>,
    group_id: Option<String>,
}

struct PartitionConsumer {
    partition: u32,
    current_offset: u64,
    receiver: broadcast::Receiver<Message>,
}

impl Consumer {
    pub async fn new(config: Config) -> Result<Self> {
        let stream = TcpStream::connect(format!("127.0.0.1:{}", config.broker_port)).await?;
        Ok(Self {
            node_id: uuid::Uuid::new_v4().to_string(),
            stream,
            assigned_partitions: HashMap::new(),
            group_id: None,
        })
    }

    pub async fn join_group(&mut self, group_id: String) -> Result<()> {
        self.group_id = Some(group_id.clone());
        
        // Send join group request
        let join_msg = NetworkMessage {
            msg_type: MessageType::JoinGroup,
            source_node: self.node_id.clone(),
            payload: Message::new(
                "system".to_string(),
                None::<Vec<u8>>,
                group_id.into_bytes()
            ),
            ..Default::default()
        };

        self.send_network_message(join_msg).await?;
        
        // Wait for partition assignment
        self.handle_partition_assignment().await
    }

    pub async fn poll_messages(&mut self) -> Result<Vec<Message>> {
        let mut messages = Vec::new();
        
        for consumer in self.assigned_partitions.values_mut() {
            while let Ok(msg) = consumer.receiver.try_recv() {
                messages.push(msg);
                consumer.current_offset += 1;
            }
        }

        Ok(messages)
    }

    async fn handle_partition_assignment(&mut self) -> Result<()> {
        // Read assignment from broker
        let assignment = self.read_assignment().await?;
        
        // Set up consumers for assigned partitions
        for (partition, offset) in assignment {
            let (_, receiver) = broadcast::channel(1000);
            self.assigned_partitions.insert(partition, PartitionConsumer {
                partition,
                current_offset: offset,
                receiver,
            });
        }

        Ok(())
    }

    async fn send_network_message(&mut self, msg: NetworkMessage) -> Result<()> {
        let msg_bytes = bincode::serialize(&msg)?;
        let msg_len = msg_bytes.len() as u32;
        
        self.stream.write_all(&msg_len.to_be_bytes()).await?;
        self.stream.write_all(&msg_bytes).await?;
        
        Ok(())
    }

    async fn read_assignment(&mut self) -> Result<HashMap<u32, u64>> {
        let mut len_buf = [0u8; 4];
        self.stream.read_exact(&mut len_buf).await?;
        let len = u32::from_be_bytes(len_buf);
        
        let mut buf = vec![0u8; len as usize];
        self.stream.read_exact(&mut buf).await?;
        
        bincode::deserialize(&buf).map_err(|e| e.into())
    }
}
