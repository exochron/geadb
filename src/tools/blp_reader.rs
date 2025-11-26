use image::{DynamicImage, GenericImageView};
use palette::{IntoColor, Lab, Srgb};
use wow_blp::convert::blp_to_image;
use wow_blp::parser::{load_blp, LoadError};

pub(crate) struct BLPReader {
    image: DynamicImage,
}

impl BLPReader {
    pub(crate) fn new(build_version: &String, file_path: &String) -> Result<Self, LoadError> {
        let file_path = r"extract/".to_owned() + build_version + r"/" + file_path;
        load_blp(file_path.clone()).map(|blp| blp_to_image(&blp, 0).expect("converted"))
                .map(|blp| Self{image: blp})
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
