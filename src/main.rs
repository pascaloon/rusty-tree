use std::{path::{Path, PathBuf}, fs::{File, self, DirEntry}, io::BufReader, collections::HashMap, thread};
use ansi_term::{Color};
use flume::{Sender, Receiver};
use serde_derive::Deserialize;
use clap::{Arg, App};

#[derive(Deserialize, Debug, Clone)]
struct DirectoryIconSet {
    default: String,
    symlink: String,
    junction: String,
    wellknown: HashMap<String, String>
}

#[derive(Deserialize, Debug, Clone)]
struct FileIconSet {
    default: String,
    symlink: String,
    junction: String,
    wellknown: HashMap<String, String>,
    extensions: HashMap<String, String>
}


#[derive(Deserialize, Debug, Clone)]
struct IconSet {
    directories: DirectoryIconSet,
    files: FileIconSet
}

#[derive(Deserialize, Debug, Clone)]
struct DirectoryColorSet {
    default: String,
    ignored: String,
    symlink: String,
    junction: String,
    wellknown: HashMap<String, String>
}

#[derive(Deserialize, Debug, Clone)]
struct FileColorSet {
    default: String,
    symlink: String,
    junction: String,
    wellknown: HashMap<String, String>,
    extensions: HashMap<String, String>
}


#[derive(Deserialize, Debug, Clone)]
struct ColorSet {
    directories: DirectoryColorSet,
    files: FileColorSet
}


#[derive(Deserialize, Debug, Clone)]
struct Settings {
    ignored_dirs: Vec<String>,
}

impl Settings {
    fn is_dir_ignored(&self, path: &PathBuf) -> bool {
        self.ignored_dirs
            .iter()
            .any(|f| path.ends_with(f))
    }
}

#[derive(Clone)]
struct Renderer<'a> {
    glyphs: HashMap<String, String>,
    icons: IconSet,
    colors: ColorSet,
    settings: &'a Settings
}
impl<'a> Renderer<'a> {
    pub fn render_file(&self, path: &PathBuf) {
        let filename_os = path.file_name().unwrap();
        let filename = filename_os.to_str().unwrap();

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

    pub fn render_dir(&self, path: &PathBuf, ignored: bool) {
        let filename_os = path.file_name().unwrap();
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
    }
}

struct RenderItem {
    path: PathBuf,
    is_dir: bool,
    is_last: bool,
    is_leaf: bool,
    depth: usize
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

    let current_exe = std::env::current_exe().unwrap();
    let data_dir = current_exe.parent().unwrap().join("../..");
    let data_dir_path = data_dir.as_path();

    let glyphs = load_glyphs(data_dir_path);
    let icons = load_icons(data_dir_path);
    let colors = load_colors(data_dir_path);
    let settings = load_settings(data_dir_path);
    let renderer = Renderer {glyphs, icons, colors, settings: &settings};

    let (tx, rx) = flume::unbounded();

    let path_ref = path;
    let settings = settings.clone();
    let tx_ref = tx;

    thread::spawn(move || {
        list_files(&path_ref, &settings, 0, &tx_ref);
    });


    render_files(&renderer, rx);
}

fn load_glyphs(path: &Path) -> HashMap<String, String> {
    let path = path.join("data/glyphs.json");
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);
    serde_json::from_reader(reader).unwrap()
}

fn load_icons(path: &Path) -> IconSet {
    let path = path.join("data/icons.json");
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);
    serde_json::from_reader(reader).unwrap()
}

fn load_colors(path: &Path) -> ColorSet {
    let path = path.join("data/colors.json");
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);
    serde_json::from_reader(reader).unwrap()
}

fn load_settings(path: &Path) -> Settings {
    let path = path.join("data/settings.json");
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

fn render_files(renderer: &Renderer, rx: Receiver<RenderItem>) {
    for item in rx.iter() {
        for _ in 0..item.depth {
            print!("{}  ", renderer.glyphs.get("pipe-v").unwrap());
        }

        if item.is_last && item.is_leaf {
            print!("{}", renderer.glyphs.get("pipe-e").unwrap());
        } else {
            print!("{}", renderer.glyphs.get("pipe-t").unwrap());
        }

        print!("{} ", renderer.glyphs.get("pipe-h").unwrap());
        if item.is_dir {
            renderer.render_dir(&item.path, item.is_leaf);
        } else {
            renderer.render_file(&item.path);
        }
    }
}

fn list_files(path: &PathBuf, settings: &Settings, depth: usize, tx: &Sender<RenderItem>) {
    let paths = fs::read_dir(path).unwrap();
    let mut files: Vec<DirEntry> = Vec::with_capacity(32);
    let mut dirs: Vec<DirEntry> = Vec::with_capacity(32);
    for path in paths {
        let path = path.unwrap();
        if path.file_type().unwrap().is_dir() {
            dirs.push(path);
        } else {
            files.push(path);
        }
    }

    let total = files.len() + dirs.len();
    let mut c = 0;

    for file in files {
        c +=1;
        let is_last = c == total;

        tx.send(RenderItem {
            path: file.path(),
            is_dir: false,
            depth: depth,
            is_leaf: true,
            is_last: is_last
        }).unwrap();
    }

    for dir in dirs {
        let path = dir.path();
        let is_ignored = settings.is_dir_ignored(&path);

        c +=1;
        let is_last = c == total;

        tx.send(RenderItem {
            path: path,
            is_dir: true,
            depth: depth,
            is_leaf: is_ignored,
            is_last: is_last
        }).unwrap();

        if !is_ignored {
            list_files(&dir.path(), settings, depth+1, tx);
        }
    }
}