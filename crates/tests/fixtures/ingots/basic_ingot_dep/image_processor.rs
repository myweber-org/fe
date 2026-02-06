
use std::path::Path;
use image::{DynamicImage, ImageFormat};
use std::fs::File;
use std::io::BufWriter;

pub struct ImageProcessor {
    width: u32,
    height: u32,
    maintain_aspect: bool,
}

impl ImageProcessor {
    pub fn new(width: u32, height: u32) -> Self {
        ImageProcessor {
            width,
            height,
            maintain_aspect: true,
        }
    }

    pub fn with_aspect_ratio(mut self, maintain: bool) -> Self {
        self.maintain_aspect = maintain;
        self
    }

    pub fn resize(&self, input_path: &str, output_path: &str) -> Result<(), String> {
        let path = Path::new(input_path);
        if !path.exists() {
            return Err(format!("Input file does not exist: {}", input_path));
        }

        let img = image::open(path)
            .map_err(|e| format!("Failed to open image: {}", e))?;

        let resized = if self.maintain_aspect {
            img.resize(self.width, self.height, image::imageops::FilterType::Lanczos3)
        } else {
            img.resize_exact(self.width, self.height, image::imageops::FilterType::Lanczos3)
        };

        let output_format = self.detect_format(output_path);
        let output_file = File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;

        let writer = BufWriter::new(output_file);
        resized.write_to(writer, output_format)
            .map_err(|e| format!("Failed to write image: {}", e))?;

        Ok(())
    }

    fn detect_format(&self, path: &str) -> ImageFormat {
        let extension = Path::new(path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        match extension.as_str() {
            "jpg" | "jpeg" => ImageFormat::Jpeg,
            "png" => ImageFormat::Png,
            "gif" => ImageFormat::Gif,
            "bmp" => ImageFormat::Bmp,
            "webp" => ImageFormat::WebP,
            _ => ImageFormat::Png,
        }
    }

    pub fn generate_thumbnail(&self, input_path: &str) -> Result<DynamicImage, String> {
        let path = Path::new(input_path);
        if !path.exists() {
            return Err(format!("Input file does not exist: {}", input_path));
        }

        let img = image::open(path)
            .map_err(|e| format!("Failed to open image: {}", e))?;

        let thumbnail = img.thumbnail(self.width, self.height);
        Ok(thumbnail)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_image_processor_creation() {
        let processor = ImageProcessor::new(800, 600);
        assert_eq!(processor.width, 800);
        assert_eq!(processor.height, 600);
        assert!(processor.maintain_aspect);
    }

    #[test]
    fn test_aspect_ratio_configuration() {
        let processor = ImageProcessor::new(800, 600).with_aspect_ratio(false);
        assert!(!processor.maintain_aspect);
    }
}