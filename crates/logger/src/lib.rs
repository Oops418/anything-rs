use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_logger() {
    let filter = EnvFilter::new("ignition=debug,indexify=debug,vaultify=debug,off");
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer())
        .init();
}
