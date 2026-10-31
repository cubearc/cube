use cube_core::{
    errors::ErrorCode,
    image::{ColorModel, Image},
};
#[test]
fn an_interface_can_lower_the_decoder_sample_budget() {
    let original =
        Image::from_samples(2, 2, ColorModel::Rgba, vec![255; 16], vec![], vec![]).unwrap();
    let png = original.encode_png().unwrap();
    assert_eq!(
        Image::decode_with_sample_limit(&png, 15).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
    assert_eq!(
        Image::decode_with_sample_limit(&png, 16).unwrap().samples(),
        &[255; 16]
    );
}
