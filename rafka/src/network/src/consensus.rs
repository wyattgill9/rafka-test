use std::sync::Arc;
use tokio::sync::{RwLock, mpsc, broadcast};
use dashmap::DashMap;
use rafka_core::{Result, Error};
use std::time::{Duration, SystemTime};

pub struct RaftConsensus {
    node_id: String,
    current_term: Arc<RwLock<u64>>,
    voted_for: Arc<RwLock<Option<String>>>,
    log: Arc<RwLock<RaftLog>>,
    commit_index: Arc<RwLock<u64>>,
    last_applied: Arc<RwLock<u64>>,
    state: Arc<RwLock<RaftState>>,
    peers: Arc<DashMap<String, RaftPeer>>,
}

#[derive(Debug, Clone)]
struct RaftLog {
    entries: Vec<LogEntry>,
    last_index: u64,
    last_term: u64,
}

#[derive(Debug, Clone)]
struct LogEntry {
    term: u64,
    index: u64,
    command: ConsensusCommand,
}

#[derive(Debug, Clone)]
enum ConsensusCommand {
    PartitionAssignment { partition: u32, node: String },
    TopicCreation { topic: String, partitions: u32 },
    MembershipChange { node: String, change_type: MembershipChangeType },
}

#[derive(Debug, Clone)]
enum RaftState {
    Follower,
    Candidate,
    Leader {
        next_index: DashMap<String, u64>,
        match_index: DashMap<String, u64>,
    },
}

impl RaftConsensus {
    pub async fn new(node_id: String) -> Self {
        Self {
            node_id,
            current_term: Arc::new(RwLock::new(0)),
            voted_for: Arc::new(RwLock::new(None)),
            log: Arc::new(RwLock::new(RaftLog {
                entries: Vec::new(),
                last_index: 0,
                last_term: 0,
            })),
            commit_index: Arc::new(RwLock::new(0)),
            last_applied: Arc::new(RwLock::new(0)),
            state: Arc::new(RwLock::new(RaftState::Follower)),
            peers: Arc::new(DashMap::new()),
        }
    }

    pub async fn propose(&self, command: ConsensusCommand) -> Result<()> {
        let state = self.state.read().await;
        match *state {
            RaftState::Leader { .. } => {
                self.append_entry(command).await?;
                self.replicate_logs().await?;
                Ok(())
            }
            _ => Err(Error::NotLeader("Not the leader".into())),
        }
    }

    async fn append_entry(&self, command: ConsensusCommand) -> Result<()> {
        let mut log = self.log.write().await;
        let term = *self.current_term.read().await;
        
        let entry = LogEntry {
            term,
            index: log.last_index + 1,
            command,
        };
        
        log.entries.push(entry);
        log.last_index += 1;
        log.last_term = term;
        
        Ok(())
    }

    async fn replicate_logs(&self) -> Result<()> {
        let state = self.state.read().await;
        match &*state {
            RaftState::Leader { next_index, match_index } => {
                let mut tasks = Vec::new();
                
                for peer in self.peers.iter() {
                    let peer_id = peer.key().clone();
                    let next_idx = next_index.get(&peer_id).map(|v| *v).unwrap_or(0);
                    
                    tasks.push(self.send_append_entries(&peer_id, next_idx));
                }

                let results = futures::future::join_all(tasks).await;
                self.update_commit_index(results).await?;
                
                Ok(())
            }
            _ => Err(Error::NotLeader("Not the leader".into())),
        }
    }

    async fn update_commit_index(&self, results: Vec<Result<u64>>) -> Result<()> {
        let mut commit_index = self.commit_index.write().await;
        let term = *self.current_term.read().await;
        let log = self.log.read().await;

        // Find the highest index that has been replicated to a majority
        let mut indices: Vec<u64> = results
            .into_iter()
            .filter_map(|r| r.ok())
            .collect();
        indices.sort_unstable();

        if let Some(majority_index) = indices.get(indices.len() / 2) {
            // Only commit if the entry is from current term
            if let Some(entry) = log.entries.get(*majority_index as usize) {
                if entry.term == term {
                    *commit_index = *majority_index;
                }
            }
        }

        Ok(())
    }

    async fn apply_committed_entries(&self) -> Result<()> {
        let commit_index = *self.commit_index.read().await;
        let mut last_applied = self.last_applied.write().await;
        let log = self.log.read().await;

        while *last_applied < commit_index {
            *last_applied += 1;
            if let Some(entry) = log.entries.get(*last_applied as usize) {
                self.apply_command(&entry.command).await?;
            }
        }

        Ok(())
    }

    async fn apply_command(&self, command: &ConsensusCommand) -> Result<()> {
        match command {
            ConsensusCommand::PartitionAssignment { partition, node } => {
                // Update partition assignments
                Ok(())
            }
            ConsensusCommand::TopicCreation { topic, partitions } => {
                // Create topic and initialize partitions
                Ok(())
            }
            ConsensusCommand::MembershipChange { node, change_type } => {
                // Handle cluster membership changes
                Ok(())
            }
        }
    }
} 