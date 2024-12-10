use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tonic::{transport::Server, Request, Response, Status};
use futures::Stream;
use std::pin::Pin;
use tokio_stream::wrappers::BroadcastStream;
use rafka_core::proto::rafka::{
    broker_service_server::{BrokerService, BrokerServiceServer},
    RegisterRequest, RegisterResponse, SubscribeRequest, SubscribeResponse,
    PublishRequest, PublishResponse, ConsumeRequest, ConsumeResponse,
};
use chrono::Utc;
use prost_types;
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct Broker {
    topics: Arc<RwLock<HashMap<String, HashSet<String>>>>,
    messages: Arc<RwLock<broadcast::Sender<ConsumeResponse>>>,
    message_counter: AtomicUsize,
    broadcast_capacity: usize,
    partition_id: u32,
    total_partitions: u32,
}

impl Broker {
    pub fn new(partition_id: u32, total_partitions: u32) -> Self {
        const BROADCAST_CAPACITY: usize = 1024 * 16;
        let (tx, _) = broadcast::channel(BROADCAST_CAPACITY);
        
        Self {
            topics: Arc::new(RwLock::new(HashMap::new())),
            messages: Arc::new(RwLock::new(tx)),
            message_counter: AtomicUsize::new(0),
            broadcast_capacity: BROADCAST_CAPACITY,
            partition_id,
            total_partitions,
        }
    }

    async fn ensure_channel(&self) -> broadcast::Sender<ConsumeResponse> {
        let sender = self.messages.read().await;
        if sender.receiver_count() == 0 {
            drop(sender);
            let mut sender = self.messages.write().await;
            let (new_tx, _) = broadcast::channel(self.broadcast_capacity);
            *sender = new_tx.clone();
            new_tx
        } else {
            sender.clone()
        }
    }

    pub async fn shutdown(&self) {
        let mut sender = self.messages.write().await;
        let (new_tx, _) = broadcast::channel(self.broadcast_capacity);
        *sender = new_tx;
    }

    pub async fn serve(self, addr: &str) -> Result<(), Box<dyn std::error::Error>> {
        let addr = addr.parse()?;
        println!("Broker listening on {}", addr);

        Server::builder()
            .add_service(BrokerServiceServer::new(self))
            .serve(addr)
            .await?;

        Ok(())
    }

    fn owns_partition(&self, message_key: &str) -> bool {
        let hash = self.hash_key(message_key);
        hash % self.total_partitions == self.partition_id
    }

    fn hash_key(&self, key: &str) -> u32 {
        key.bytes().fold(0u32, |acc, b| acc.wrapping_add(b as u32))
    }
}

#[tonic::async_trait]
impl BrokerService for Broker {
    async fn publish(
        &self,
        request: Request<PublishRequest>,
    ) -> Result<Response<PublishResponse>, Status> {
        let req = request.into_inner();
        
        if !self.owns_partition(&req.producer_id) {
            return Err(Status::failed_precondition(format!(
                "Message belongs to partition {} not {}",
                self.hash_key(&req.producer_id) % self.total_partitions,
                self.partition_id
            )));
        }

        let msg_count = self.message_counter.fetch_add(1, Ordering::SeqCst);
        
        let response = ConsumeResponse {
            message_id: format!("msg-{}", msg_count),
            topic: req.topic.clone(),
            payload: req.payload,
            sent_at: Some(prost_types::Timestamp {
                seconds: Utc::now().timestamp(),
                nanos: 0,
            }),
        };

        let sender = self.ensure_channel().await;
        match sender.send(response.clone()) {
            Ok(_) => {
                println!("Published message to partition {}", self.partition_id);
                Ok(Response::new(PublishResponse {
                    message_id: response.message_id,
                    success: true,
                    message: format!("Published successfully to partition {}", self.partition_id),
                }))
            },
            Err(e) => {
                Err(Status::internal(format!("Failed to broadcast message: {}", e)))
            }
        }
    }

    async fn consume(
        &self,
        request: Request<ConsumeRequest>,
    ) -> Result<Response<Self::ConsumeStream>, Status> {
        let req = request.into_inner();
        println!("Consumer {} started consuming", req.consumer_id);
        
        let sender = self.ensure_channel().await;
        let rx = sender.subscribe();
        
        Ok(Response::new(MessageStream {
            inner: BroadcastStream::new(rx)
        }))
    }

    type ConsumeStream = MessageStream;

    async fn register(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<RegisterResponse>, Status> {
        let req = request.into_inner();
        println!("Client registered: {} ({:?})", req.client_id, req.client_type);
        
        Ok(Response::new(RegisterResponse {
            success: true,
            message: "Registered successfully".to_string(),
        }))
    }

    async fn subscribe(
        &self,
        request: Request<SubscribeRequest>,
    ) -> Result<Response<SubscribeResponse>, Status> {
        let req = request.into_inner();
        let mut topics = self.topics.write().await;
        
        topics
            .entry(req.topic.clone())
            .or_insert_with(HashSet::new)
            .insert(req.consumer_id.clone());

        println!("Consumer {} subscribed to topic {}", req.consumer_id, req.topic);
        
        Ok(Response::new(SubscribeResponse {
            success: true,
            message: "Subscribed successfully".to_string(),
        }))
    }
}

pub struct MessageStream {
    pub(crate) inner: BroadcastStream<ConsumeResponse>,
}

impl Stream for MessageStream {
    type Item = Result<ConsumeResponse, Status>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        use std::task::Poll;
        match Pin::new(&mut self.inner).poll_next(cx) {
            Poll::Ready(Some(Ok(msg))) => Poll::Ready(Some(Ok(msg))),
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(Status::internal(e.to_string())))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}
