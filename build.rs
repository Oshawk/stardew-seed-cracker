use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use xxhash_rust::xxh32::xxh32;

fn main() {
    let out_path: PathBuf = Path::new(&env::var("OUT_DIR").unwrap()).join("codegen.rs");
    let mut out_file: BufWriter<File> = BufWriter::new(File::create(&out_path).unwrap());

    // Parse Objects.json (StardewXnbHack flat JSON format).
    // serde_json with preserve_order reads keys in file order, which matches the game's
    // C# Dictionary iteration order — critical for correct RNG key assignment.
    let objects_file: File = File::open(Path::new("assets/Objects.json")).unwrap();
    let objects_json: serde_json::Value = serde_json::from_reader(objects_file).unwrap();
    let objects = objects_json.as_object().unwrap();

    let mut enumeration_entries: Vec<String> = Vec::new();
    let mut eligible_objects: Vec<(u16, String, u16)> = Vec::new();
    let mut eligible_index: u16 = 0;
    let mut object_positions_builder = phf_codegen::Map::<u16>::new();

    for (pos, (key, value)) in objects.iter().enumerate() {
        let name = value.get("Name").unwrap().as_str().unwrap();
        let price = value.get("Price").unwrap().as_i64().unwrap() as i32;
        let typ = value.get("Type").unwrap().as_str().unwrap();
        let category = value.get("Category").unwrap().as_i64().unwrap() as i32;
        let exclude = value
            .get("ExcludeFromRandomSale")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // RANDOM_ITEMS filter: numeric key in [2, 789], not excluded, price > 0.
        // Non-numeric keys (e.g. "MossSoup") still consume an RNG call in the game's
        // LINQ shuffle but are filtered out by int.TryParse, so they must remain in
        // the enumeration as Ineligible.
        let id = key.parse::<u16>().ok();
        let passes_random_items =
            id.is_some_and(|id| id >= 2 && id <= 789) && !exclude && price > 0;
        if !passes_random_items {
            enumeration_entries.push("EnumerationEntry { class: ObjectClass::Ineligible }".into());
            continue;
        }
        let id = id.unwrap();

        // Layer 2 (PerItemCondition): Category < 0, Category != -999, Type not Quest/Minerals/Arch
        let passes_condition = category < 0
            && category != -999
            && typ != "Quest"
            && typ != "Minerals"
            && typ != "Arch";
        if !passes_condition {
            enumeration_entries.push("EnumerationEntry { class: ObjectClass::Intermediate }".into());
            continue;
        }

        // Fully eligible for the Traveling Cart
        enumeration_entries.push(format!(
            "EnumerationEntry {{ class: ObjectClass::FullyEligible({}) }}",
            eligible_index
        ));
        object_positions_builder.entry(id, &format!("{}u16", pos));
        eligible_objects.push((id, name.to_string(), price as u16));
        eligible_index += 1;
    }

    // --- Write generated code ---

    // OBJECT_ENUMERATION: one entry per object in dictionary order
    writeln!(
        &mut out_file,
        "pub static OBJECT_ENUMERATION: [EnumerationEntry; {}] = [",
        enumeration_entries.len()
    )
    .unwrap();
    for entry in &enumeration_entries {
        writeln!(&mut out_file, "    {},", entry).unwrap();
    }
    writeln!(&mut out_file, "];").unwrap();

    // ELIGIBLE_OBJECTS: fully-eligible items in dictionary order
    writeln!(
        &mut out_file,
        "pub static ELIGIBLE_OBJECTS: [EligibleObject; {}] = [",
        eligible_objects.len()
    )
    .unwrap();
    for (id, name, price) in &eligible_objects {
        writeln!(
            &mut out_file,
            "    EligibleObject {{ id: {}, name: {:?}, price: {} }},",
            id, name, price
        )
        .unwrap();
    }
    writeln!(&mut out_file, "];").unwrap();

    // ELIGIBLE_OBJECTS_SORTED: indices into ELIGIBLE_OBJECTS, sorted by (name, id)
    let mut sorted_indices: Vec<u16> = (0..eligible_objects.len() as u16).collect();
    sorted_indices.sort_by_key(|&idx| {
        let (id, ref name, _) = eligible_objects[idx as usize];
        (name.clone(), id)
    });
    writeln!(
        &mut out_file,
        "pub static ELIGIBLE_OBJECTS_SORTED: [u16; {}] = {:?};",
        sorted_indices.len(),
        sorted_indices
    )
    .unwrap();

    // OBJECT_POSITIONS: object ID -> enumeration position (PHF map)
    writeln!(
        &mut out_file,
        "pub static OBJECT_POSITIONS: phf::Map<u16, u16> = {};",
        object_positions_builder.build()
    )
    .unwrap();

    // SYNCED_RANDOM key hashes
    let synced_keys = [
        ("HASH_CART_FEZ", "cart_fez"),
        ("HASH_CART_COFFEE_BEAN", "cart_coffee_bean"),
        ("HASH_CART_RARECROW", "cart_rarecrow"),
        ("HASH_CART_RETRO_CATALOGUE", "cart_retroCatalogue"),
        ("HASH_TRAVELER_SKILL_BOOK", "travelerSkillBook"),
        ("HASH_TEASET", "teaset"),
        ("HASH_CART_JOJA_CATALOGUE", "cart_jojaCatalogue"),
        ("HASH_CART_JUNIMO_CATALOGUE", "cart_junimoCatalogue"),
    ];
    for (const_name, key_str) in &synced_keys {
        let hash = xxh32(key_str.as_bytes(), 0) as i32;
        writeln!(&mut out_file, "pub const {}: i32 = {}i32;", const_name, hash).unwrap();
    }

    // Constants
    writeln!(
        &mut out_file,
        "pub const TOTAL_OBJECTS: usize = {};",
        enumeration_entries.len()
    )
    .unwrap();
    writeln!(
        &mut out_file,
        "pub const TOTAL_ELIGIBLE: usize = {};",
        eligible_objects.len()
    )
    .unwrap();
}
