mod special_pixel_iterator;

use std::path::Path;
use crate::special_pixel_iterator::{PixelChannelCombinatorIteratorExt, PixelChannelSeparatorIteratorExt};

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Image {
    Rgb8(Rgb8Image),
    Rgba8(Rgba8Image),
}

impl Image {
    pub fn load_from_file<P>(path: P) -> Result<Self, LoadFromFileError> where P: AsRef<Path> {
        #[cfg(feature = "webp")]
        match Self::load_from_file_with_format(path, ImageFormat::WebP) {
            Ok(image) => return Ok(image),
            Err(LoadFromFileWithFormatError::ReadFile(e)) => return Err(LoadFromFileError::ReadFile(e)),
            Err(LoadFromFileWithFormatError::LoadFailed(_)) => (),
        }
        Err(LoadFromFileError::UnknownFormat)
    }

    pub fn load_from_file_with_format<P>(path: P, format: ImageFormat) -> Result<Self, LoadFromFileWithFormatError> where P: AsRef<Path> {
        match format {
            #[cfg(feature = "webp")]
            ImageFormat::WebP => Ok(Self::load_from_memory_with_format(
                &std::fs::read(path).map_err(LoadFromFileWithFormatError::ReadFile)?, format
            ).map_err(LoadFromFileWithFormatError::LoadFailed)?),
        }
    }

    pub fn load_from_memory(buffer: &[u8]) -> Result<Self, LoadFromMemoryError> {
        #[cfg(feature = "webp")]
        match Self::load_from_memory_with_format(buffer, ImageFormat::WebP) {
            Ok(image) => return Ok(image),
            Err(LoadFromMemoryWithFormatError::DecodeWebP) => (),
        }
        Err(LoadFromMemoryError::UnknownFormat)
    }

    pub fn load_from_memory_with_format(buffer: &[u8], format: ImageFormat) -> Result<Self, LoadFromMemoryWithFormatError> {
        match format {
            #[cfg(feature = "webp")]
            ImageFormat::WebP => match webp::Decoder::new(buffer).decode().ok_or(LoadFromMemoryWithFormatError::DecodeWebP)?.to_image() {
                image::DynamicImage::ImageRgb8(r) => Ok(Self::Rgb8(Rgb8Image {
                    width: r.width(),
                    height: r.height(),
                    pixel_data: r.into_vec(),
                })),
                image::DynamicImage::ImageRgba8(r) => Ok(Self::Rgba8(Rgba8Image {
                    width: r.width(),
                    height: r.height(),
                    pixel_data: r.into_vec(),
                })),
                image::DynamicImage::ImageLuma8(_) |image::DynamicImage::ImageLumaA8(_) |image::DynamicImage::ImageLuma16(_) |
                image::DynamicImage::ImageLumaA16(_) |image::DynamicImage::ImageRgb16(_) |image::DynamicImage::ImageRgba16(_) |
                image::DynamicImage::ImageRgb32F(_) |image::DynamicImage::ImageRgba32F(_) => unreachable!(),
                _ => unimplemented!(),
            },
        }
    }

    pub fn save_to_file<P>(&self, path: P, format: ImageFormat) where P: AsRef<Path> {
        std::fs::write(path, &self.save_to_memory(format)).unwrap();
    }

    pub fn save_to_memory(&self, format: ImageFormat) -> Vec<u8> {
        match format {
            #[cfg(feature = "webp")]
            ImageFormat::WebP => match self {
                Image::Rgb8(rgb8_image) => webp::Encoder::from_rgb(&rgb8_image.pixel_data, rgb8_image.width, rgb8_image.height)
                    .encode_lossless()
                    .to_vec(),
                Image::Rgba8(rgba8_image) => webp::Encoder::from_rgb(&rgba8_image.pixel_data, rgba8_image.width, rgba8_image.height)
                    .encode_lossless()
                    .to_vec(),
            }
        }
    }

