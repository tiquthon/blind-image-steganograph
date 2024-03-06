use blind_image_steganography::{Image, ImageFormat, InsertConfig, RemainingBitsAction};
use image::GenericImageView;
use temp_dir::TempDir;

#[test]
fn zeroize_works() {
    // Arrange
    let (test_pixel_x, test_pixel_y, test_pixel_value_initial) = {
        let image = image::open("tests/assets/pexels-pixabay-53114_resized.webp").unwrap();
        let (x, y) = (image.width() - 6, image.height() - 6);
        (x, y, image.get_pixel(x, y))
    };
    assert_eq!(test_pixel_value_initial.0[0] & 1, 1);
    assert_eq!(test_pixel_value_initial.0[1] & 1, 1);
    assert_eq!(test_pixel_value_initial.0[2] & 1, 1);
    assert_eq!(test_pixel_value_initial.0[3] & 1, 1);

    let mut image =
        Image::load_from_file("tests/assets/pexels-pixabay-53114_resized.webp").unwrap();

    // Act
    image
        .insert_data(
            "Hallo".as_bytes(),
            &InsertConfig::builder()
                .remaining_bits_action(RemainingBitsAction::Zero)
                .build(),
        )
        .unwrap();

    // Assert
    let temp_dir = TempDir::new().unwrap();
    let temp_file = temp_dir.child("test.webp");
    image.save_to_file(&temp_file, ImageFormat::WebP).unwrap();

    let test_pixel_value = image::open(&temp_file)
        .unwrap()
        .get_pixel(test_pixel_x, test_pixel_y);
    assert_eq!(test_pixel_value.0[0] & 1, 0);
    assert_eq!(test_pixel_value.0[1] & 1, 0);
    assert_eq!(test_pixel_value.0[2] & 1, 0);
    assert_eq!(test_pixel_value.0[3] & 1, 1);
}

#[test]
#[cfg(feature = "random")]
fn randomize_works() {
    // Arrange
    let (test_pixel_x, test_pixel_y, test_pixel_value_initial) = {
        let image = image::open("tests/assets/pexels-pixabay-53114_resized.webp").unwrap();
        let (x, y) = (image.width() - 1, image.height() - 1);
        (x, y, image.get_pixel(x, y))
    };

    let mut image =
        Image::load_from_file("tests/assets/pexels-pixabay-53114_resized.webp").unwrap();

    // Act
    image
        .insert_data(
            "Hallo".as_bytes(),
            &InsertConfig::builder()
                .remaining_bits_action(RemainingBitsAction::Randomize { seed: Some(4) })
                .build(),
        )
        .unwrap();

    // Assert
    let temp_dir = TempDir::new().unwrap();
    let temp_file = temp_dir.child("test.webp");
    image.save_to_file(&temp_file, ImageFormat::WebP).unwrap();

    let test_pixel_value = image::open(&temp_file)
        .unwrap()
        .get_pixel(test_pixel_x, test_pixel_y);
    assert_ne!(test_pixel_value_initial, test_pixel_value);
}
