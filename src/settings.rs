use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use clap::Parser;
use glob_match::glob_match;
use serde_derive::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct DirectoryIconSet {
    pub default: String,
    pub symlink: String,
    pub junction: String,
    pub wellknown: HashMap<String, String>
}

#[derive(Deserialize, Debug, Clone)]
pub struct FileIconSet {
    pub default: String,
    pub symlink: String,
    pub junction: String,
    pub wellknown: HashMap<String, String>,
    pub extensions: HashMap<String, String>
}


#[derive(Deserialize, Debug, Clone)]
pub struct IconSet {
    pub directories: DirectoryIconSet,
    pub files: FileIconSet
}

#[derive(Deserialize, Debug, Clone)]
pub struct DirectoryColorSet {
    pub default: String,
    pub ignored: String,
    pub symlink: String,
    pub junction: String,
    pub wellknown: HashMap<String, String>
}

#[derive(Deserialize, Debug, Clone)]
pub struct FileColorSet {
    pub default: String,
    pub symlink: String,
    pub junction: String,
    pub wellknown: HashMap<String, String>,
    pub extensions: HashMap<String, String>
}


#[derive(Deserialize, Debug, Clone)]
pub struct ColorSet {
    pub directories: DirectoryColorSet,
    pub files: FileColorSet
}


#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Name of the person to greet
    #[clap()]
    pub path: Option<String>,

    #[arg(short, long, default_value_t = false)]
    pub unfold: bool,

    #[arg(short, long)]
    pub filter: Option<String>,
}


#[derive(Deserialize, Debug, Clone)]
pub struct Settings {
    pub ignored_dirs: Vec<String>,
    pub extensions_fold_count: usize
}

pub struct Config {
    pub settings: Settings,
    pub glyphs: HashMap<String, String>,
    pub icons: IconSet,
    pub colors: ColorSet,
    pub args: Args,

}

fn load_glyphs(path: &Path) -> HashMap<String, String> {
    load_from_file(&path.join("data/glyphs.json"))
}

fn load_icons(path: &Path) -> IconSet {
    load_from_file(&path.join("data/icons.json"))
}

fn load_colors(path: &Path) -> ColorSet {
    load_from_file(&path.join("data/colors.json"))
}

fn load_settings(path: &Path) -> Settings {
    load_from_file(&path.join("data/settings.json"))
}

fn load_from_file<T>(path: &PathBuf) -> T
    where T: serde::de::DeserializeOwned,
{
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);
    serde_json::from_reader(reader).unwrap()
}

fn get_data_dir_path() -> PathBuf {
    let current_exe = std::env::current_exe().unwrap();
    let data_dir = current_exe.parent().unwrap().join("../..");
    data_dir
}


/// A utility method that tries to find the **longest extension matching the filename**
fn find_item_from_extension<'b>(map: &'b HashMap<String, String>, filename: &str) -> Option<&'b String> {
    if let Some(v) = map.get(filename) {
        return Some(v);
    }
    for (pos, _) in filename.match_indices('.') {
        let pos = pos + 1;
        if pos < filename.len() {
            let (_, ext) = filename.split_at(pos);
            if let Some(v) = map.get(ext) {
                return Some(v);
            }
        }
    }

    None
}

impl Config {
    pub fn load() -> Self {
        let data_dir = get_data_dir_path();
        let data_dir = data_dir.as_path();
        Config {
            settings: load_settings(data_dir),
            glyphs: load_glyphs(data_dir),
            icons: load_icons(data_dir),
            colors: load_colors(data_dir),
            args: Args::parse(),
        }
    }

    pub fn is_dir_ignored(&self, path: &PathBuf) -> bool {
        self.settings.ignored_dirs
            .iter()
            .any(|f| path.ends_with(f))
    }

    pub fn get_clean_current_path(&self) -> PathBuf {
        let path_str = self.args.path.clone().unwrap_or(".".to_string());
        let path = if Path::new(&path_str).is_absolute() {
            PathBuf::from(path_str)
        } else {
            std::env::current_dir().unwrap().join(path_str)
        }.canonicalize().unwrap();
        let path = path.to_str().unwrap().strip_prefix(r"\\?\").unwrap_or(path.to_str().unwrap());
        PathBuf::from_str(path).unwrap()
    }

    pub fn get_associated_ext_glyph(&self, ext: &str) -> &String {
        let icon =
            if let Some(icon) = self.icons.files.extensions.get(ext) {
                icon
            } else {
                &self.icons.files.default
            };
        self.glyphs.get(icon).unwrap()
    }

    pub fn get_associated_ext_color(&self, ext: &str) -> &String {
        if let Some(color) = self.colors.files.extensions.get(ext) {
            color
        } else {
            &self.colors.files.default
        }
    }

    pub fn get_associated_file_glyph(&self, filename: &str) -> &String {
        let icon =
            if let Some(icon) = self.icons.files.wellknown.get(filename) {
                icon
            } else if let Some(icon) = find_item_from_extension(&self.icons.files.extensions, filename) {
                icon
            } else {
                &self.icons.files.default
            };
        self.glyphs.get(icon).unwrap()
    }

    pub fn get_associated_file_color(&self, filename: &str) -> &String {
        if let Some(color) = self.colors.files.wellknown.get(filename) {
            color
        } else if let Some(color) = find_item_from_extension(&self.colors.files.extensions, filename) {
            color
        } else {
            &self.colors.files.default
        }
    }

    pub fn get_associated_dir_glyph(&self, filename: &str) -> &String {
        let icon = self.icons.directories.wellknown.get(filename)
            .unwrap_or(&self.icons.files.default);
        self.glyphs.get(icon).unwrap()
    }

    pub fn get_associated_dir_color(&self, filename: &str) -> &String {
        self.colors.directories.wellknown.get(filename).unwrap_or(&self.colors.files.default)
    }

    pub fn is_filtered(&self) -> bool {
        self.args.filter.is_some()
    }

    pub fn is_file_valid(&self, path: &Path) -> bool {
        match &self.args.filter {
            Some(gm) => glob_match(gm, path.file_name().unwrap().to_str().unwrap()),
            _ => true
        }
    }

}