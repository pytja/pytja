use wasmtime::{Config, Engine, Instance, Linker, Module, Store};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use anyhow::{Result, anyhow};

#[derive(Clone)]
pub struct PluginManager {
    engine: Engine,
    // Der Thread-sichere Cache für vorkompilierte Maschinencode-Module
    module_cache: Arc<RwLock<HashMap<String, Module>>>,
}

impl PluginManager {
    /// Initialisiert die Wasmtime-Engine mit maximaler Optimierung
    pub fn new() -> Result<Self> {
        let mut config = Config::new();
        // Wir nutzen den Cranelift-Compiler auf der höchsten Optimierungsstufe
        config.cranelift_opt_level(wasmtime::OptLevel::SpeedAndSize);
        // Da wir Tokio's spawn_blocking nutzen, brauchen wir keinen internen Async-Overhead von Wasmtime
        config.async_support(false);

        let engine = Engine::new(&config)?;

        Ok(Self {
            engine,
            module_cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    // =========================================================================
    // --- 1. MODULE CACHING (Compile Once) ---
    // =========================================================================

    /// Lädt eine .wasm Datei, kompiliert sie zu nativem Maschinencode und
    /// speichert sie dauerhaft im RAM ab.
    pub async fn load_plugin(&self, plugin_name: &str, wasm_bytes: Vec<u8>) -> Result<()> {
        let engine_clone = self.engine.clone();

        // CPU-Offloading: Kompilieren blockiert den Thread massiv!
        let module = tokio::task::spawn_blocking(move || {
            Module::new(&engine_clone, &wasm_bytes)
        })
            .await
            .map_err(|e| anyhow!("Task Panicked during WASM compilation: {}", e))??;

        // Modul in den Cache schreiben (Write-Lock)
        let mut cache = self.module_cache.write().await;
        cache.insert(plugin_name.to_string(), module);

        Ok(())
    }

    // =========================================================================
    // --- 2. FAST INSTANTIATION (Run Many) ---
    // =========================================================================

    /// Klont eine leichtgewichtige Instanz aus dem vorgehaltenen Modul und
    /// führt die Datenverarbeitung aus. Dauer: Nanosekunden.
    pub async fn execute_plugin(&self, plugin_name: &str, input_data: Vec<u8>) -> Result<Vec<u8>> {
        // 1. Modul aus dem Cache lesen (Nur Read-Lock, blockiert keine anderen parallelen Aufrufe)
        let module = {
            let cache = self.module_cache.read().await;
            cache.get(plugin_name)
                .cloned() // Module in Wasmtime sind intern Referenz-gezählt (Arc), Klonen ist also extrem billig
                .ok_or_else(|| anyhow!("Plugin '{}' not found in cache", plugin_name))?
        };

        let engine = self.engine.clone();

        // 2. CPU-Offloading: Die eigentliche Datenverarbeitung der KI oder des Skripts
        tokio::task::spawn_blocking(move || {
            // Ein "Store" isoliert den Sandbox-Speicher für genau diese eine Ausführung
            let mut store = Store::new(&engine, ());
            let linker = Linker::new(&engine);

            // Instanziierung aus dem bereits kompilierten Modul (Zero-Overhead)
            let instance = linker.instantiate(&mut store, &module)
                .map_err(|e| anyhow!("WASM Instantiation failed: {}", e))?;

            // --- Abstrahierte WASM Ausführung ---
            // In der Realität würdest du hier Speicher in die WASM-Sandbox kopieren,
            // die Export-Funktion (z.B. "process_data") aufrufen und das Ergebnis auslesen.
            // ...

            Ok(input_data) // Platzhalter-Rückgabe
        })
            .await
            .map_err(|e| anyhow!("Task Panicked during WASM execution: {}", e))?
    }
}