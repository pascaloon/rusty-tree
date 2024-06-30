use std::collections::VecDeque;
use std::fs;
use std::fs::DirEntry;
use std::path::PathBuf;
use flume::{Receiver, Sender};
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
    let mut uncommited_dirs: VecDeque<FoundItemEvent> = VecDeque::with_capacity(8);

    for item in rx_io.iter() {
        if item.is_dir {
            uncommited_dirs.retain(|d| d.depth < item.depth);
            if item.is_ignored {
                tx_render.send(RenderItem {
                    item: RenderType::Dir(FileRenderItem { path: item.name }),
                    depth: item.depth,
                    is_leaf: true,
                    is_last: item.is_last
                }).unwrap();
            } else {
                uncommited_dirs.push_back(item);
            }
            continue;
        }

        if !config.is_file_valid(item.name.as_path()) {
            continue
        }

        while let Some(d) = uncommited_dirs.pop_front() {
            tx_render.send(RenderItem {
                item: RenderType::Dir(FileRenderItem { path: d.name }),
                depth: d.depth,
                is_leaf: (d.is_ignored && d.is_last),
                is_last: d.is_last
            }).unwrap();
        }

        tx_render.send(RenderItem {
            item: RenderType::File(FileRenderItem {path: item.name}),
            depth: item.depth,
            is_leaf: true,
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


