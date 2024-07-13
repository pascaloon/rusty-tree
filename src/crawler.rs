use std::collections::VecDeque;
use std::fs;
use std::fs::DirEntry;
use std::path::PathBuf;
use crossbeam_channel::{Receiver, Sender};
use smallvec::{SmallVec, smallvec};
use crate::{FileRenderItem, RenderItem, RenderType};
use crate::renderer::Renderer;
use crate::settings::Config;

pub enum IOEvent {
    FilesListed(FilesInfo),
    DirectoryStarted(DirectoryInfo)
}

pub struct FileInfo {
    pub path: PathBuf
}

pub struct FilesInfo {
    files: SmallVec<[FileInfo; 4]>,
    depth: usize
}

pub struct DirectoryInfo {
    depth: usize,
    is_last: bool,
    is_ignored: bool,
    name: PathBuf
}


pub fn list_files(path: &PathBuf, config: &Config, depth: usize, tx_io: &Sender<IOEvent>) {
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

    let mut files_info = FilesInfo {
        files: smallvec![],
        depth
    };

    for file in files {
        c +=1;
        files_info.files.push(FileInfo {
           path: file.path()
        });
    }

    tx_io.send(IOEvent::FilesListed(files_info)).unwrap();

    for dir in dirs {
        let path = dir.path();
        let is_ignored = config.is_dir_ignored(&path);
        c +=1;
        let is_last = (c == total);

        tx_io.send(IOEvent::DirectoryStarted(DirectoryInfo {
            is_last,
            is_ignored,
            depth,
            name: dir.path()
        })).unwrap();

        if !is_ignored {
            list_files(&path, config, depth+1, tx_io);
        }
    }
}

pub fn compute(config: &Config, rx_io: &Receiver<IOEvent>, tx_render: &Sender<RenderItem>) {
    let mut uncommited_dirs: VecDeque<DirectoryInfo> = VecDeque::with_capacity(8);
    // let is_filtered_compute = config.is_filtered();

    for event in rx_io.iter() {
        match event {
            IOEvent::DirectoryStarted(ds) => {
                uncommited_dirs.retain(|d| d.depth < ds.depth);
                if !ds.is_ignored {
                    uncommited_dirs.push_back(ds);
                }
            },
            IOEvent::FilesListed(fs) => {
                if !fs.files.iter().any(|f| config.is_file_valid(f.path.as_path())) {
                    continue;
                }

                while let Some(d) = uncommited_dirs.pop_front() {
                    tx_render.send(RenderItem {
                        item: RenderType::Dir(FileRenderItem { path: d.name }),
                        depth: d.depth,
                        is_leaf: (d.is_ignored && d.is_last),
                        is_last: d.is_last
                    }).unwrap();
                }

                let mut c = 0;
                let files_count = fs.files.len();
                for file in fs.files {
                    c += 1;
                    if !config.is_file_valid(file.path.as_path()) {
                        continue;
                    }

                    let is_last = (c == files_count);
                    tx_render.send(RenderItem {
                        item: RenderType::File(FileRenderItem {path: file.path}),
                        depth: fs.depth,
                        is_leaf: true,
                        is_last: false
                    }).unwrap();
                }
            }
        }


    }
}

pub fn render_files(config: &Config, rx_render: Receiver<RenderItem>) {
    let mut renderer = Renderer::new(config);
    for item in rx_render.iter() {
        for _ in 0..item.depth {
            renderer.render_pipe_v();
        }

        if item.is_last && item.is_leaf {
            renderer.render_pipe_e();
        } else {
            renderer.render_pipe_t();
        }

        renderer.render_pipe_h();
        match item.item {
            RenderType::File(f) => renderer.render_file(&f.path),
            RenderType::Dir(d) => renderer.render_dir(&d.path, item.is_leaf),
            RenderType::SkppedFiles(s) => renderer.render_skippedfiles(&s.ext, s.count),
        };
    }
}


