use phf::phf_set;
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

#[derive(Debug)]
struct ObjectInformation {
    pub name: String,
    pub price: u16,
    pub edibility: i16,
    pub type_and_category: String,
    pub display_name: String,
    pub description: String,
}

static OFF_LIMIT: phf::Set<u16> = phf_set!(
    69u16, 73u16, 79u16, 91u16, 158u16, 159u16, 160u16, 161u16, 162u16, 163u16, 261u16, 277u16,
    279u16, 289u16, 292u16, 305u16, 308u16, 326u16, 341u16, 413u16, 417u16, 437u16, 439u16, 447u16,
    454u16, 460u16, 645u16, 680u16, 681u16, 682u16, 688u16, 689u16, 690u16, 774u16, 775u16, 797u16,
    798u16, 799u16, 800u16, 801u16, 802u16, 803u16, 807u16, 812u16
);

fn main() {
    let out_path: PathBuf = Path::new(&env::var("OUT_DIR").unwrap()).join("codegen.rs");
    let mut out_file: BufWriter<File> = BufWriter::new(File::create(&out_path).unwrap());

    let object_information_path: &Path = Path::new("assets/ObjectInformation.json");
    let object_information_file: File = File::open(&object_information_path).unwrap();
    let object_information_json: serde_json::Value =
        serde_json::from_reader(object_information_file).unwrap();

    let mut object_information_map: HashMap<u16, ObjectInformation> = HashMap::new();
    for (key, value) in object_information_json
        .get("content")
        .unwrap()
        .as_object()
        .unwrap()
    {
        let value_split: Vec<&str> = value.as_str().unwrap().split("/").collect();
        object_information_map.insert(
            key.parse::<u16>().unwrap(),
            ObjectInformation {
                name: value_split[0].to_string(),
                price: value_split[1].parse::<u16>().unwrap(),
                edibility: value_split[2].parse::<i16>().unwrap(),
                type_and_category: value_split[3].to_string(),
                display_name: value_split[4].to_string(),
                description: value_split[5].to_string(),
            },
        );
    }

    let mut object_information_builder: phf_codegen::Map<u16> = phf_codegen::Map::new();
    for (key, value) in &object_information_map {
        object_information_builder.entry(key.clone(), format!("{:?}", value).as_str());
    }

    let object_information_sorted: Vec<u16> = {
        let mut object_information_vec: Vec<(&u16, &ObjectInformation)> =
            object_information_map.iter().collect();
        object_information_vec
            .sort_by_key(|(index, object_information)| (&object_information.name, *index));
        object_information_vec
            .into_iter()
            .map(|(index, _object_information)| *index)
            .collect()
    };

    let mut first_filter_set: HashSet<u16> = HashSet::new();
    for index in 0u16..790u16 {
        if !object_information_map.contains_key(&index) || OFF_LIMIT.contains(&index) {
            first_filter_set.insert(index);
        }
    }

    let mut first_filter_builder: phf_codegen::Set<u16> = phf_codegen::Set::new();
    for index in &first_filter_set {
        first_filter_builder.entry(index.clone());
    }

    let mut second_filter_set: HashSet<u16> = HashSet::new();
    for index in 0u16..790u16 {
        if !first_filter_set.contains(&index) {
            let value: &ObjectInformation = object_information_map.get(&index).unwrap();
            if !value.type_and_category.contains("-")
                || value.price <= 0
                || value.type_and_category.contains("-13")
                || value.type_and_category == "Quest"
                || value.name == "Weeds"
                || value.type_and_category.contains("Minerals")
                || value.type_and_category.contains("Arch")
            {
                second_filter_set.insert(index);
            }
        }
    }

    let mut second_filter_builder: phf_codegen::Set<u16> = phf_codegen::Set::new();
    for index in &second_filter_set {
        second_filter_builder.entry(index.clone());
    }

    // Pretty sure this is wrong.
    let mut fast_exclude_map: HashMap<u16, u16> = HashMap::new();
    for index in 0u16..790u16 {
        let mut value: u16 = index.clone();
        loop {
            value += 1u16;
            value %= 790u16;

            if !first_filter_set.contains(&value) && !second_filter_set.contains(&value) {
                break;
            }
        }

        fast_exclude_map.insert(index, value);
    }

    let mut fast_exclude_builder: phf_codegen::Map<u16> = phf_codegen::Map::new();
    for (key, value) in &fast_exclude_map {
        fast_exclude_builder.entry(key.clone(), format!("{:?}", value).as_str());
    }

    writeln!(
        &mut out_file,
        "pub static OBJECT_INFORMATION: phf::Map<u16, ObjectInformation> = {};",
        object_information_builder.build()
    )
    .unwrap();
    writeln!(
        &mut out_file,
        "pub static OBJECT_INFORMATION_SORTED: [u16; {}] = {:?};",
        object_information_sorted.len(),
        object_information_sorted
    )
    .unwrap();
    writeln!(
        &mut out_file,
        "pub static FIRST_FILTER: phf::Set<u16> = {};",
        first_filter_builder.build()
    )
    .unwrap();
    writeln!(
        &mut out_file,
        "pub static SECOND_FILTER: phf::Set<u16> = {};",
        second_filter_builder.build()
    )
    .unwrap();
    writeln!(
        &mut out_file,
        "pub static FAST_EXCLUDE: phf::Map<u16, u16> = {};",
        fast_exclude_builder.build()
    )
    .unwrap();
}
