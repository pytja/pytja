use std::fs;

fn main() {
    println!("🚀 [Plugin] Test-Plugin started in Pytja Sandbox!");

    // 1. Daten schreiben
    let data = "id,name,email\n1,Alice,alice@example.com\n2,Bob,bob@example.com";
    let data_path = "/scraped_data.csv";

    match fs::write(data_path, data) {
        Ok(_) => println!("✅ [Plugin] Wrote 2 records to {}", data_path),
        Err(e) => eprintln!("❌ [Plugin] Failed to write data: {}", e),
    }

    // 2. Metadaten (Sidecar) schreiben
    // Wichtig: Der Name MUSS exakt data_path + ".meta.json" sein!
    let metadata = r#"{
        "source": "test_plugin",
        "records": 2,
        "tags": ["test", "csv", "osint"]
    }"#;
    let meta_path = "/scraped_data.csv.meta.json";

    match fs::write(meta_path, metadata) {
        Ok(_) => println!("✅ [Plugin] Wrote metadata to {}", meta_path),
        Err(e) => eprintln!("❌ [Plugin] Failed to write metadata: {}", e),
    }

    println!("🎉 [Plugin] Execution finished! Waiting for Pytja Sync...");
}