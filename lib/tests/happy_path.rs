use temp_dir::TempDir;
use blind_image_steganography::{ExtractConfig, Image, ImageFormat, InsertConfig};

#[test]
fn happy_path() {
    // Arrange
    let insert_data = "Hello steganography!";
    let temp_dir = TempDir::new().unwrap();
    let result_file = temp_dir.child("test_a.webp");
    let mut image = Image::load_from_file("tests/assets/pexels-pixabay-53114_resized.webp").unwrap();

    // Act
    image.insert_data(insert_data.as_bytes(), &InsertConfig::builder().build()).unwrap();
    image.save_to_file(&result_file, ImageFormat::WebP);
    let image = Image::load_from_file(&result_file).unwrap();
    let extracted_data = image.extract_data(&ExtractConfig::builder().build());

    // Assert
    assert_eq!(insert_data.as_bytes(), &extracted_data);
}
