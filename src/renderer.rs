use std::path::PathBuf;
use flume::Receiver;
use crate::{hex_to_color, RenderItem, RenderType};
use crate::settings::Config;


#[derive(Clone)]
pub struct Renderer<'a> {
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