    pub fn max_data_capacity_with_config(&self, config: &InsertConfig) -> usize {
        let usable_bits_per_pixel_color_channel_in_red: usize = config.count_of_least_significant_bits_in_red.bit_count();
        let usable_bits_per_pixel_color_channel_in_green: usize = config.count_of_least_significant_bits_in_green.bit_count();
        let usable_bits_per_pixel_color_channel_in_blue: usize = config.count_of_least_significant_bits_in_blue.bit_count();
        let usable_bits_per_pixel = usable_bits_per_pixel_color_channel_in_red as usize + usable_bits_per_pixel_color_channel_in_green as usize + usable_bits_per_pixel_color_channel_in_blue as usize;
        let pixels = match self {
            Image::Rgb8(rgb8_image) => rgb8_image.width as usize * rgb8_image.height as usize,
            Image::Rgba8(rgba8_image) => rgba8_image.width as usize * rgba8_image.height as usize,
        };
        let needed_space_for_length_meta_information = 128;
        let usable_bits = usable_bits_per_pixel * pixels - needed_space_for_length_meta_information;
        usable_bits / 8
    }

    pub fn insert_data(&mut self, data: &[u8], config: &InsertConfig) -> Result<(), InsertDataError> {
        let max_capacity = self.max_data_capacity_with_config(config);
        if data.len() > max_capacity {
            return Err(InsertDataError::DataExceedsCapacity);
        }

        let length: [u8; 16] = (data.len() as u128).to_be_bytes();

        let insert_data = length.iter().chain(data.iter());

        match self {
            Image::Rgb8(rgb8_image) => {
                let insert_data = insert_data
                    .separate_pixel_channel_rgb(
                        config.count_of_least_significant_bits_in_red,
                        config.count_of_least_significant_bits_in_green,
                        config.count_of_least_significant_bits_in_blue
                    )
                    .zip(rgb8_image.pixel_data.iter_mut());
                for ((bits, bit_mask), pixel_channel_data) in insert_data {
                    *pixel_channel_data = (*pixel_channel_data & !bit_mask) | bits;
                }
            },
            Image::Rgba8(rgba8_image) => {
                let insert_data = insert_data
                    .separate_pixel_channel_rgba(
                        config.count_of_least_significant_bits_in_red,
                        config.count_of_least_significant_bits_in_green,
                        config.count_of_least_significant_bits_in_blue,
                        config.count_of_least_significant_bits_in_alpha
                    )
                    .zip(rgba8_image.pixel_data.iter_mut());
                for ((bits, bit_mask), pixel_channel_data) in insert_data {
                    *pixel_channel_data = (*pixel_channel_data & !bit_mask) | bits;
                }
            },
        }

        Ok(())
    }

