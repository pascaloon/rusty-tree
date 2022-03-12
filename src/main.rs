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

    let icon_set = load_icons();
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
