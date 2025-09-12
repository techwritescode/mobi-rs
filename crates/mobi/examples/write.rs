use image::imageops::FilterType;
use image::{GenericImageView, ImageFormat};
use mobi::mobi_writer::MobiWriter;
use std::io::Cursor;

fn main() -> anyhow::Result<()> {
    // let mut mobi = MOBI::new("Perfect World");
    // mobi.set_content("<html><head></head><body><p>Test</p></body></html>");
    //
    // std::fs::write("output.mobi", &mobi.to_bytes()?)?;
    let mut html = "<html><head></head><body>".to_owned();
    let mut writer = MobiWriter::new("Perfect World".to_owned());
    for (i, image) in std::fs::read_dir("download/b21864d6-d661-4702-a86e-4efd5ebdaecb")?.enumerate() {
        let img = image::open(image.unwrap().path())?;

        let (width, height) = img.dimensions();
        let max_width = 600;
        let max_height = 800;

        let scale_w = max_width as f32 / width as f32;
        let scale_h = max_height as f32 / height as f32;

        let scale = scale_w.min(scale_h);

        let new_width = (width as f32 * scale).round() as u32;
        let new_height = (height as f32 * scale).round() as u32;

        let img = img.resize(new_width, new_height, FilterType::Lanczos3).grayscale();

        let mut buf = Cursor::new(Vec::new());
        img.write_to(&mut buf, ImageFormat::Jpeg)?;

        writer.add_image(buf.into_inner());
        eprintln!("Writing page {i}");
        // let new_width = 600;
        // let new_height = 800;
        html += format!("<p height=\"0pt\" width=\"0pt\" align=\"center\"><img recindex=\"{:05}\" align=\"baseline\" width=\"{}\" height=\"{}\"></img></p><mbp:pagebreak/>", i+1, new_width, new_height).as_str();
    }
    html += "</body></html>";
    writer.set_content(html);
    std::fs::write("test.mobi", writer.to_bytes().expect("Failed to write"))
        .expect("Failed to save");
    Ok(())
}