    pub fn extract_data(&self, config: &ExtractConfig) -> Vec<u8> {
        match self {
            Image::Rgb8(rgb8_image) => {
                let mut m = rgb8_image.pixel_data.iter()
                    .combine_pixel_channel_rgb(
                        config.count_of_least_significant_bits_in_red,
                        config.count_of_least_significant_bits_in_green,
                        config.count_of_least_significant_bits_in_blue,
                    );
                let length = i128::from_be_bytes([
                    m.next().unwrap(),
                    m.next().unwrap(),
                    m.next().unwrap(),
                    m.next().unwrap(),

                    m.next().unwrap(),
                    m.next().unwrap(),
                    m.next().unwrap(),
                    m.next().unwrap(),

                    m.next().unwrap(),
                    m.next().unwrap(),
                    m.next().unwrap(),
                    m.next().unwrap(),

                    m.next().unwrap(),
                    m.next().unwrap(),
                    m.next().unwrap(),
                    m.next().unwrap(),
                ]) as usize;
                m.take(length).collect::<Vec<_>>()
            }
            Image::Rgba8(rgba8_image) => {
                let mut m = rgba8_image.pixel_data.iter()
                    .combine_pixel_channel_rgba(
                        config.count_of_least_significant_bits_in_red,
                        config.count_of_least_significant_bits_in_green,
                        config.count_of_least_significant_bits_in_blue,
                        config.count_of_least_significant_bits_in_alpha,
                    );
                let length = i128::from_be_bytes([
                    m.next().unwrap(),
                    m.next().unwrap(),
                    m.next().unwrap(),
                    m.next().unwrap(),

                    m.next().unwrap(),
                    m.next().unwrap(),
                    m.next().unwrap(),
                    m.next().unwrap(),

                    m.next().unwrap(),
                    m.next().unwrap(),
                    m.next().unwrap(),
                    m.next().unwrap(),

                    m.next().unwrap(),
                    m.next().unwrap(),
                    m.next().unwrap(),
                    m.next().unwrap(),
                ]) as usize;
                m.take(length).collect::<Vec<_>>()
            },
        }
    }
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Rgb8Image {
    width: u32,
    height: u32,
    pixel_data: Vec<u8>,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Rgba8Image {
    width: u32,
    height: u32,
    pixel_data: Vec<u8>,
}

#[derive(thiserror::Error, Debug)]
pub enum LoadFromFileError {
    #[error("")]
    ReadFile(#[source] std::io::Error),
    #[error("")]
    UnknownFormat,
}

#[derive(thiserror::Error, Debug)]
pub enum LoadFromFileWithFormatError {
    #[error("")]
    ReadFile(#[source] std::io::Error),
    #[error("")]
    LoadFailed(#[source] LoadFromMemoryWithFormatError),
}

#[derive(thiserror::Error, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum LoadFromMemoryError {
    #[error("")]
    UnknownFormat,
}

#[derive(thiserror::Error, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum LoadFromMemoryWithFormatError {
    #[error("")]
    DecodeWebP,
}

#[derive(thiserror::Error, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum InsertDataError {
    #[error("")]
    DataExceedsCapacity,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[non_exhaustive]
pub enum ImageFormat {
    // TODO: Png,
    // TODO: Gif,
    #[cfg(feature = "webp")]
    WebP,
    // TODO: Bmp,
    // TODO: Ico,
}

#[derive(typed_builder::TypedBuilder, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct InsertConfig {
    #[builder(default)]
    pub count_of_least_significant_bits_in_red: CountOfLeastSignificantBits,
    #[builder(default)]
    pub count_of_least_significant_bits_in_green: CountOfLeastSignificantBits,
    #[builder(default)]
    pub count_of_least_significant_bits_in_blue: CountOfLeastSignificantBits,
    #[builder(default)]
    pub count_of_least_significant_bits_in_alpha: CountOfLeastSignificantBits,
    #[builder(default)]
    pub remaining_bits_action: RemainingBitsAction,
}

#[derive(typed_builder::TypedBuilder, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct ExtractConfig {
    #[builder(default)]
    pub count_of_least_significant_bits_in_red: CountOfLeastSignificantBits,
    #[builder(default)]
    pub count_of_least_significant_bits_in_green: CountOfLeastSignificantBits,
    #[builder(default)]
    pub count_of_least_significant_bits_in_blue: CountOfLeastSignificantBits,
    #[builder(default)]
    pub count_of_least_significant_bits_in_alpha: CountOfLeastSignificantBits,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
#[cfg_attr(test, derive(strum::EnumIter))]
pub enum CountOfLeastSignificantBits {
    Zero,
    #[default]
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
}

impl CountOfLeastSignificantBits {
    pub fn bit_count(self) -> usize {
        match self {
            CountOfLeastSignificantBits::Zero => 0,
            CountOfLeastSignificantBits::One => 1,
            CountOfLeastSignificantBits::Two => 2,
            CountOfLeastSignificantBits::Three => 3,
            CountOfLeastSignificantBits::Four => 4,
            CountOfLeastSignificantBits::Five => 5,
            CountOfLeastSignificantBits::Six => 6,
            CountOfLeastSignificantBits::Seven => 7,
            CountOfLeastSignificantBits::Eight => 8,
        }
    }

    pub fn try_from_bit_count(bit_count: usize) -> Result<Self, TryFromBitCountError> {
        match bit_count {
            0 => Ok(Self::Zero),
            1 => Ok(Self::One),
            2 => Ok(Self::Two),
            3 => Ok(Self::Three),
            4 => Ok(Self::Four),
            5 => Ok(Self::Five),
            6 => Ok(Self::Six),
            7 => Ok(Self::Seven),
            8 => Ok(Self::Eight),
            _ => Err(TryFromBitCountError::Unknown(bit_count)),
        }
    }
}

#[derive(thiserror::Error, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum TryFromBitCountError {
    #[error("")]
    Unknown(usize),
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub enum RemainingBitsAction {
    #[default]
    None,
    Zero,
    #[cfg(feature = "random")]
    Randomize,
}

#[cfg(test)]
mod tests {
    use strum::IntoEnumIterator;

    use super::*;

    #[test]
    fn count_of_least_significant_bits_conversion_completely_works() {
        // Arrange
        for count_of_least_significant_bits in CountOfLeastSignificantBits::iter() {
            // Act
            let encoded: usize = count_of_least_significant_bits.bit_count();
            let decoded = CountOfLeastSignificantBits::try_from_bit_count(encoded);

            // Assert
            assert_eq!(Ok(count_of_least_significant_bits), decoded);
        }
    }
}
