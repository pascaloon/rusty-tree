use std::fs;
use std::fs::DirEntry;
use std::path::{Path, PathBuf};
use flume::{Receiver, Sender};
use glob_match::glob_match;
use crate::{FileRenderItem, RenderItem, RenderType};
use crate::renderer::Renderer;
use crate::settings::Config;




pub struct FoundItemEvent {
    is_dir: bool,
    is_last: bool,
    is_ignored: bool,
    depth: usize,
    name: PathBuf,
}

pub fn list_files(path: &PathBuf, config: &Config, depth: usize, tx_io: &Sender<FoundItemEvent>) {
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
        tx_io.send(FoundItemEvent {
            is_dir: false,
            is_ignored: false,
            is_last: (c == total),
            depth,
            name: file.path(),
        }).unwrap();
    }

    for dir in dirs {
        let path = dir.path();
        let is_ignored = config.is_dir_ignored(&path);
        c +=1;
        tx_io.send(FoundItemEvent {
            is_dir: true,
            is_last: (c == total),
            is_ignored,
            depth,
            name: dir.path(),
        }).unwrap();

        if !is_ignored {
            list_files(&path, config, depth+1, tx_io);
        }
    }
}

pub fn compute(config: &Config, rx_io: &Receiver<FoundItemEvent>, tx_render: &Sender<RenderItem>) {
    for item in rx_io.iter() {
        let i = match item.is_dir {
            true => RenderType::Dir(FileRenderItem { path: item.name }),
            false => RenderType::File(FileRenderItem { path: item.name}),
        };
        tx_render.send(RenderItem {
            item: i,
            depth: item.depth,
            is_leaf: (item.is_ignored && item.is_last),
            is_last: item.is_last
        }).unwrap();
    }
}

pub fn render_files(config: &Config, rx_render: Receiver<RenderItem>) {
    let renderer = Renderer { config };
    for item in rx_render.iter() {
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


