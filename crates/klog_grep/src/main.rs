#![allow(dead_code)]
#![allow(unused_variables)]

use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
};

use anyhow::{anyhow, Result};
use clap::{Arg, Command};
use futures::{io::AsyncBufRead, TryStreamExt};
use k8s_openapi::api::core::v1::Pod;
use kube::{api::LogParams, runtime::watcher, Api, Client};
use tokio::{spawn, sync::Mutex};
const POD_LABEL_FLAG: &'static str = "pod-label";
type LogResult = (String, String);
static APP_STATE: OnceLock<Arc<Mutex<State>>> = OnceLock::new();

struct State {
    pod_log: HashMap<String, String>,
    pods: Api<Pod>,
    client: Client,
}

impl State {
    fn new(client: Client) -> Self {
        Self {
            pod_log: HashMap::new(),
            pods: Api::default_namespaced(client.clone()),
            client,
        }
    }

    fn add_pod(&mut self, pod: Pod) {}

    fn remove_pod(&mut self, pod: Pod) {}
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().init();
    let matches = Command::new("kgrep")
        .about("grep in multiple pods")
        .author("Vishal Kumar, vishalcjha@gmail.com")
        .arg(
            Arg::new(POD_LABEL_FLAG)
                .value_name(POD_LABEL_FLAG)
                .help("Pod label identifier(s)")
                .required(true)
                .num_args(1..),
        )
        .get_matches();
    let pod_labels: Vec<String> = matches
        .get_many(POD_LABEL_FLAG)
        .expect("must provide pod to match against")
        .cloned()
        .collect();
    let client = Client::try_default().await?;
    let client_clone = client.clone();
    APP_STATE.get_or_init(|| Arc::new(Mutex::new(State::new(client_clone))));
    let _ = spawn(async move {
        let state = APP_STATE.get().unwrap().clone();
        let _ = update_pods_names(state, pod_labels).await;
    })
    .await;
    Ok(())
}

async fn update_pods_names(state: Arc<Mutex<State>>, pod_selector: Vec<String>) -> Result<()> {
    let state = state.lock().await;
    let pod_selector = Arc::new(pod_selector);
    let pods: Api<Pod> = Api::all(state.client.clone());
    let watcher_config = watcher::Config::default();
    let pod_watcher = watcher(pods, watcher_config);
    pod_watcher
        .try_for_each(|event| async {
            match event {
                watcher::Event::Applied(pod) => {
                    if is_supervised_pod(&pod, &pod_selector.clone()) {
                        tracing::info!("Pod added {:?}", &pod);
                    }
                }
                watcher::Event::Deleted(pod) => {
                    if is_supervised_pod(&pod, &pod_selector.clone()) {
                        tracing::info!("Pod deleted {:?}", pod.metadata.name);
                    }
                }
                watcher::Event::Restarted(pods) => {
                    for pod in pods {
                        if is_supervised_pod(&pod, &pod_selector.clone()) {
                            tracing::info!("Pod updated {:?}", pod.metadata.name);
                        }
                    }
                }
            };
            Ok(())
        })
        .await?;
    Ok(())
}

fn is_supervised_pod(pod: &Pod, supervised_pod_labels: &[String]) -> bool {
    let Some(pod_label) = pod.metadata.labels.as_ref().and_then(|it| it.get("app")) else {
        return false;
    };
    supervised_pod_labels
        .iter()
        .any(|it| pod_label.contains(it))
}

async fn get_logs(pod_name: String, client: Client) -> Result<impl AsyncBufRead> {
    let pods: Api<Pod> = Api::default_namespaced(client);
    pods.log_stream(&pod_name, &LogParams::default())
        .await
        .map_err(|err| anyhow!(format!("Error getting logs {:?}", err)))
}
