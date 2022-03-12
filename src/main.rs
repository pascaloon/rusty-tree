use std::{path::Path, fs::File, io::BufReader, collections::HashMap};
use serde::Deserialize;
use serde_derive::Deserialize;

#[derive(Deserialize, Debug)]
struct DirectoryIconSet {
    default: String,
    symlink: String,
    junction: String,
    wellknown: HashMap<String, String>
}

#[derive(Deserialize, Debug)]
struct FileIconSet {
    default: String,
    symlink: String,
    junction: String,
    wellknown: HashMap<String, String>,
    extensions: HashMap<String, String>
}


#[derive(Deserialize, Debug)]
struct IconSet {
    directories: DirectoryIconSet,
    files: FileIconSet
}

fn main() {

    // let branch_icon = dict.get("nf-dev-git_branch").unwrap();
    // println!("icon: {}", branch_icon);

    // for (key, icon) in &dict {
    //     println!("{}: {}", key, icon);
    // }

    let glyphs = load_glyphs();
    let icons = load_icons();
    
    let rust_icon = icons.files.extensions.get(".rs").unwrap();
    let rust_glyph = glyphs.get(rust_icon).unwrap();
    println!("rust icon: {} -> {}", rust_icon, rust_glyph);

    let cs_icon = icons.files.extensions.get(".cs").unwrap();
    let cs_glyph = glyphs.get(cs_icon).unwrap();
    println!("cs icon: {} -> {}", cs_icon, cs_glyph);
}

fn load_glyphs() -> HashMap<String, String> {
    let path = Path::new("data/glyphs.json");
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);
    serde_json::from_reader(reader).unwrap()
}

fn load_icons() -> IconSet {
    let path = Path::new("data/icons.json");
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);
    serde_json::from_reader(reader).unwrap()
}
