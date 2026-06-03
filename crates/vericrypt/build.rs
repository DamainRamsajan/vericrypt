use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let axiom_dir = PathBuf::from("src/compliance/axioms_compiled");

    println!("cargo:rerun-if-changed=src/compliance/axioms_compiled/");

    let mut embed_code = String::from("use std::collections::HashMap;");
    embed_code.push_str("pub fn get_embedded_bytecode() -> HashMap<String, Vec<u8>> {");
    embed_code.push_str("let mut map = HashMap::new();");

    if let Ok(entries) = fs::read_dir(&axiom_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "aslb") {
                let framework = path.file_stem().unwrap().to_string_lossy().to_uppercase();
                let path_str = path.to_string_lossy().to_string();
                embed_code.push_str(&format!(
                    "map.insert(\"{}\".to_string(), include_bytes!(\"{}\").to_vec());",
                    framework, path_str
                ));
            }
        }
    }

    embed_code.push_str("map");
    embed_code.push('}');

    let embed_path = out_dir.join("embedded_axioms.rs");
    fs::write(&embed_path, embed_code.as_bytes()).unwrap_or_else(|e| {
        panic!("Failed to write embedded axioms source: {}", e);
    });
    println!("cargo:warning=Embedded axiom frameworks");
}
