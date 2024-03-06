use image::DynamicImage;
use std::path::Path;

use crate::special_pixel_iterator::{
    PixelChannelCombinatorIteratorExt, PixelChannelSeparatorIteratorExt,
};

mod special_pixel_iterator;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Image {
    Rgb8(Rgb8Image),
    Rgba8(Rgba8Image),
}

impl Image {
    pub fn from_dynamic_image(dynamic_image: DynamicImage) -> Self {
        match dynamic_image {
            DynamicImage::ImageLuma8(_) => unimplemented!(),
            DynamicImage::ImageLumaA8(_) => unimplemented!(),
            DynamicImage::ImageRgb8(rgb8_image) => Self::Rgb8(Rgb8Image {
                width: rgb8_image.width(),
                height: rgb8_image.height(),
                pixel_data: rgb8_image.to_vec(),
            }),
            DynamicImage::ImageRgba8(rgba8_image) => Self::Rgb8(Rgb8Image {
                width: rgba8_image.width(),
                height: rgba8_image.height(),
                pixel_data: rgba8_image.to_vec(),
            }),
            DynamicImage::ImageLuma16(_) => unimplemented!(),
            DynamicImage::ImageLumaA16(_) => unimplemented!(),
            DynamicImage::ImageRgb16(_) => unimplemented!(),
            DynamicImage::ImageRgba16(_) => unimplemented!(),
            DynamicImage::ImageRgb32F(_) => unimplemented!(),
            DynamicImage::ImageRgba32F(_) => unimplemented!(),
            _ => unimplemented!(),
        }
    }

    pub fn load_from_file<P>(path: P) -> Result<Self, LoadFromFileError>
    where
        P: AsRef<Path>,
    {
        #[cfg(feature = "webp")]
        match Self::load_from_file_with_format(path, ImageFormat::WebP) {
            Ok(image) => return Ok(image),
            Err(LoadFromFileWithFormatError::ReadFile(e)) => {
                return Err(LoadFromFileError::ReadFile(e))
            }
            Err(LoadFromFileWithFormatError::LoadFailed(_)) => (),
        }
        Err(LoadFromFileError::UnknownFormat)
    }

