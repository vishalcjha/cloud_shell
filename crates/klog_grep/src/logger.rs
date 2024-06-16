use tokio::sync::mpsc::UnboundedReceiver;

pub(crate) struct Logger {
    rx: UnboundedReceiver<String>,
}
impl Logger {
    pub fn new(rx: UnboundedReceiver<String>) -> Self {
        Self { rx }
    }

    pub async fn start(&mut self) {
        while let Some(log) = self.rx.recv().await {
            tracing::info!(log);
        }
    }
}
