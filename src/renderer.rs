use std::io::{BufWriter, StdoutLock, Write};
use std::path::PathBuf;
use crate::hex_to_color;
use crate::settings::Config;

pub struct Renderer<'a, 'b> {
    pub config: &'a Config,
    pub writer: BufWriter<StdoutLock<'b>>
}

impl<'a, 'b> Renderer<'a, 'b> {
    pub(crate) fn new(config: &'a Config) -> Self {
        let stdout = std::io::stdout();
        let writer = BufWriter::new(stdout.lock());
        Renderer {config, writer}
    }

    pub fn render_file(&mut self, path: &PathBuf) {
        let filename_os = path.file_name().unwrap();
        let filename = filename_os.to_str().unwrap();

        let glyph = self.config.get_associated_file_glyph(filename);
        let color = self.config.get_associated_file_color(filename);
        let style = hex_to_color(color).normal();

        write!(&mut self.writer, "{} {}\n", style.paint(glyph), style.paint(filename)).unwrap();
    }

    pub fn render_pipe_v(&mut self) {
        write!(&mut self.writer, "{} ", self.config.glyphs.get("pipe-v").unwrap()).unwrap();
    }

    pub fn render_pipe_h(&mut self) {
        write!(&mut self.writer, "{}", self.config.glyphs.get("pipe-h").unwrap()).unwrap();
    }

    pub fn render_pipe_t(&mut self) {
        write!(&mut self.writer, "{}", self.config.glyphs.get("pipe-t").unwrap()).unwrap();
    }

    pub fn render_pipe_e(&mut self) {
        write!(&mut self.writer, "{} ", self.config.glyphs.get("pipe-e").unwrap()).unwrap();
    }

    pub fn render_dir(&mut self, path: &PathBuf, ignored: bool) {
        let filename_os = path.file_name().unwrap();
        let filename = filename_os.to_str().unwrap();

        let glyph = self.config.get_associated_dir_glyph(filename);

        if ignored {
            let color = &self.config.colors.directories.ignored;
            let style = hex_to_color(color).normal();
            write!(&mut self.writer, "{} {}{}\n", style.paint(glyph), style.paint(filename), style.paint("/...")).unwrap();

        } else {
            let color = self.config.get_associated_dir_color(filename);
            let style = hex_to_color(color).normal();
            write!(&mut self.writer, "{} {}\n", style.paint(glyph), style.paint(filename)).unwrap();
        }
    }

    pub fn render_skippedfiles(&mut self, ext: &String, count: i32) {
        let glyph = self.config.get_associated_ext_glyph(ext);

        let color = self.config.get_associated_ext_color(ext);
        let style = hex_to_color(color).normal();

        let value = format!("{} {} files...", count, ext);
        write!(&mut self.writer, "{} {}\n", style.paint(glyph), style.paint(value)).unwrap();
    }
}
