use wry::application::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    dpi::LogicalSize,
};
use wry::webview::WebViewBuilder;
use serde::Deserialize;
use std::env;
use std::io::{self, BufRead};
use std::thread;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

#[derive(Deserialize)]
struct WindowConfig {
    plugin_id: String,
    title: String,
    html_b64: String,
    width: f64,
    height: f64,
}

#[derive(Debug)]
enum UserEvent {
    IncomingData(String),
    Shutdown, // ENTERPRISE FIX: Sicheres Herunterfahren
}

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        anyhow::bail!("Missing window configuration payload");
    }

    let config: WindowConfig = serde_json::from_str(&args[1])
        .map_err(|e| anyhow::anyhow!("Invalid payload: {}", e))?;

    let html_content = String::from_utf8(BASE64.decode(&config.html_b64)?)?;
    let window_title = format!("{} [Agent: {}]", config.title, config.plugin_id.to_uppercase());

    let event_loop = EventLoop::<UserEvent>::with_user_event();
    let proxy = event_loop.create_proxy();

    thread::spawn(move || {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            if let Ok(content) = line {
                let _ = proxy.send_event(UserEvent::IncomingData(content));
            }
        }
        // ENTERPRISE FIX: Wenn der Host stirbt oder "daemon kill" aufruft,
        // schließt sich die Pipe (EOF). Wir feuern das Shutdown-Event!
        let _ = proxy.send_event(UserEvent::Shutdown);
    });

    let window = WindowBuilder::new()
        .with_title(window_title)
        .with_inner_size(LogicalSize::new(config.width, config.height))
        .build(&event_loop)?;

    let plugin_id_clone = config.plugin_id.clone();

    let webview = WebViewBuilder::new(window)?
        .with_html(&html_content)?
        .with_ipc_handler(move |_, string_payload| {
            let event_json = serde_json::json!({
                "source_plugin": plugin_id_clone,
                "event_data": string_payload
            });
            println!("PYTJA_IPC_EVENT:{}", event_json);
        })
        .build()?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::UserEvent(UserEvent::IncomingData(js_payload)) => {
                let script = format!("window.dispatchEvent(new CustomEvent('pytja_host_event', {{ detail: {} }}));", js_payload);
                let _ = webview.evaluate_script(&script);
            }
            // ENTERPRISE FIX: Das Fenster sauber aus dem RAM entfernen
            Event::UserEvent(UserEvent::Shutdown) => {
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                *control_flow = ControlFlow::Exit;
                println!("PYTJA_IPC_EVENT:{{\"source_plugin\": \"{}\", \"event_data\": \"WINDOW_CLOSED\"}}", config.plugin_id);
            }
            _ => {}
        }
    });
}