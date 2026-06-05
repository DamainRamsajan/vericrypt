use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let axiom_dir = PathBuf::from("src/compliance/axioms");

    println!("cargo:rerun-if-changed=src/compliance/axioms/");

    let mut embed_code = String::from("use std::collections::HashMap;\n");
    embed_code.push_str("pub fn get_embedded_bytecode() -> HashMap<String, Vec<u8>> {\n");
    embed_code.push_str("    let mut map = HashMap::new();\n");

    if let Ok(entries) = fs::read_dir(&axiom_dir) {
        let mut found = entries
            .flatten()
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "asl"))
            .collect::<Vec<_>>();

        found.sort_by_key(|e| e.path());

        for entry in found {
            let path = entry.path();
            let framework = path.file_stem().unwrap().to_string_lossy().to_uppercase();
            let source = fs::read_to_string(&path).unwrap_or_default();

            match seedc::compile(&source) {
                Ok(bytecode) => {
                    embed_code.push_str(&format!(
                        "    map.insert(\"{}\".to_string(), vec!{:?});\n",
                        framework, bytecode
                    ));
                    eprintln!("cargo:warning=Compiled axiom {} ({} bytes)", framework, bytecode.len());
                }
                Err(e) => {
                    panic!("build error: failed to compile axiom '{}' at {}: {:?}", framework, path.display(), e);
                }
            }
        }
    }

    embed_code.push_str("    map\n");
    embed_code.push_str("}\n");

    fs::write(out_dir.join("embedded_axioms.rs"), embed_code.as_bytes()).unwrap();
}
