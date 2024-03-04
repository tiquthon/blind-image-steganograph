use temp_dir::TempDir;

use blind_image_steganography::{Image, ImageFormat};

#[test]
fn load_save_file() {
    // Arrange
    let temp_dir = TempDir::new().unwrap();
    let result_file_a = temp_dir.child("test_a.webp");
    let result_file_b = temp_dir.child("test_b.webp");

    // Act
    let image_a = Image::load_from_file("tests/assets/pexels-pixabay-53114_resized.webp").unwrap();

    image_a.save_to_file(&result_file_a, ImageFormat::WebP);

    let image_a_bytes = std::fs::read(&result_file_a).unwrap();
    let image_b = Image::load_from_memory(&image_a_bytes).unwrap();

    let image_b_bytes = image_b.save_to_memory(ImageFormat::WebP);

    std::fs::write(&result_file_b, &image_b_bytes).unwrap();
    let image_c = Image::load_from_file_with_format(&result_file_b, ImageFormat::WebP).unwrap();

    let image_d = Image::load_from_memory_with_format(&image_b_bytes, ImageFormat::WebP).unwrap();

    // Assert
    assert_eq!(image_a, image_b);
    assert_eq!(image_a, image_c);
    assert_eq!(image_a, image_d);
    assert_eq!(image_b, image_c);
    assert_eq!(image_b, image_d);
    assert_eq!(image_c, image_d);
}
