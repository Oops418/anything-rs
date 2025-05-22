use tracing::{Level, info};
use tracing_subscriber::{filter::Targets, fmt, layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_log() {
    let scope = Level::DEBUG;
    let filter: Targets = Targets::new()
        .with_targets([
            ("facade", scope),
            ("indexify", scope),
            ("logger", scope),
            ("sentrify", scope),
            ("vaultify", scope),
            ("ignition", scope),
        ])
        .with_default(Level::WARN);

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer())
        .init();

    info!("Logger initialized");
}
