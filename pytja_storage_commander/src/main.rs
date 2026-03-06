use std::thread;
use std::time::Duration;

const UI_HTML: &str = include_str!("ui.html");

fn main() {
    if let Err(e) = pytja_sdk::window::create("Storage Commander", UI_HTML, 1000.0, 700.0) {
        let _ = pytja_sdk::host::alarm(&format!("Failed to spawn window: {}", e));
        return;
    }

    let mut connections = 142;

    loop {
        pytja_sdk::host::ping();

        // Simulierte Telemetriedaten generieren (später Server-Fetch)
        let stats = serde_json::json!({
            "type": "stats_update",
            "storage": "1.24 TB",
            "connections": connections,
            "health": "Online"
        });

        let _ = pytja_sdk::window::emit(stats);

        connections += 5; // Simuliere Netzwerkverkehr
        thread::sleep(Duration::from_secs(2));
    }
}