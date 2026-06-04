use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let embed_path = out_dir.join("embedded_axioms.rs");
    
    // Only regenerate if the embedded file doesn't exist
    if embed_path.exists() {
        println!("cargo:warning=Axioms already compiled, skipping");
        return;
    }
    
    let axiom_dir = PathBuf::from("src/compliance/axioms");
    println!("cargo:rerun-if-changed=src/compliance/axioms/");

    let mut embed_code = String::from("use std::collections::HashMap;");
    embed_code.push_str("pub fn get_embedded_bytecode() -> HashMap<String, Vec<u8>> {");
    embed_code.push_str("let mut map = HashMap::new();");

    if let Ok(entries) = fs::read_dir(&axiom_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "asl") {
                let framework = path.file_stem().unwrap().to_string_lossy().to_uppercase();
                let source = fs::read_to_string(&path).unwrap_or_else(|e| {
                    panic!("Failed to read axiom {:?}: {}", path, e);
                });
                println!("cargo:warning=Compiling {} axioms...", framework);
                match seedc::compile(&source) {
                    Ok(bytecode) => {
                        embed_code.push_str(&format!(
                            "map.insert(\"{}\".to_string(), vec!{:?});", framework, bytecode
                        ));
                        println!("cargo:warning=Compiled {} ({} bytes)", framework, bytecode.len());
                    }
                    Err(e) => panic!("Failed to compile {}: {:?}", framework, e),
                }
            }
        }
    }

    embed_code.push_str("map");
    embed_code.push('}');
    fs::write(&embed_path, embed_code.as_bytes()).unwrap_or_else(|e| {
        panic!("Failed to write embedded axioms: {}", e);
    });
}
