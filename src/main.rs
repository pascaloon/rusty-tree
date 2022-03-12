use std::{path::{Path, PathBuf}, fs::{File, self, DirEntry}, io::BufReader, collections::HashMap, str::FromStr};
use ansi_term::{Color};
use serde_derive::Deserialize;
use clap::{Arg, App};

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
    ignored: String,
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


#[derive(Deserialize, Debug)]
struct Settings {
    ignored_dirs: Vec<String>,
}


struct Renderer {
    glyphs: HashMap<String, String>,
    icons: IconSet,
    colors: ColorSet,
    settings: Settings
}
impl Renderer {
    pub fn render_file(&self, file: &DirEntry) {
        let filename_os = file.file_name();
        let filename = filename_os.to_str().unwrap();

        let path = file.path();
        let extension = match path.extension() {
            Some(ext) => ext.to_str().unwrap(),
            None => ""
        };

        let icon = 
            if let Some(icon) = self.icons.files.wellknown.get(filename) {
                icon
            } else if let Some(icon) = self.icons.files.extensions.get(extension) {
                icon
            } else {
                &self.icons.files.default
            };
        let glyph = self.glyphs.get(icon).unwrap();

        let color = 
            if let Some(color) = self.colors.files.wellknown.get(filename) {
                color
            } else if let Some(color) = self.colors.files.extensions.get(extension) {
                color
            } else {
                &self.colors.files.default
        };

        let style = hex_to_color(color).normal();
        println!("{} {}", style.paint(glyph), style.paint(filename));
    }

    pub fn render_dir(&self, file: &DirEntry, ignored: bool) {
        let filename_os = file.file_name();
        let filename = filename_os.to_str().unwrap();

        let icon = 
            if let Some(icon) = self.icons.directories.wellknown.get(filename) {
                icon
            } else {
                &self.icons.directories.default
            };
        let glyph = self.glyphs.get(icon).unwrap();

        if ignored {
            let color = &self.colors.directories.ignored;
            let style = hex_to_color(color).normal();
            println!("{} {}{}", style.paint(glyph), style.paint(filename), style.paint("/..."));

        } else {
            let color = 
                if let Some(color) = self.colors.directories.wellknown.get(filename) {
                    color
                } else {
                    &self.colors.directories.default
                };
            let style = hex_to_color(color).normal();
            println!("{} {}", style.paint(glyph), style.paint(filename));
        }

        let color = 
            if let Some(color) = self.colors.directories.wellknown.get(filename) {
                color
            } else {
                &self.colors.directories.default
        };
    }

    fn is_dir_ignored(&self, file: &DirEntry) -> bool {
        let path = file.path();
        self.settings.ignored_dirs
            .iter()
            .any(|f| path.ends_with(f))
    }
}


fn main() {

    let app_args = App::new("rusty-tree")
                          .version("1.0")
                          .author("pascaloon")
                          .about("shows file tree")
                          .arg(Arg::with_name("PATH")
                               .help("Path to render tree")
                               .required(false)
                               .index(1))
                          .get_matches();

    let path_str = app_args.value_of("PATH").unwrap_or(".");

    let path: PathBuf = if Path::new(path_str).is_absolute() {
        PathBuf::from(path_str)
    } else {
        std::env::current_dir().unwrap().join(path_str)
    };

    println!("{}", path.to_str().unwrap());

    let glyphs = load_glyphs();
    let icons = load_icons();
    let colors = load_colors();
    let settings = load_settings();
    let renderer = Renderer {glyphs, icons, colors, settings};


    list_files(&path, &renderer, 0);
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

fn load_settings() -> Settings {
    let path = Path::new("data/settings.json");
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);
    serde_json::from_reader(reader).unwrap()
}


fn hex_to_color(hex: &String) -> Color {
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap();
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap();
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap();
    Color::RGB(r, g, b)
}

fn list_files(path: &PathBuf, renderer: &Renderer, depth: usize) {
    let spaces = "                                    ";
    let paths = fs::read_dir(path).unwrap();
    for path in paths {
        let path = path.unwrap();
        if path.file_type().unwrap().is_dir() {
            print!("{}", &spaces[0..(depth*2)]);
            let is_ignored = renderer.is_dir_ignored(&path);
            renderer.render_dir(&path, is_ignored);
            if !is_ignored {
                list_files(&path.path(), renderer, depth+1);
            }
        } else {
            print!("{}", &spaces[0..(depth*2)]);
            renderer.render_file(&path);
        }
    }
}