use anyhow::{Ok, Result};
use facade::component::anything_item::Something;
use smol::channel::{Receiver, Sender};
use tracing::info;
use vaultify::Vaultify;

fn main() -> Result<()> {
    logger::init_log();
    Vaultify::init_vault();
    let (request_sender, request_reciver, data_sender, data_reciver) = init_channel();
    indexify::init_service(request_reciver, data_sender)?;
    sentrify::init_service();
    facade::setup(request_sender, data_reciver);

    Ok(())
}

fn init_channel() -> (
    Sender<String>,
    Receiver<String>,
    Sender<Vec<Something>>,
    Receiver<Vec<Something>>,
) {
    let (request_sender, request_reciver) = smol::channel::unbounded::<String>();
    let (data_sender, data_reciver) = smol::channel::unbounded::<Vec<Something>>();
    info!("channel initialized");
    (request_sender, request_reciver, data_sender, data_reciver)
}
