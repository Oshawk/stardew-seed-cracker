use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

fn main() {
    let out_path: PathBuf = Path::new(&env::var("OUT_DIR").unwrap()).join("codegen.rs");
    let mut out_file: BufWriter<File> = BufWriter::new(File::create(&out_path).unwrap());

    let objects_path = Path::new("assets/Objects.json");
    let objects_file = File::open(objects_path).unwrap();
    let objects_json: serde_json::Value = serde_json::from_reader(objects_file).unwrap();

    // Parse content: { "id_str": { "Name": "...", "Price": N, ... }, ... }
    let mut objects_map: HashMap<u32, (String, u32)> = HashMap::new();
    for (key, value) in objects_json
        .get("content")
        .unwrap()
        .as_object()
        .unwrap()
    {
        let id: u32 = match key.parse::<u32>() {
            Ok(id) => id,
            Err(_) => continue,
        };
        let obj = match value.as_object() {
            Some(o) => o,
            None => continue,
        };
        let name = match obj.get("Name").and_then(|v| v.as_str()) {
            Some(s) => s.to_string(),
            None => continue,
        };
        // Price field may be absent or null; default to 0
        let price = obj
            .get("Price")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;

        objects_map.insert(id, (name, price));
    }

    // Generate OBJECTS phf::Map<u32, ObjectData>
    let mut objects_builder: phf_codegen::Map<u32> = phf_codegen::Map::new();
    for (id, (name, price)) in &objects_map {
        objects_builder.entry(
            *id,
            format!(
                "ObjectData {{ name: {:?}, price: {:?} }}",
                name.as_str(),
                price
            )
            .as_str(),
        );
    }

    // Generate OBJECTS_BY_NAME: [u32; N] sorted by name ascending
    let mut objects_sorted: Vec<(u32, String)> = objects_map
        .iter()
        .map(|(id, (name, _))| (*id, name.clone()))
        .collect();
    objects_sorted.sort_by(|a, b| a.1.cmp(&b.1).then(a.0.cmp(&b.0)));
    let sorted_ids: Vec<u32> = objects_sorted.iter().map(|(id, _)| *id).collect();

    writeln!(
        &mut out_file,
        "pub static OBJECTS: phf::Map<u32, ObjectData> = {};",
        objects_builder.build()
    )
    .unwrap();

    writeln!(
        &mut out_file,
        "pub static OBJECTS_BY_NAME: [u32; {}] = {:?};",
        sorted_ids.len(),
        sorted_ids
    )
    .unwrap();
}
