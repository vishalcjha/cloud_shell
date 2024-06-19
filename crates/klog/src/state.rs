use std::collections::HashMap;

use colored::{Color, ColoredString, Colorize};
use futures::{AsyncBufReadExt, TryStreamExt};
use k8s_openapi::api::core::v1::Pod;
use kube::{api::LogParams, runtime::reflector::Lookup, Api, Client};
use tokio::sync::mpsc::UnboundedSender;
const COLOR_SET: &'static [Color] = &[
    Color::Red,
    Color::Blue,
    Color::Green,
    Color::Cyan,
    Color::Yellow,
];
pub(crate) struct State {
    pod_log: HashMap<String, String>,
    pods: Api<Pod>,
    pub(crate) client: Client,
    tx: UnboundedSender<(ColoredString, String)>,
    color_idx: usize,
}

impl State {
    pub(crate) fn new(client: Client, tx: UnboundedSender<(ColoredString, String)>) -> Self {
        Self {
            pod_log: HashMap::new(),
            pods: Api::default_namespaced(client.clone()),
            client,
            tx,
            color_idx: 0,
        }
    }

    pub(crate) fn add_pod(&mut self, pod: Pod) {
        let tx = self.tx.clone();
        let color = &COLOR_SET[self.color_idx];
        self.color_idx = (self.color_idx + 1) % COLOR_SET.len();
        let client = self.client.clone();
        tokio::spawn(async move {
            let pods: Api<Pod> = Api::default_namespaced(client);
            let mut logs = pods
                .log_stream(
                    pod.name().unwrap().as_ref(),
                    &LogParams {
                        follow: true,
                        ..Default::default()
                    },
                )
                .await?
                .lines();

            while let Some(line) = logs.try_next().await? {
                tx.send((
                    State::get_colored_string(&format!("[pod : {}]", pod.name().unwrap(),), &color),
                    String::from_utf8_lossy(line.as_bytes()).to_string(),
                ))?;
            }

            Ok::<(), anyhow::Error>(())
        });
    }

    fn get_colored_string(line: &str, color: &Color) -> ColoredString {
        match color {
            Color::Red => line.red(),
            Color::Green => line.green(),
            Color::Yellow => line.yellow(),
            Color::Blue => line.blue(),
            Color::Magenta => line.magenta(),
            Color::Cyan => line.cyan(),
            _ => line.normal(),
        }
    }
}
