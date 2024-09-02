pub mod codec;

use codec::{AdaControllerCodec, AdaControllerCommand, AdaControllerResponse};
use futures_util::{stream::StreamExt, SinkExt};
use log::{debug, error};
use tokio::sync::{watch, Mutex};
use tokio_serial::SerialPortBuilderExt;
use tokio_util::codec::Decoder;

const BAUD_RATE: u32 = 9600;

#[derive(Debug)]
pub struct AdaController {
    endpoint: String,
    pub tx: Mutex<Option<watch::Sender<AdaControllerCommand>>>,
    pub rx: Mutex<Option<watch::Receiver<Option<AdaControllerResponse>>>>,
}

impl AdaController {
    pub fn new(endpoint: &str) -> Self {
        Self {
            tx: Mutex::new(None),
            rx: Mutex::new(None),
            endpoint: endpoint.to_string(),
        }
    }
    pub async fn listen(&self) -> anyhow::Result<()> {
        let conn = tokio_serial::new(self.endpoint.clone(), BAUD_RATE).open_native_async()?;
        let stream = AdaControllerCodec.framed(conn);
        let (mut tx, mut rx) = stream.split();

        let (command_tx, mut command_rx) = watch::channel(AdaControllerCommand::Inhibit);
        *self.tx.lock().await = Some(command_tx);

        let (event_tx, event_rx) = watch::channel(None);
        *self.rx.lock().await = Some(event_rx);

        tokio::spawn(async move {
            loop {
                if let Some(response) = rx.next().await {
                    match response {
                        Ok(response) => {
                            debug!("Received '{response}' from controller channel. Sending to event channel...");
                            event_tx.send(Some(response)).unwrap();
                        }
                        Err(err) => {
                            error!("Error reading line: {err}")
                        }
                    }
                }
            }
        });

        tokio::spawn(async move {
            loop {
                let data = command_rx.borrow_and_update().clone();
                debug!("Received '{data}' from command channel. Sending to controller... ");
                tx.send(data).await.unwrap();
                if let Err(err) = command_rx.changed().await {
                    error!("Call to channel's `changed()` failed: {err}");
                    break;
                }
            }
        });

        Ok(())
    }

    pub async fn send_command(&self, command: AdaControllerCommand) {
        let tx = self.tx.lock().await.clone().unwrap();
        tx.send(command).unwrap();
    }
}
