use std::path::PathBuf;
use blind_image_steganography::{CountOfLeastSignificantBits, ExtractConfig, Image, ImageFormat, InsertConfig};
use clap::Parser;
use image::DynamicImage;
use image::imageops::FilterType;

fn main() {
    let args = Args::parse();

    match args.command {
        Command::Insert { input_image_file, input_data_file, output_image_file, remaining_bits_action, count_of_least_significant_bits, shrink_to_fit_minimum_pixels, grow_to_fit_maximum_pixels } => {
            let input_data = std::fs::read(input_data_file).unwrap();

            let count_lsb = CountOfLeastSignificantBits::try_from_bit_count(count_of_least_significant_bits.into()).unwrap();
            let insert_config = InsertConfig::builder()
                .remaining_bits_action(remaining_bits_action.into())
                .count_of_least_significant_bits_in_red(count_lsb)
                .count_of_least_significant_bits_in_green(count_lsb)
                .count_of_least_significant_bits_in_blue(count_lsb)
                .count_of_least_significant_bits_in_alpha(CountOfLeastSignificantBits::Zero)
                .build();

            let mut image = image::open(input_image_file).unwrap();
            let has_alpha_channel = match image {
                DynamicImage::ImageLuma8(_) => false,
                DynamicImage::ImageLumaA8(_) => true,
                DynamicImage::ImageRgb8(_) => false,
                DynamicImage::ImageRgba8(_) => true,
                DynamicImage::ImageLuma16(_) => false,
                DynamicImage::ImageLumaA16(_) => true,
                DynamicImage::ImageRgb16(_) => false,
                DynamicImage::ImageRgba16(_) => true,
                DynamicImage::ImageRgb32F(_) => false,
                DynamicImage::ImageRgba32F(_) => true,
                _ => unimplemented!(),
            };
            let current_pixels = image.width() as usize * image.height() as usize;
            let min_pixels = Image::min_pixels_needed_with_config(input_data.len(), &insert_config, has_alpha_channel);

            if current_pixels > min_pixels {
                if let Some(shrink_to_fit_minimum_pixels) = shrink_to_fit_minimum_pixels {
                    if current_pixels > shrink_to_fit_minimum_pixels {
                        let factor = (min_pixels.max(shrink_to_fit_minimum_pixels) as f64 / current_pixels as f64).sqrt();
                        let new_width = (image.width() as f64 * factor).ceil() as u32;
                        let new_height = (image.height() as f64 * factor).ceil() as u32;
                        image = image.resize_to_fill(new_width, new_height, FilterType::Gaussian);
                    }
                }
            } else if current_pixels < min_pixels {
                if let Some(grow_to_fit_maximum_pixels) = grow_to_fit_maximum_pixels {
                    if current_pixels < grow_to_fit_maximum_pixels {
                        let factor = (min_pixels.min(grow_to_fit_maximum_pixels) as f64 / current_pixels as f64).sqrt();
                        let new_width = (image.width() as f64 * factor).ceil() as u32;
                        let new_height = (image.height() as f64 * factor).ceil() as u32;
                        image = image.resize_to_fill(new_width, new_height, FilterType::Gaussian);
                    }
                }
            }

            let mut image = Image::from_dynamic_image(image);

            image.insert_data(&input_data, &insert_config).unwrap();
            image.save_to_file(output_image_file, ImageFormat::WebP).unwrap();
        },
        Command::Extract { input_image_file, output_data_file, count_of_least_significant_bits } => {
            let image = Image::load_from_file(input_image_file).unwrap();
            let count_lsb = CountOfLeastSignificantBits::try_from_bit_count(count_of_least_significant_bits.into()).unwrap();
            let output_data = image.extract_data(&ExtractConfig::builder()
                .count_of_least_significant_bits_in_red(count_lsb)
                .count_of_least_significant_bits_in_green(count_lsb)
                .count_of_least_significant_bits_in_blue(count_lsb)
                .count_of_least_significant_bits_in_alpha(CountOfLeastSignificantBits::Zero)
                .build()).unwrap();
            std::fs::write(output_data_file, output_data).unwrap();
        },
        Command::Space { input_image_file, count_of_least_significant_bits } => {
            let image = image::open(input_image_file).unwrap();
            let width = image.width();
            let height = image.height();
            let image = Image::from_dynamic_image(image);
            let count_lsb = CountOfLeastSignificantBits::try_from_bit_count(count_of_least_significant_bits.into()).unwrap();
            let max_data_capacity = image.max_data_capacity_with_config(
                &InsertConfig::builder()
                .count_of_least_significant_bits_in_red(count_lsb)
                .count_of_least_significant_bits_in_green(count_lsb)
                .count_of_least_significant_bits_in_blue(count_lsb)
                .count_of_least_significant_bits_in_alpha(CountOfLeastSignificantBits::Zero)
                .build()
            );
            println!("Image with {width}x{height}={} pixels and '{count_lsb:?}' least significant bits provides {max_data_capacity} bytes storage.", width * height);
        },
    }
}

#[derive(clap::Parser)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    Insert {
        #[arg(short='i', long)]
        input_image_file: PathBuf,
        #[arg(short='d', long)]
        input_data_file: PathBuf,
        #[arg(short='o', long)]
        output_image_file: PathBuf,
        #[arg(short, long, value_enum, default_value_t=RemainingBitsAction::Randomize)]
        remaining_bits_action: RemainingBitsAction,
        #[arg(short, long, default_value_t=1)]
        count_of_least_significant_bits: u8,
        #[arg(short='s', long)]
        shrink_to_fit_minimum_pixels: Option<usize>,
        #[arg(short='g', long)]
        grow_to_fit_maximum_pixels: Option<usize>,
    },
    Extract {
        #[arg(short='i', long)]
        input_image_file: PathBuf,
        #[arg(short='o', long)]
        output_data_file: PathBuf,
        #[arg(short, long, default_value_t=1)]
        count_of_least_significant_bits: u8,
    },
    Space {
        #[arg(short='i', long)]
        input_image_file: PathBuf,
        #[arg(short, long, default_value_t=1)]
        count_of_least_significant_bits: u8,
    }
}

#[derive(clap::ValueEnum, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
enum RemainingBitsAction {
    None,
    Zero,
    Randomize,
}

#[allow(clippy::from_over_into)]
impl Into<blind_image_steganography::RemainingBitsAction> for RemainingBitsAction {
    fn into(self) -> blind_image_steganography::RemainingBitsAction {
        match self {
            RemainingBitsAction::None => blind_image_steganography::RemainingBitsAction::None,
            RemainingBitsAction::Zero => blind_image_steganography::RemainingBitsAction::Zero,
            RemainingBitsAction::Randomize => blind_image_steganography::RemainingBitsAction::Randomize { seed: None },
        }
    }
}
