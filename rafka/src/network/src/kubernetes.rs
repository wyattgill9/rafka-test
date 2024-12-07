use k8s_openapi::api::core::v1::Pod;
use kube::{
    api::{Api, ListParams, Meta, PostParams},
    Client,
};
use rafka_core::{Result, Error};

pub struct KubernetesManager {
    client: Client,
    namespace: String,
    statefulset_name: String,
}

impl KubernetesManager {
    pub async fn new() -> Result<Self> {
        let client = Client::try_default()
            .await
            .map_err(|e| Error::Kubernetes(e.to_string()))?;

        Ok(Self {
            client,
            namespace: std::env::var("POD_NAMESPACE")
                .unwrap_or_else(|_| "default".into()),
            statefulset_name: std::env::var("STATEFULSET_NAME")
                .unwrap_or_else(|_| "rafka".into()),
        })
    }

    pub async fn discover_peers(&self) -> Result<Vec<String>> {
        let pods: Api<Pod> = Api::namespaced(self.client.clone(), &self.namespace);
        let lp = ListParams::default()
            .labels(&format!("app={}", self.statefulset_name));

        let pod_list = pods.list(&lp)
            .await
            .map_err(|e| Error::Kubernetes(e.to_string()))?;

        Ok(pod_list.iter()
            .filter_map(|pod| {
                let name = Meta::name(pod);
                let phase = pod.status.as_ref()?.phase.as_ref()?;
                if phase == "Running" {
                    Some(format!("{}.{}", name, self.statefulset_name))
                } else {
                    None
                }
            })
            .collect())
    }

    pub async fn update_readiness(&self, is_ready: bool) -> Result<()> {
        // Update K8s readiness probe endpoint
        Ok(())
    }

    pub async fn get_pod_ordinal(&self) -> Result<i32> {
        let hostname = std::env::var("HOSTNAME")
            .map_err(|_| Error::Kubernetes("Unable to get hostname".into()))?;
        
        let ordinal = hostname.split('-')
            .last()
            .and_then(|s| s.parse::<i32>().ok())
            .ok_or_else(|| Error::Kubernetes("Invalid pod name format".into()))?;

        Ok(ordinal)
    }
} 