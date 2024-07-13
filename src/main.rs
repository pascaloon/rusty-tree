use std::{path::PathBuf, thread};
use ansi_term::Color;
use crate::crawler::{compute, IOEvent, list_files, render_files};
use crate::settings::Config;
mod multimap;
mod settings;
mod crawler;
mod renderer;
mod counter;


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


fn main() {
    let config = Config::load();

    let path: PathBuf = config.get_clean_current_path();
    println!("{}", path.display());

    let path_ref = &path;
    let config_ref = &config;

    thread::scope(|scope|{
        let (tx_io, rx_io) = crossbeam_channel::unbounded::<IOEvent>();
        let (tx_render, rx_render) = crossbeam_channel::unbounded::<RenderItem>();

        scope.spawn(move || {
            list_files(path_ref, config_ref, 0, &tx_io);
        });
        scope.spawn(move || {
            compute(config_ref, &rx_io, &tx_render);
        });
        render_files(&config, rx_render);
    });
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