#![allow(dead_code)]
#![allow(unused_variables)]
mod logger;
mod state;
use std::sync::{Arc, OnceLock};

use anyhow::Result;
use clap::{Arg, Command};
use futures::TryStreamExt;
use k8s_openapi::api::core::v1::Pod;
use kube::{runtime::watcher, Api, Client};
use logger::Logger;
use state::State;
use tokio::{
    spawn,
    sync::{mpsc::unbounded_channel, Mutex},
};

const POD_LABEL_FLAG: &'static str = "pod-label";
type LogResult = (String, String);
static APP_STATE: OnceLock<Arc<Mutex<State>>> = OnceLock::new();

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
    let (tx, rx) = unbounded_channel();
    let state = State::new(client.clone(), tx);
    APP_STATE.get_or_init(|| Arc::new(Mutex::new(state)));

    spawn(async move {
        let mut logger = Logger::new(rx);
        logger.start().await;
    });

    let _ = spawn(async move {
        let state = APP_STATE.get().unwrap().clone();
        let _ = update_pods_names(state, pod_labels).await;
    })
    .await;
    Ok(())
}

async fn update_pods_names(state: Arc<Mutex<State>>, pod_selector: Vec<String>) -> Result<()> {
    let lock_state = state.lock().await;
    let pod_selector = Arc::new(pod_selector);
    let pods: Api<Pod> = Api::all(lock_state.client.clone());
    drop(lock_state);
    let watcher_config = watcher::Config::default();
    let pod_watcher = watcher(pods, watcher_config);
    pod_watcher
        .try_for_each(|event| async {
            match event {
                watcher::Event::Applied(pod) => {
                    if is_supervised_pod(&pod, &pod_selector.clone()) {
                        let mut state = state.lock().await;
                        tracing::info!("Pod added {:?}", pod.metadata.name);
                        state.add_pod(pod);
                    }
                }
                watcher::Event::Deleted(pod) => {
                    if is_supervised_pod(&pod, &pod_selector.clone()) {
                        tracing::info!("Pod deleted {:?}", pod.metadata.name);
                    }
                }
                watcher::Event::Restarted(pods) => {
                    let mut state = state.lock().await;
                    for pod in pods {
                        if is_supervised_pod(&pod, &pod_selector.clone()) {
                            tracing::info!("Pod updated {:?}", pod.metadata.name);
                            state.add_pod(pod);
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
