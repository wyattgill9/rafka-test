use crate::core::{Message, Result};
use tokio::net::{TcpListener, TcpStream};
use std::sync::Arc;

pub struct P2PNetwork {
    port: u16,
    peers: Arc<DashMap<String, PeerConnection>>,
}

impl P2PNetwork {
    pub async fn start(&self) -> Result<()> {
        let listener = TcpListener::bind(("0.0.0.0", self.port)).await?;
        
        loop {
            let (socket, addr) = listener.accept().await?;
            let peer_conn = PeerConnection::new(socket);
            self.peers.insert(addr.to_string(), peer_conn);
            
            // Spawn handler for peer messages
            tokio::spawn(async move {
                self.handle_peer_messages(addr.to_string()).await
            });
        }
    }

    async fn handle_peer_messages(&self, peer_id: String) -> Result<()> {
        // zero-copy message handling using RDMA when available
        if let Some(rdma_connection) = self.setup_rdma() {
            rdma_connection.process_messages().await?;
        } else {
            self.process_tcp_messages(peer_id).await?;
        }
        Ok(())
    }
}
