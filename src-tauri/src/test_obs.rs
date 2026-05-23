use obws::Client;
use obws::events::Event;
use tokio::sync::broadcast;

fn test() {
    let (tx, _) = broadcast::channel::<Event>(10);
}
