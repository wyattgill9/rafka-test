use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use rafka_core::{Result, Error};
use std::time::{Duration, SystemTime};

pub struct LeaderElection {
    node_id: String,
    term: Arc<RwLock<u64>>,
    leader_id: Arc<RwLock<Option<String>>>,
    vote_manager: Arc<VoteManager>,
    state: Arc<RwLock<ElectionState>>,
    state_change_tx: broadcast::Sender<LeaderStateChange>,
}

#[derive(Debug, Clone)]
enum ElectionState {
    Follower,
    Candidate,
    Leader,
}

#[derive(Debug, Clone)]
struct VoteRequest {
    candidate_id: String,
    term: u64,
    last_log_index: u64,
    last_log_term: u64,
}

#[derive(Debug, Clone)]
struct VoteResponse {
    voter_id: String,
    term: u64,
    granted: bool,
}

impl LeaderElection {
    pub fn new(node_id: String) -> Self {
        let (tx, _) = broadcast::channel(1000);
        Self {
            node_id,
            term: Arc::new(RwLock::new(0)),
            leader_id: Arc::new(RwLock::new(None)),
            vote_manager: Arc::new(VoteManager::new()),
            state: Arc::new(RwLock::new(ElectionState::Follower)),
            state_change_tx: tx,
        }
    }

    pub async fn start(&self) -> Result<()> {
        // Start election timeout monitor
        self.monitor_election_timeout().await?;
        Ok(())
    }

    pub async fn handle_vote_request(&self, request: VoteRequest) -> Result<VoteResponse> {
        let mut term = self.term.write().await;
        let state = self.state.read().await;

        // If request term is older, reject
        if request.term < *term {
            return Ok(VoteResponse {
                voter_id: self.node_id.clone(),
                term: *term,
                granted: false,
            });
        }

        // If newer term, step down if leader
        if request.term > *term {
            *term = request.term;
            if matches!(*state, ElectionState::Leader) {
                self.step_down().await?;
            }
        }

        // Check if vote can be granted
        if self.vote_manager.can_vote_for(&request).await {
            self.vote_manager.record_vote(request.candidate_id.clone()).await?;
            Ok(VoteResponse {
                voter_id: self.node_id.clone(),
                term: *term,
                granted: true,
            })
        } else {
            Ok(VoteResponse {
                voter_id: self.node_id.clone(),
                term: *term,
                granted: false,
            })
        }
    }

    async fn start_election(&self) -> Result<()> {
        let mut state = self.state.write().await;
        let mut term = self.term.write().await;
        
        *state = ElectionState::Candidate;
        *term += 1;
        
        let request = VoteRequest {
            candidate_id: self.node_id.clone(),
            term: *term,
            last_log_index: self.vote_manager.get_last_log_index().await,
            last_log_term: self.vote_manager.get_last_log_term().await,
        };

        // Request votes from all nodes
        let votes = self.request_votes(request).await?;
        
        // If majority achieved, become leader
        if self.has_majority(votes) {
            self.become_leader().await?;
        }

        Ok(())
    }

    async fn become_leader(&self) -> Result<()> {
        let mut state = self.state.write().await;
        *state = ElectionState::Leader;
        
        let mut leader = self.leader_id.write().await;
        *leader = Some(self.node_id.clone());

        // Broadcast leader change
        self.broadcast_leader_change().await?;

        // Start sending heartbeats
        self.start_heartbeats().await?;

        Ok(())
    }

    async fn step_down(&self) -> Result<()> {
        let mut state = self.state.write().await;
        *state = ElectionState::Follower;
        
        let mut leader = self.leader_id.write().await;
        *leader = None;

        Ok(())
    }

    async fn monitor_election_timeout(&self) -> Result<()> {
        let election = self.clone();
        tokio::spawn(async move {
            loop {
                let timeout = rand::random::<u64>() % 150 + 150; // 150-300ms
                tokio::time::sleep(Duration::from_millis(timeout)).await;

                let state = election.state.read().await;
                if matches!(*state, ElectionState::Follower) {
                    election.start_election().await?;
                }
            }
            #[allow(unreachable_code)]
            Ok::<(), Error>(())
        });
        Ok(())
    }
} 