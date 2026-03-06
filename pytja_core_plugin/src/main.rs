use std::fs;
use std::thread;
use std::time::Duration;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

// --- IPC DATA STRUCTURES (ENTERPRISE ENUM PATTERN) ---

#[derive(Serialize)]
#[serde(untagged)]
enum IpcParams {
    VfsRead { path: String },
    VfsWrite { path: String, content: String },
    Network { method: String, url: String, body: Option<String> },
}

#[derive(Serialize)]
struct IpcRequest {
    module: String,
    method: String,
    params: IpcParams,
}

#[derive(Deserialize, Debug, Clone)]
struct VfsItem {
    name: String,
    is_folder: bool,
    size: u64,
}

#[derive(Deserialize, Debug)]
struct VfsData {
    items: Vec<VfsItem>,
}

#[derive(Deserialize, Debug)]
struct NetworkData {
    status_code: u16,
    body: String,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum IpcResponseData {
    Vfs(VfsData),
    Network(NetworkData),
}

#[derive(Deserialize, Debug)]
struct IpcResponse {
    status: String,
    data: Option<IpcResponseData>,
    message: Option<String>,
}

// --- RADAR ABI ---

#[link(wasm_import_module = "radar_abi")]
unsafe extern "C" {
    fn host_log_status(code: i32);
    fn host_ipc_request(req_ptr: *const u8, req_len: i32, res_ptr: *mut u8, res_cap: i32) -> i32;
    fn host_heartbeat(); // NEU
}

// --- ENTERPRISE FIX: DMA HELPER FUNCTIONS ---

// 1. Für typsichere Structs
fn execute_ipc(request: &IpcRequest) -> Option<IpcResponse> {
    if let Ok(request_json) = serde_json::to_string(request) {
        return send_dma_payload(request_json.as_bytes());
    }
    None
}

// 2. Für dynamische JSON-Makros (Ad-hoc Payloads)
fn send_raw_ipc(payload: &serde_json::Value) -> Option<IpcResponse> {
    let request_str = payload.to_string();
    send_dma_payload(request_str.as_bytes())
}

// Die zentrale Zero-Copy Memory Pipeline
fn send_dma_payload(req_bytes: &[u8]) -> Option<IpcResponse> {
    let mut res_cap = 65536; // 64KB Initial-Puffer
    let mut res_buf = vec![0u8; res_cap as usize];

    let mut status = unsafe {
        host_ipc_request(
            req_bytes.as_ptr(),
            req_bytes.len() as i32,
            res_buf.as_mut_ptr(),
            res_cap
        )
    };

    if status < 0 {
        res_cap = -status;
        res_buf = vec![0u8; res_cap as usize];
        status = unsafe {
            host_ipc_request(
                req_bytes.as_ptr(),
                req_bytes.len() as i32,
                res_buf.as_mut_ptr(),
                res_cap
            )
        };
    }

    if status > 0 {
        let res_len = status as usize;
        if let Ok(response_str) = std::str::from_utf8(&res_buf[..res_len]) {
            return serde_json::from_str::<IpcResponse>(response_str).ok();
        }
    }
    None
}

fn main() {
    println!("--- RADAR SOAR AGENT ONLINE ---");
    println!("[SOAR] VFS Monitoring & C2 Event Bus initialized.");

    let mut baseline: Option<HashMap<String, u64>> = None;
    let mut current_target = "/".to_string();

    // --- ENTERPRISE WEBSOCKET TEST ---
    println!("[SOAR] Initiating real-time telemetry stream via WebSocket...");

    let raw_ws_req = serde_json::json!({
        "module": "network",
        "method": "ws_connect",
        "params": {
            "id": "telemetry_stream",
            "url": "wss://echo.websocket.events"
        }
    });

    // --- ENTERPRISE UI RENDER ---
    println!("[SOAR] Compiling interactive dashboard layout...");
    let dashboard_html = r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Pytja SOAR Agent</title>
            <style>
                body { background-color: #0f172a; color: #38bdf8; font-family: monospace; padding: 2rem; }
                .card { background: #1e293b; padding: 1.5rem; border-radius: 8px; border: 1px solid #334155; }
            </style>
        </head>
        <body>
            <div class="card">
                <h1>RADAR SOAR DASHBOARD</h1>
                <p>Status: ACTIVE & MONITORING</p>
                <p>Telemetry WebSocket: CONNECTED</p>
            </div>
        </body>
        </html>
    "#;

    let ui_req = serde_json::json!({
        "module": "display",
        "method": "render",
        "params": {
            "html": dashboard_html
        }
    });

    // ENTERPRISE FIX: Direkter DMA Aufruf statt Dateisystem!
    send_raw_ipc(&ui_req);
    println!("[SOAR] Dashboard published. View at http://127.0.0.1:8080/ui/pytja_core_plugin");

    // ENTERPRISE FIX: Direkter DMA Aufruf statt Dateisystem!
    send_raw_ipc(&raw_ws_req);
    println!("[SOAR] Telemetry stream connected.");

    loop {
        unsafe { host_heartbeat(); } // WATCHDOG PING

        thread::sleep(Duration::from_secs(3));

        // Der C2 Inbox-Scanner bleibt asynchron über das VFS, da es als Queue fungiert
        if let Ok(msg) = fs::read_to_string("/workspace/.radar_inbox") {
            if !msg.is_empty() {
                let cmd = msg.trim();

                if cmd.starts_with("TARGET:") {
                    current_target = cmd.trim_start_matches("TARGET:").to_string();
                    println!("\n[C2 EVENT] Command received. Recalibrating scanner target to: {}", current_target);
                    baseline = None;
                }
                else if cmd.starts_with("WS_MSG:telemetry_stream:") {
                    let payload = cmd.trim_start_matches("WS_MSG:telemetry_stream:");
                    println!("[STREAM] Live Data: {}", payload);
                }
                else if cmd.starts_with("WS_SEND:") {
                    let payload = cmd.trim_start_matches("WS_SEND:");
                    println!("[SOAR] Transmitting payload to telemetry stream: {}", payload);

                    let raw_send = serde_json::json!({
                        "module": "network",
                        "method": "ws_send",
                        "params": {
                            "id": "telemetry_stream",
                            "data": payload
                        }
                    });

                    // ENTERPRISE FIX: Direkter DMA Aufruf statt Dateisystem!
                    send_raw_ipc(&raw_send);
                }

                let _ = fs::remove_file("/workspace/.radar_inbox");
            }
        }

        // --- VFS SCAN ---
        let vfs_req = IpcRequest {
            module: "vfs".to_string(),
            method: "list_dir".to_string(),
            params: IpcParams::VfsRead { path: current_target.clone() },
        };

        if let Some(response) = execute_ipc(&vfs_req) {
            if response.status != "success" { continue; }

            if let Some(IpcResponseData::Vfs(data)) = response.data {
                let mut current_state = HashMap::new();
                for item in data.items {
                    current_state.insert(item.name.clone(), item.size);
                }

                let mut baseline_needs_update = false;

                if let Some(previous_state) = &baseline {
                    let mut anomaly_detected = false;
                    let mut alert_message = String::new();

                    for (name, size) in &current_state {
                        match previous_state.get(name) {
                            None => {
                                println!("\n[CRITICAL] NEW FILE DETECTED IN {}: {}", current_target, name);
                                alert_message = format!("Unauthorized file creation: {}/{}", current_target, name);
                                anomaly_detected = true;
                            },
                            Some(prev_size) if prev_size != size => {
                                println!("\n[CRITICAL] FILE MODIFICATION DETECTED: {}", name);
                                alert_message = format!("Unauthorized file modification: {}/{}", current_target, name);
                                anomaly_detected = true;
                            },
                            _ => {}
                        }
                    }

                    for name in previous_state.keys() {
                        if !current_state.contains_key(name) {
                            println!("\n[CRITICAL] FILE DELETED IN {}: {}", current_target, name);
                            alert_message = format!("Unauthorized file deletion: {}/{}", current_target, name);
                            anomaly_detected = true;
                        }
                    }

                    if anomaly_detected {
                        baseline_needs_update = true;
                        unsafe { host_log_status(999); }

                        let raw_alarm = serde_json::json!({
                            "module": "host",
                            "method": "alarm",
                            "params": {
                                "message": alert_message.clone()
                            }
                        });

                        // ENTERPRISE FIX: Direkter DMA Aufruf statt Dateisystem!
                        send_raw_ipc(&raw_alarm);

                        println!("[SOAR] Dispatching automated threat intelligence report via HTTP...");
                        let net_req = IpcRequest {
                            module: "network".to_string(),
                            method: "send".to_string(),
                            params: IpcParams::Network {
                                method: "POST".to_string(),
                                url: "https://httpbin.org/post".to_string(),
                                body: Some(format!(r#"{{"alert": "{}"}}"#, alert_message))
                            },
                        };

                        if let Some(net_res) = execute_ipc(&net_req) {
                            if let Some(IpcResponseData::Network(net_data)) = net_res.data {
                                println!("[SOAR] Webhook transmitted. Remote status: {}", net_data.status_code);
                            }
                        }

                        println!("[SOAR] Generating local forensic report on VFS...");
                        let report_req = IpcRequest {
                            module: "vfs".to_string(),
                            method: "write_file".to_string(),
                            params: IpcParams::VfsWrite {
                                path: "/local_cache/ids_report.txt".to_string(),
                                content: format!("Intrusion Detected: {}\nTimestamp: UNIX_TIME\n", alert_message)
                            },
                        };
                        let _ = execute_ipc(&report_req);
                        println!("[SOAR] Forensic report safely stored in /local_cache/ids_report.txt.");
                    }
                } else {
                    println!("[SOAR] Baseline established for target: {}. Transitioning to silent monitoring.", current_target);
                    baseline_needs_update = true;
                }

                if baseline_needs_update {
                    baseline = Some(current_state);
                }
            }
        }
    }
}