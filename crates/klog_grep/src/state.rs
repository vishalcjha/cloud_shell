use std::collections::HashMap;

use futures::{AsyncBufReadExt, TryStreamExt};
use k8s_openapi::api::core::v1::Pod;
use kube::{api::LogParams, runtime::reflector::Lookup, Api, Client};
use tokio::sync::mpsc::UnboundedSender;

pub(crate) struct State {
    pod_log: HashMap<String, String>,
    pods: Api<Pod>,
    pub(crate) client: Client,
    tx: UnboundedSender<String>,
}

impl State {
    pub(crate) fn new(client: Client, tx: UnboundedSender<String>) -> Self {
        Self {
            pod_log: HashMap::new(),
            pods: Api::default_namespaced(client.clone()),
            client,
            tx,
        }
    }

    pub(crate) fn add_pod(&mut self, pod: Pod) {
        let tx = self.tx.clone();
        let client = self.client.clone();
        tokio::spawn(async move {
            let pods: Api<Pod> = Api::default_namespaced(client);
            let mut logs = pods
                .log_stream(pod.name().unwrap().as_ref(), &LogParams::default())
                .await?
                .lines();

            while let Some(line) = logs.try_next().await? {
                tx.send(line)?;
            }

            Ok::<(), anyhow::Error>(())
        });
    }
}
