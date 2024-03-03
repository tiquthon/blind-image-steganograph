use std::path::Path;

use image::{DynamicImage};
use webp::{Decoder, Encoder};

#[derive(Clone, PartialEq, Debug)]
pub enum Image {
    Rgba8(u32, u32, Vec<u8>),
}

impl Image {
    pub fn load_from_file<P>(path: P) -> Self where P: AsRef<Path> {
        Self::load_from_file_with_format(path, ImageFormat::WebP)
    }

    pub fn load_from_file_with_format<P>(path: P, format: ImageFormat) -> Self where P: AsRef<Path> {
        match format {
            ImageFormat::WebP => Self::load_from_memory_with_format(&std::fs::read(path).unwrap(), format),
        }
    }

    pub fn load_from_memory(buffer: &[u8]) -> Self {
        Self::load_from_memory_with_format(buffer, ImageFormat::WebP)
    }

    pub fn load_from_memory_with_format(buffer: &[u8], format: ImageFormat) -> Self {
        match format {
            ImageFormat::WebP => match Decoder::new(buffer).decode().unwrap().to_image() {
                DynamicImage::ImageLuma8(_) => todo!(),
                DynamicImage::ImageLumaA8(_) => todo!(),
                DynamicImage::ImageRgb8(_) => todo!(),
                DynamicImage::ImageRgba8(r) => Self::Rgba8(r.width(), r.height(), r.into_vec()),
                DynamicImage::ImageLuma16(_) => todo!(),
                DynamicImage::ImageLumaA16(_) => todo!(),
                DynamicImage::ImageRgb16(_) => todo!(),
                DynamicImage::ImageRgba16(_) => todo!(),
                DynamicImage::ImageRgb32F(_) => todo!(),
                DynamicImage::ImageRgba32F(_) => todo!(),
                _ => todo!(),
            },
        }
    }

    pub fn save_to_file<P>(&self, path: P, format: ImageFormat) where P: AsRef<Path> {
        let data = self.save_to_memory(format);
        std::fs::write(path, &data).unwrap();
    }

    pub fn save_to_memory(&self, format: ImageFormat) -> Vec<u8> {
        match format {
            ImageFormat::WebP => match self {
                Image::Rgba8(w, h, p) => Encoder::from_rgba(p, *w, *h).encode_lossless().to_vec(),
            }
        }
    }

    pub fn insert_data(&mut self, data: &[u8], config: InsertConfig) {
        todo!()
    }

    pub fn extract_data(&self, config: ExtractConfig) -> Vec<u8> {
        todo!()
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[non_exhaustive]
pub enum ImageFormat {
    // TODO: Png,
    // TODO: Gif,
    WebP,
    // TODO: Bmp,
    // TODO: Ico,
}

#[derive(typed_builder::TypedBuilder, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct InsertConfig {

}

#[derive(typed_builder::TypedBuilder, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct ExtractConfig {

}

