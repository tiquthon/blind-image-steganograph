use blind_image_steganography::{CountOfLeastSignificantBits, Image, ImageFormat, InsertConfig, RemainingBitsAction};

fn main() {
    let mut m = Image::load_from_file("tests/assets/pexels-pixabay-53114_resized.webp").unwrap();
    let conf = InsertConfig::builder()
        .count_of_least_significant_bits_in_red(CountOfLeastSignificantBits::Two)
        .count_of_least_significant_bits_in_green(CountOfLeastSignificantBits::Two)
        .count_of_least_significant_bits_in_blue(CountOfLeastSignificantBits::Two)
        .count_of_least_significant_bits_in_alpha(CountOfLeastSignificantBits::Zero)
        .remaining_bits_action(RemainingBitsAction::Randomize)
        .build();
    println!("Max Data: {} bytes", m.max_data_capacity_with_config(&conf));
    m.insert_data(
        "Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua. At vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd gubergren, no sea takimata sanctus est Lorem ipsum dolor sit amet. Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua. At vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd gubergren, no sea takimata sanctus est Lorem ipsum dolor sit amet."
            .repeat(800).as_bytes(),
        &conf
    ).unwrap();
    m.save_to_file("kek.webp", ImageFormat::WebP);
}
