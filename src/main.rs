use std::{path::{Path, PathBuf}, fs::{File, self, DirEntry}, io::BufReader, collections::HashMap, thread, str::FromStr, ffi::OsStr};
use ansi_term::{Color};
use counter::Counter;
use flume::{Sender, Receiver};
use multimap::MultiMap;
use serde_derive::Deserialize;
use clap::{Parser};
use glob_match::glob_match;
use crate::settings::Config;

mod multimap;
mod counter;
mod settings;



#[derive(Clone)]
struct Renderer<'a> {
    pub config: &'a Config
}
impl<'a> Renderer<'a> {
    pub fn render_file(&self, path: &PathBuf) {
        let filename_os = path.file_name().unwrap();
        let filename = filename_os.to_str().unwrap();

        let glyph = self.config.get_associated_file_glyph(filename);
        let color = self.config.get_associated_file_color(filename);
        let style = hex_to_color(color).normal();
        println!("{} {}", style.paint(glyph), style.paint(filename));
    }

    pub fn render_dir(&self, path: &PathBuf, ignored: bool) {
        let filename_os = path.file_name().unwrap();
        let filename = filename_os.to_str().unwrap();

        let glyph = self.config.get_associated_dir_glyph(filename);

        if ignored {
            let color = &self.config.colors.directories.ignored;
            let style = hex_to_color(color).normal();
            println!("{} {}{}", style.paint(glyph), style.paint(filename), style.paint("/..."));

        } else {
            let color = self.config.get_associated_dir_color(filename);
            let style = hex_to_color(color).normal();
            println!("{} {}", style.paint(glyph), style.paint(filename));
        }
    }

    pub fn render_skippedfiles(&self, ext: &String, count: i32) {
        let glyph = self.config.get_associated_ext_glyph(ext);

        let color = self.config.get_associated_ext_color(ext);
        let style = hex_to_color(color).normal();

        let value = format!("{} {} files...", count, ext);
        println!("{} {}", style.paint(glyph), style.paint(value));
    }
}


fn hex_to_color(hex: &String) -> Color {
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap();
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap();
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap();
    Color::RGB(r, g, b)
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

struct FoundItemEvent {
    is_dir: bool,
    is_last: bool,
    depth: usize,
    name: PathBuf,
}




fn main() {
    let config = Config::load();

    let path: PathBuf = config.get_clean_current_path();
    println!("{}", path.display());

    let renderer = Renderer {config: &config};

    let path_ref = &path;
    let config_ref = &config;

    thread::scope(|scope|{
        let (tx_fs, rx_compute) = flume::unbounded();
        let (tx_compute, rx_render) = flume::unbounded();

        scope.spawn(move || {
            list_files2(path_ref, config_ref, 0, &tx_fs);
        });
        scope.spawn(move || {
            compute(config_ref, &rx_compute, &tx_compute);
        });

        render_files(&renderer, rx_render);
    });
}




fn render_files(renderer: &Renderer, rx: Receiver<RenderItem>) {
    for item in rx.iter() {
        for _ in 0..item.depth {
            print!("{}  ", renderer.config.glyphs.get("pipe-v").unwrap());
        }

        if item.is_last && item.is_leaf {
            print!("{}", renderer.config.glyphs.get("pipe-e").unwrap());
        } else {
            print!("{}", renderer.config.glyphs.get("pipe-t").unwrap());
        }

        print!("{} ", renderer.config.glyphs.get("pipe-h").unwrap());
        match item.item {
            RenderType::File(f) => renderer.render_file(&f.path),
            RenderType::Dir(d) => renderer.render_dir(&d.path, item.is_leaf),
            RenderType::SkppedFiles(s) => renderer.render_skippedfiles(&s.ext, s.count),
        };
    }
}

fn compute(config: &Config, rx_compute: &Receiver<FoundItemEvent>, tx_compute: &Sender<RenderItem>) {
    for item in rx_compute.iter() {
        let i = match item.is_dir {
            true => RenderType::Dir(FileRenderItem { path: item.name }),
            false => RenderType::File(FileRenderItem { path: item.name}),
        };
        tx_compute.send(RenderItem {
            item: i,
            depth: item.depth,
            is_leaf: false,
            is_last: item.is_last
        }).unwrap();
    }
}


fn list_files2(path: &PathBuf, config: &Config, depth: usize, tx_fs: &Sender<FoundItemEvent>) {
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

        tx_fs.send(FoundItemEvent {
            is_dir: false,
            depth: depth,
            name: file.path(),
            is_last: is_last
        }).unwrap();
    }

    for dir in dirs {
        let path = dir.path();
        let is_ignored = config.is_dir_ignored(&path);

        c +=1;
        let is_last = c == total;

        tx_fs.send(FoundItemEvent {
            is_dir: true,
            depth: depth,
            name: dir.path(),
            is_last: is_last
        }).unwrap();

        if !is_ignored {
            crate::list_files2(&dir.path(), config, depth+1, tx_fs);
        }
    }
}


