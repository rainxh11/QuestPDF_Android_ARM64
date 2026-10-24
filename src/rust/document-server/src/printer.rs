//! ThermalReceiptPrinter: 150 DPI, maximum 576 dots, ESC/POS raster buffer.
use anyhow::{Result, bail};
use std::{
    io::Write,
    net::{TcpStream, ToSocketAddrs},
    time::Duration,
};
pub fn receipt_bytes(document: &questpdf::Document) -> Result<Vec<u8>> {
    questpdf::Settings::set_license(questpdf::LicenseType::Community);
    let mut renderer = questpdf::Renderer::new()?;
    renderer.register_font_file(crate::documents::assets().join("BAHNSCHRIFT.TTF"))?;
    let images = renderer.generate_images(document, 150.)?;
    if images.len() != 1 {
        bail!("A thermal receipt must produce exactly one raster image")
    }
    encode_image(&images[0])
}
pub fn encode_image(png: &[u8]) -> Result<Vec<u8>> {
    let image = image::load_from_memory(png)?.to_rgba8();
    let image = crate::resample::resize(&image, 576);
    let gray = image::GrayImage::from_fn(image.width(), image.height(), |x, y| {
        let p = image.get_pixel(x, y);
        let value = ((p[0] as f32 * (1. / 255.)) * 0.2126 + (p[1] as f32 * (1. / 255.)) * 0.7152)
            + (p[2] as f32 * (1. / 255.)) * 0.0722;
        image::Luma([((value.clamp(0., 1.) * 255.) + 0.5) as u8])
    });
    let (width, height) = gray.dimensions();
    let row_bytes = width.div_ceil(8) as usize;
    let mut raster = vec![0u8; row_bytes * height as usize];
    let mut values: Vec<f32> = gray.as_raw().iter().map(|v| *v as f32).collect();
    let kernel = [
        (1i32, 0i32, 8f32),
        (2, 0, 4.),
        (-2, 1, 2.),
        (-1, 1, 4.),
        (0, 1, 8.),
        (1, 1, 4.),
        (2, 1, 2.),
        (-2, 2, 1.),
        (-1, 2, 2.),
        (0, 2, 4.),
        (1, 2, 2.),
        (2, 2, 1.),
    ];
    for y in 0..height {
        for x in 0..width {
            let old = values[(y * width + x) as usize];
            let new = if old < 127.5 { 0. } else { 255. };
            if new == 0. {
                raster[y as usize * row_bytes + x as usize / 8] |= 0x80 >> (x % 8)
            }
            let error = old * (1. / 255.) - new * (1. / 255.);
            for (dx, dy, weight) in kernel {
                let xx = x as i32 + dx;
                let yy = y as i32 + dy;
                if xx >= 0 && yy >= 0 && xx < width as i32 && yy < height as i32 {
                    let i = yy as usize * width as usize + xx as usize;
                    let value = values[i] * (1. / 255.) + error * (weight / 42.);
                    values[i] = ((value.clamp(0., 1.) * 255.) + 0.5) as u8 as f32;
                }
            }
        }
    }
    // Source BufferImage(isLegacy: true) uses GS v 0, followed by the same
    // density, buffer-print, feed and cut commands as ThermalReceiptPrinter.
    let mut data = vec![0x1b, 0x61, 0];
    data.extend_from_slice(&[0x1d, 0x76, 0x30, 0x00]);
    data.extend_from_slice(&(row_bytes as u16).to_le_bytes());
    data.extend_from_slice(&(height as u16).to_le_bytes());
    data.extend(raster);
    data.extend_from_slice(&[0x1d, 0x28, 0x4c, 4, 0, 0x30, 0x31, 0x33, 0x33]);
    data.extend_from_slice(&[
        0x1d, 0x28, 0x4c, 0x02, 0x00, 0x30, 0x32, 0x1b, 0x64, 6, 0x1d, 0x56, 0,
    ]);
    Ok(data)
}
pub fn print_receipt(document: &questpdf::Document, host: &str, port: u16) -> Result<()> {
    if host.trim().is_empty() || port == 0 {
        bail!("Printer host and valid port are required")
    }
    let data = receipt_bytes(document)?;
    let mut last = None;
    for address in (host, port).to_socket_addrs()? {
        match TcpStream::connect_timeout(&address, Duration::from_secs(15)) {
            Ok(mut stream) => {
                stream.set_write_timeout(Some(Duration::from_secs(15)))?;
                stream.write_all(&data)?;
                return Ok(());
            }
            Err(e) => last = Some(e),
        }
    }
    if let Some(e) = last {
        Err(e.into())
    } else {
        bail!("Printer host resolved to no addresses")
    }
}
