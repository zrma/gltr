use glium::texture::SrgbTexture2d;
use glium::Display;

pub fn create_from(
    file_path: String,
    display: &Display,
) -> Result<(SrgbTexture2d, (u32, u32)), Box<dyn std::error::Error>> {
    // Load the images
    let img = image::open(file_path)
        .map_err(|e| e.to_string())?
        .to_rgba8();

    // Get the dimensions and convert the images to RawImage2d
    let dim = img.dimensions();
    let img_data = glium::texture::RawImage2d::from_raw_rgba_reversed(&img.into_raw(), dim);

    let texture = SrgbTexture2d::new(display, img_data)
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

    Ok((texture, dim))
}
