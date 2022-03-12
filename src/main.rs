use std::{path::Path, fs::File, io::BufReader, collections::HashMap};
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

#[derive(Deserialize, Debug)]
struct DirectoryColorSet {
    default: String,
    symlink: String,
    junction: String,
    wellknown: HashMap<String, String>
}

#[derive(Deserialize, Debug)]
struct FileColorSet {
    default: String,
    symlink: String,
    junction: String,
    wellknown: HashMap<String, String>,
    extensions: HashMap<String, String>
}


#[derive(Deserialize, Debug)]
struct ColorSet {
    directories: DirectoryColorSet,
    files: FileColorSet
}

fn main() {

    // let branch_icon = dict.get("nf-dev-git_branch").unwrap();
    // println!("icon: {}", branch_icon);

    // for (key, icon) in &dict {
    //     println!("{}: {}", key, icon);
    // }

    let glyphs = load_glyphs();
    let icons = load_icons();
    let colors = load_colors();
    
    let rust_icon = icons.files.extensions.get(".rs").unwrap();
    let rust_color = colors.files.extensions.get(".rs").unwrap();
    let rust_glyph = glyphs.get(rust_icon).unwrap();
    println!("rust icon: {} -> {} [{}]", rust_icon, rust_glyph, rust_color);

    let cs_icon = icons.files.extensions.get(".cs").unwrap();
    let cs_color = colors.files.extensions.get(".cs").unwrap();
    let cs_glyph = glyphs.get(cs_icon).unwrap();
    println!("cs icon: {} -> {} [{}]", cs_icon, cs_glyph, cs_color);
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

fn load_colors() -> ColorSet {
    let path = Path::new("data/colors.json");
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);
    serde_json::from_reader(reader).unwrap()
}