use image::{DynamicImage, GenericImageView};
use image_blp::convert::blp_to_image;
use image_blp::parser::load_blp;
use palette::{IntoColor, Lab, Srgb};

pub(crate) struct BLPReader {
    image: DynamicImage,
}

impl BLPReader {
    pub(crate) fn new(build_version: &String, file_path: &String) -> Self {
        let file_path = r"extract/".to_owned() + build_version + r"/" + file_path;
        let blp_file = load_blp(file_path).expect("loaded blp");
        let image = blp_to_image(&blp_file, 0).expect("converted");

        Self { image }
    }

    pub(crate) fn convert_to_lab(&self) -> Vec<Lab> {
        self.image
            .pixels()
            .map(|(_x, _y, pxl)| {
                Srgb::new(pxl.0[0], pxl.0[1], pxl.0[2])
                    .into_format()
                    .into_color()
            })
            .collect()
    }
}
