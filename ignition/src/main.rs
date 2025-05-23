use crossbeam_channel::{Receiver, Sender, unbounded};
use indexify;
use tracing::info;
use vaultify::Vaultify;

use facade::component::anything_item::Something;

fn main() {
    logger::init_log();
    Vaultify::init_vault();
    let (request_sender, request_reciver, data_sender, data_reciver) = init_channel();
    indexify::init_service(request_reciver, data_sender);
    sentrify::init_service();
    facade::setup(request_sender, data_reciver);
}

fn init_channel() -> (
    Sender<String>,
    Receiver<String>,
    Sender<Vec<Something>>,
    Receiver<Vec<Something>>,
) {
    let (request_sender, request_reciver) = unbounded::<String>();
    let (data_sender, data_reciver) = unbounded::<Vec<Something>>();
    info!("channel initialized");
    (request_sender, request_reciver, data_sender, data_reciver)
}

#[cfg(test)]
mod tests {
    use vaultify::Vaultify;

    #[test]
    fn test_vaultify() {
        Vaultify::init_vault();
    }
}
