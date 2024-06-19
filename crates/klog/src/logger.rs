use colored::ColoredString;
use tokio::sync::mpsc::UnboundedReceiver;

pub(crate) struct Logger {
    rx: UnboundedReceiver<(ColoredString, String)>,
}
impl Logger {
    pub fn new(rx: UnboundedReceiver<(ColoredString, String)>) -> Self {
        Self { rx }
    }

    pub async fn start(&mut self) {
        while let Some(log) = self.rx.recv().await {
            println!("{} {}", log.0, log.1);
        }
    }
}
