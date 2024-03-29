use std::{path::{Path, PathBuf}, fs::{File, self, DirEntry}, io::BufReader, collections::HashMap, thread, str::FromStr, ffi::OsStr};
use ansi_term::{Color};
use counter::Counter;
use flume::{Sender, Receiver};
use multimap::MultiMap;
use serde_derive::Deserialize;
use clap::{Parser};

mod multimap;
mod counter;

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
    extensions_fold_count: usize
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

        let icon = 
            if let Some(icon) = self.icons.files.wellknown.get(filename) {
                icon
            } else if let Some(icon) = Self::find_item_from_extension(&self.icons.files.extensions, filename) {
                icon
            } else {
                &self.icons.files.default
            };

        let glyph = self.glyphs.get(icon).unwrap();

        let color = 
            if let Some(color) = self.colors.files.wellknown.get(filename) {
                color
            } else if let Some(color) = Self::find_item_from_extension(&self.colors.files.extensions, filename) {
                color
            } else {
                &self.colors.files.default
        };

        let style = hex_to_color(color).normal();
        println!("{} {}", style.paint(glyph), style.paint(filename));
    }

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

    pub fn render_skippedfiles(&self, ext: &String, count: i32) {
        let icon = 
            if let Some(icon) = self.icons.files.extensions.get(ext) {
                icon
            } else {
                &self.icons.files.default
            };

        let glyph = self.glyphs.get(icon).unwrap();

        let color = 
            if let Some(color) = self.colors.files.extensions.get(ext) {
                color
            } else {
                &self.colors.files.default
        };

        let style = hex_to_color(color).normal();
        let value = format!("{} {} files...", count, ext);
        println!("{} {}", style.paint(glyph), style.paint(value));
    }
}

enum RenderType {
    File(FileRenderItem),
    Dir(FileRenderItem),
    SkppedFiles(SkippedRenderIten)
}

struct FileRenderItem {
    path: PathBuf
}

struct SkippedRenderIten {
    ext: String,
    count: i32
}

struct RenderItem {
    item: RenderType,
    is_last: bool,
    is_leaf: bool,
    depth: usize
}



#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
   /// Name of the person to greet
   #[clap()]
   path: Option<String>,

   #[arg(short, long, default_value_t = false)]
   unfold: bool,
}


fn main() {

    let app_args = Args::parse();

    let path_str = app_args.path.unwrap_or(".".to_string());
    let unfold = app_args.unfold;

    let path: PathBuf = if Path::new(&path_str).is_absolute() {
        PathBuf::from(path_str)
    } else {
        std::env::current_dir().unwrap().join(path_str)
    }.canonicalize().unwrap();
    let path = path.to_str().unwrap().strip_prefix(r"\\?\").unwrap_or(path.to_str().unwrap());
    println!("{}", path);
    let path = PathBuf::from_str(path).unwrap();

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
        list_files(&path_ref, &settings, 0, &tx_ref, unfold);
    });


    render_files(&renderer, rx);
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
        match item.item {
            RenderType::File(f) => renderer.render_file(&f.path),
            RenderType::Dir(d) => renderer.render_dir(&d.path, item.is_leaf),
            RenderType::SkppedFiles(s) => renderer.render_skippedfiles(&s.ext, s.count),
        };
    }
}

fn list_files(path: &PathBuf, settings: &Settings, depth: usize, tx: &Sender<RenderItem>, unfold: bool) {
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

    if unfold || files.len() < settings.extensions_fold_count {
        for file in files {
            c +=1;
            let is_last = c == total;
    
            tx.send(RenderItem {
                item: RenderType::File(FileRenderItem { path: file.path() }),
                depth: depth,
                is_leaf: true,
                is_last: is_last
            }).unwrap();
        }
    } else {
        
        // Find all extensions
        let mut paths: Vec<(PathBuf, Option<String>)> = Vec::with_capacity(files.len());
        let mut counter: Counter<String> = Counter::new();
        for file in files {
            let path = file.path();
            if let Some(ext) = path.extension() {
                let ext = ext.to_str().unwrap().to_string();
                counter.inc(&ext);
                paths.push((path, Some(ext)));
            } else {
                paths.push((path, None));
            }
        }
        
        let max_fold = settings.extensions_fold_count as i32;

        for (k, v) in counter.iter() {
            if *v >= max_fold {
                c += std::cmp::max(0, *v) as usize;
                let is_last = c == total;

                tx.send(RenderItem {
                    item: RenderType::SkppedFiles(SkippedRenderIten { ext: k.clone(), count: *v }),
                    depth: depth,
                    is_leaf: true,
                    is_last: is_last
                }).unwrap();
            }
        }

        for (path, ext) in paths {
            let display = match ext {
                Some(ext) => counter.get(&ext).map_or(true, |c| c < max_fold),
                None => true,
            };

            if display {
                c += 1;
                let is_last = c == total;

                tx.send(RenderItem {
                    item: RenderType::File(FileRenderItem { path }),
                    depth: depth,
                    is_leaf: true,
                    is_last: is_last
                }).unwrap();
            }
        }
    }

    for dir in dirs {
        let path = dir.path();
        let is_ignored = settings.is_dir_ignored(&path);

        c +=1;
        let is_last = c == total;

        tx.send(RenderItem {
            item: RenderType::Dir(FileRenderItem { path }),
            depth: depth,
            is_leaf: is_ignored,
            is_last: is_last
        }).unwrap();

        if !is_ignored {
            list_files(&dir.path(), settings, depth+1, tx, unfold);
        }
    }
}