//
// fn list_files(path: &PathBuf, settings: &Settings, depth: usize, tx: &Sender<RenderItem>, unfold: bool, globbing: &Option<String>) {
//     let paths = fs::read_dir(path).unwrap();
//     let mut files: Vec<DirEntry> = Vec::with_capacity(32);
//     let mut dirs: Vec<DirEntry> = Vec::with_capacity(32);
//     for path in paths {
//         let path = path.unwrap();
//         if path.file_type().unwrap().is_dir() {
//             dirs.push(path);
//         } else {
//             if let Some(globbing) = &globbing {
//                 let path_buff = path.file_name();
//                 let path_str = path_buff.to_str().unwrap();
//                 // println!("{path_str}");
//                 if glob_match(globbing, path_str) {
//                     files.push(path);
//                 }
//             }
//             else {
//                 files.push(path);
//             }
//         }
//     }
//
//     let total = files.len() + dirs.len();
//     let mut c = 0;
//
//     if unfold || files.len() < settings.extensions_fold_count {
//         for file in files {
//             c +=1;
//             let is_last = c == total;
//
//             tx.send(RenderItem {
//                 item: RenderType::File(FileRenderItem { path: file.path() }),
//                 depth: depth,
//                 is_leaf: true,
//                 is_last: is_last
//             }).unwrap();
//         }
//     } else {
//
//         // Find all extensions
//         let mut paths: Vec<(PathBuf, Option<String>)> = Vec::with_capacity(files.len());
//         let mut counter: Counter<String> = Counter::new();
//         for file in files {
//             let path = file.path();
//             if let Some(ext) = path.extension() {
//                 let ext = ext.to_str().unwrap().to_string();
//                 counter.inc(&ext);
//                 paths.push((path, Some(ext)));
//             } else {
//                 paths.push((path, None));
//             }
//         }
//
//         let max_fold = settings.extensions_fold_count as i32;
//
//         for (k, v) in counter.iter() {
//             if *v >= max_fold {
//                 c += std::cmp::max(0, *v) as usize;
//                 let is_last = c == total;
//
//                 tx.send(RenderItem {
//                     item: RenderType::SkppedFiles(SkippedRenderIten { ext: k.clone(), count: *v }),
//                     depth: depth,
//                     is_leaf: true,
//                     is_last: is_last
//                 }).unwrap();
//             }
//         }
//
//         for (path, ext) in paths {
//             let display = match ext {
//                 Some(ext) => counter.get(&ext).map_or(true, |c| c < max_fold),
//                 None => true,
//             };
//
//             if display {
//                 c += 1;
//                 let is_last = c == total;
//
//                 tx.send(RenderItem {
//                     item: RenderType::File(FileRenderItem { path }),
//                     depth: depth,
//                     is_leaf: true,
//                     is_last: is_last
//                 }).unwrap();
//             }
//         }
//     }
//
//     for dir in dirs {
//         let path = dir.path();
//         let is_ignored = settings.is_dir_ignored(&path);
//
//         c +=1;
//         let is_last = c == total;
//
//         tx.send(RenderItem {
//             item: RenderType::Dir(FileRenderItem { path }),
//             depth: depth,
//             is_leaf: is_ignored,
//             is_last: is_last
//         }).unwrap();
//
//         if !is_ignored {
//             list_files(&dir.path(), settings, depth+1, tx, unfold, globbing);
//         }
//     }
// }