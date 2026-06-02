use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let axiom_dir = PathBuf::from("src/compliance/axioms");

    println!("cargo:rerun-if-changed=src/compliance/axioms/");

    let mut compiled_axioms = Vec::new();

    for entry in fs::read_dir(&axiom_dir).expect("Failed to read axiom directory") {
        let entry = entry.expect("Failed to read entry");
        let path = entry.path();

        if path.extension().map_or(false, |ext| ext == "asl") {
            let source = fs::read_to_string(&path)
                .expect(&format!("Failed to read axiom file: {:?}", path));

            let framework = path.file_stem()
                .unwrap()
                .to_string_lossy()
                .to_uppercase();

            match seedc::compile(&source) {
                Ok(bytecode) => {
                    let output_path = out_dir.join(format!("{}.aslb", framework.to_lowercase()));
                    fs::write(&output_path, &bytecode)
                        .expect(&format!("Failed to write compiled axiom: {:?}", output_path));
                    compiled_axioms.push((framework.clone(), output_path));
                    println!("cargo:warning=Compiled {} axioms successfully", framework);
                }
                Err(e) => {
                    panic!("Failed to compile {} axioms: {:?}", framework, e);
                }
            }
        }
    }

    // Generate Rust source that embeds compiled bytecode
    let mut embed_code = String::from("
        use std::collections::HashMap;

        pub fn get_embedded_bytecode() -> HashMap<String, Vec<u8>> {
            let mut map = HashMap::new();
    ");

    for (framework, path) in &compiled_axioms {
        let path_str = path.to_string_lossy();
        embed_code.push_str(&format!(
            "map.insert(\"{}\".to_string(), include_bytes!(\"{}\").to_vec());\n",
            framework, path_str
        ));
    }

    embed_code.push_str("
            map
        }
    ");

    let embed_path = out_dir.join("embedded_axioms.rs");
    fs::write(&embed_path, embed_code)
        .expect("Failed to write embedded axioms source");

    println!("cargo:warning=ASL axiom compilation complete: {} frameworks compiled", compiled_axioms.len());
}