    pub fn load_from_file_with_format<P>(
        path: P,
        format: ImageFormat,
    ) -> Result<Self, LoadFromFileWithFormatError>
    where
        P: AsRef<Path>,
    {
        match format {
            #[cfg(feature = "webp")]
            ImageFormat::WebP => Ok(Self::load_from_memory_with_format(
                &std::fs::read(path).map_err(LoadFromFileWithFormatError::ReadFile)?,
                format,
            )
            .map_err(LoadFromFileWithFormatError::LoadFailed)?),
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

    pub fn load_from_memory_with_format(
        buffer: &[u8],
        format: ImageFormat,
    ) -> Result<Self, LoadFromMemoryWithFormatError> {
        match format {
            #[cfg(feature = "webp")]
            ImageFormat::WebP => match webp::Decoder::new(buffer)
                .decode()
                .ok_or(LoadFromMemoryWithFormatError::DecodeWebP)?
                .to_image()
            {
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
                image::DynamicImage::ImageLuma8(_)
                | image::DynamicImage::ImageLumaA8(_)
                | image::DynamicImage::ImageLuma16(_)
                | image::DynamicImage::ImageLumaA16(_)
                | image::DynamicImage::ImageRgb16(_)
                | image::DynamicImage::ImageRgba16(_)
                | image::DynamicImage::ImageRgb32F(_)
                | image::DynamicImage::ImageRgba32F(_) => unreachable!(),
                _ => unimplemented!(),
            },
        }
    }

    pub fn save_to_file<P>(&self, path: P, format: ImageFormat) -> Result<(), SaveToFileError>
    where
        P: AsRef<Path>,
    {
        std::fs::write(
            path,
            self.save_to_memory(format)
                .map_err(SaveToFileError::SaveFailed)?,
        )
        .map_err(SaveToFileError::WriteFile)
    }

    pub fn save_to_memory(&self, format: ImageFormat) -> Result<Vec<u8>, SaveToMemoryError> {
        match format {
            #[cfg(feature = "webp")]
            ImageFormat::WebP => match self {
                Image::Rgb8(rgb8_image) => Ok(webp::Encoder::from_rgb(
                    &rgb8_image.pixel_data,
                    rgb8_image.width,
                    rgb8_image.height,
                )
                .encode_lossless()
                .to_vec()),
                Image::Rgba8(rgba8_image) => Ok(webp::Encoder::from_rgb(
                    &rgba8_image.pixel_data,
                    rgba8_image.width,
                    rgba8_image.height,
                )
                .encode_lossless()
                .to_vec()),
            },
        }
    }

    pub fn max_data_capacity_with_config(&self, config: &InsertConfig) -> usize {
        let usable_bits_per_pixel_color_channel_in_red: usize =
            config.count_of_least_significant_bits_in_red.bit_count();
        let usable_bits_per_pixel_color_channel_in_green: usize =
            config.count_of_least_significant_bits_in_green.bit_count();
        let usable_bits_per_pixel_color_channel_in_blue: usize =
            config.count_of_least_significant_bits_in_blue.bit_count();
        let (pixels, usable_bits_per_pixel) = match self {
            Image::Rgb8(rgb8_image) => (
                rgb8_image.width as usize * rgb8_image.height as usize,
                usable_bits_per_pixel_color_channel_in_red
                    + usable_bits_per_pixel_color_channel_in_green
                    + usable_bits_per_pixel_color_channel_in_blue,
            ),
            Image::Rgba8(rgba8_image) => {
                let usable_bits_per_pixel_color_channel_in_alpha: usize =
                    config.count_of_least_significant_bits_in_alpha.bit_count();
                (
                    rgba8_image.width as usize * rgba8_image.height as usize,
                    usable_bits_per_pixel_color_channel_in_red
                        + usable_bits_per_pixel_color_channel_in_green
                        + usable_bits_per_pixel_color_channel_in_blue
                        + usable_bits_per_pixel_color_channel_in_alpha,
                )
            }
        };
        let needed_space_for_length_meta_information = 128;
        let usable_bits = usable_bits_per_pixel * pixels - needed_space_for_length_meta_information;
        usable_bits / 8
    }

    pub fn min_pixels_needed_with_config(
        data_size: usize,
        config: &InsertConfig,
        has_alpha_channel: bool,
    ) -> usize {
        let usable_bits_per_pixel_color_channel_in_red: usize =
            config.count_of_least_significant_bits_in_red.bit_count();
        let usable_bits_per_pixel_color_channel_in_green: usize =
            config.count_of_least_significant_bits_in_green.bit_count();
        let usable_bits_per_pixel_color_channel_in_blue: usize =
            config.count_of_least_significant_bits_in_blue.bit_count();
        let usable_bits_per_pixel = if has_alpha_channel {
            let usable_bits_per_pixel_color_channel_in_alpha: usize =
                config.count_of_least_significant_bits_in_alpha.bit_count();
            usable_bits_per_pixel_color_channel_in_red
                + usable_bits_per_pixel_color_channel_in_green
                + usable_bits_per_pixel_color_channel_in_blue
                + usable_bits_per_pixel_color_channel_in_alpha
        } else {
            usable_bits_per_pixel_color_channel_in_red
                + usable_bits_per_pixel_color_channel_in_green
                + usable_bits_per_pixel_color_channel_in_blue
        };
        let needed_space_for_length_meta_information = 128;
        (((data_size * 8) + needed_space_for_length_meta_information) as f64
            / usable_bits_per_pixel as f64)
            .ceil() as usize
    }

    pub fn insert_data(
        &mut self,
        data: &[u8],
        config: &InsertConfig,
    ) -> Result<(), InsertDataError> {
        let max_capacity = self.max_data_capacity_with_config(config);
        if data.len() > max_capacity {
            return Err(InsertDataError::DataExceedsCapacity {
                data: data.len(),
                capacity: max_capacity,
            });
        }

        let length: [u8; 16] = (data.len() as u128).to_be_bytes();

        let insert_data = length.iter().chain(data.iter());

        match self {
            Image::Rgb8(rgb8_image) => {
                insert_data
                    .separate_pixel_channel_rgb(
                        config.count_of_least_significant_bits_in_red,
                        config.count_of_least_significant_bits_in_green,
                        config.count_of_least_significant_bits_in_blue,
                        Some(rgb8_image.width as usize * rgb8_image.height as usize),
                        config.remaining_bits_action,
                    )
                    .zip(rgb8_image.pixel_data.iter_mut())
                    .for_each(|((bits, bit_mask), pixel_channel_data)| {
                        *pixel_channel_data = (*pixel_channel_data & !bit_mask) | bits
                    });
            }
            Image::Rgba8(rgba8_image) => {
                insert_data
                    .separate_pixel_channel_rgba(
                        config.count_of_least_significant_bits_in_red,
                        config.count_of_least_significant_bits_in_green,
                        config.count_of_least_significant_bits_in_blue,
                        config.count_of_least_significant_bits_in_alpha,
                        Some(rgba8_image.width as usize * rgba8_image.height as usize),
                        config.remaining_bits_action,
                    )
                    .zip(rgba8_image.pixel_data.iter_mut())
                    .for_each(|((bits, bit_mask), pixel_channel_data)| {
                        *pixel_channel_data = (*pixel_channel_data & !bit_mask) | bits
                    });
            }
        }

        Ok(())
    }

    pub fn extract_data(&self, config: &ExtractConfig) -> Result<Vec<u8>, ExtractDataError> {
        match self {
            Image::Rgb8(rgb8_image) => {
                let mut extract_data = rgb8_image.pixel_data.iter().combine_pixel_channel_rgb(
                    config.count_of_least_significant_bits_in_red,
                    config.count_of_least_significant_bits_in_green,
                    config.count_of_least_significant_bits_in_blue,
                );
                let mut length = [0u8; 16];
                for (index, length_cell) in length.iter_mut().enumerate() {
                    *length_cell = extract_data
                        .next()
                        .ok_or(ExtractDataError::MissingByteForLength { index })?;
                }
                let length = i128::from_be_bytes(length) as usize;
                let mut output = Vec::with_capacity(length);
                for index in 0..length {
                    output.push(
                        extract_data
                            .next()
                            .ok_or(ExtractDataError::MissingDataBytes { index, length })?,
                    );
                }
                Ok(output)
            }
            Image::Rgba8(rgba8_image) => {
                let mut extract_data = rgba8_image.pixel_data.iter().combine_pixel_channel_rgba(
                    config.count_of_least_significant_bits_in_red,
                    config.count_of_least_significant_bits_in_green,
                    config.count_of_least_significant_bits_in_blue,
                    config.count_of_least_significant_bits_in_alpha,
                );
                let mut length = [0u8; 16];
                for (index, length_cell) in length.iter_mut().enumerate() {
                    *length_cell = extract_data
                        .next()
                        .ok_or(ExtractDataError::MissingByteForLength { index })?;
                }
                let length = i128::from_be_bytes(length) as usize;
                let mut output = Vec::with_capacity(length);
                for index in 0..length {
                    output.push(
                        extract_data
                            .next()
                            .ok_or(ExtractDataError::MissingDataBytes { index, length })?,
                    );
                }
                Ok(output)
            }
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
    #[error("Could not read file: {0}")]
    ReadFile(#[source] std::io::Error),
    #[error("Unknown file format")]
    UnknownFormat,
}

#[derive(thiserror::Error, Debug)]
pub enum LoadFromFileWithFormatError {
    #[error("Could not read file: {0}")]
    ReadFile(#[source] std::io::Error),
    #[error("Loading file failed: {0}")]
    LoadFailed(#[source] LoadFromMemoryWithFormatError),
}

#[derive(thiserror::Error, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum LoadFromMemoryError {
    #[error("Unknown format")]
    UnknownFormat,
}

#[derive(thiserror::Error, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum LoadFromMemoryWithFormatError {
    #[error("Could not decode WebP data")]
    DecodeWebP,
}

#[derive(thiserror::Error, Debug)]
pub enum SaveToFileError {
    #[error("Saving file failed: {0}")]
    SaveFailed(#[source] SaveToMemoryError),
    #[error("Could not save file: {0}")]
    WriteFile(#[source] std::io::Error),
}

#[derive(thiserror::Error, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum SaveToMemoryError {}

#[derive(thiserror::Error, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum InsertDataError {
    #[error("Data with size '{data}' exceeds maximum available capacity of '{capacity}'")]
    DataExceedsCapacity { data: usize, capacity: usize },
}

#[derive(thiserror::Error, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ExtractDataError {
    #[error("Missing byte at index {index} for length information")]
    MissingByteForLength { index: usize },
    #[error("Missing data byte at index {index} and the remaining of full length of {length}")]
    MissingDataBytes { index: usize, length: usize },
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

#[derive(typed_builder::TypedBuilder, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct InsertConfig {
    #[builder(default=CountOfLeastSignificantBits::One)]
    pub count_of_least_significant_bits_in_red: CountOfLeastSignificantBits,
    #[builder(default=CountOfLeastSignificantBits::One)]
    pub count_of_least_significant_bits_in_green: CountOfLeastSignificantBits,
    #[builder(default=CountOfLeastSignificantBits::One)]
    pub count_of_least_significant_bits_in_blue: CountOfLeastSignificantBits,
    #[builder(default=CountOfLeastSignificantBits::Zero)]
    pub count_of_least_significant_bits_in_alpha: CountOfLeastSignificantBits,
    #[builder(default)]
    pub remaining_bits_action: RemainingBitsAction,
}

#[derive(typed_builder::TypedBuilder, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct ExtractConfig {
    #[builder(default=CountOfLeastSignificantBits::One)]
    pub count_of_least_significant_bits_in_red: CountOfLeastSignificantBits,
    #[builder(default=CountOfLeastSignificantBits::One)]
    pub count_of_least_significant_bits_in_green: CountOfLeastSignificantBits,
    #[builder(default=CountOfLeastSignificantBits::One)]
    pub count_of_least_significant_bits_in_blue: CountOfLeastSignificantBits,
    #[builder(default=CountOfLeastSignificantBits::Zero)]
    pub count_of_least_significant_bits_in_alpha: CountOfLeastSignificantBits,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[cfg_attr(test, derive(strum::EnumIter))]
pub enum CountOfLeastSignificantBits {
    Zero,
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
    #[error("'{0}' is not valid for count of least significant bits")]
    Unknown(usize),
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub enum RemainingBitsAction {
    #[default]
    None,
    Zero,
    #[cfg(feature = "random")]
    Randomize {
        seed: Option<u64>,
    },
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
