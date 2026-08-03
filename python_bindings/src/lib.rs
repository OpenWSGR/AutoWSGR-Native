use numpy::{PyArrayMethods, PyReadonlyArray2, PyReadonlyArray3, PyUntypedArrayMethods};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use std::borrow::Cow;

/// Private native (Rust) backend module for the `autowsgr_native` package.
///
/// This is the single shared extension that backs every feature subpackage.
/// Prefer the public feature packages (e.g. `autowsgr_native.recognition`),
/// which re-export the functions defined here with documentation and types.
#[pyo3::pymodule]
mod _native {
    use super::*;
    use ::autowsgr_native::recognition::{
        WrappedPixels, image::BGRImage, recognize_enemy::character_image::CharacterImage,
    };

    #[pymodule_init]
    fn init(m: &Bound<'_, PyModule>) -> PyResult<()> {
        m.add_class::<vessel_type::VesselType>()?;
        Ok(())
    }

    #[pyfunction]
    fn locate(img: PyReadonlyArray3<u8>) -> PyResult<Vec<[i32; 2]>> {
        let shape = img.shape();
        if shape[2] != 3 {
            return Err(PyValueError::new_err(format!(
                "expected 3-channel BGR image, got {} channels",
                shape[2]
            )));
        }
        let (height, width) = (shape[0], shape[1]);
        let pixels = if let Ok(slice) = img.as_slice() {
            Cow::Borrowed(slice)
        } else {
            Cow::Owned(img.to_owned_array().into_raw_vec_and_offset().0)
        };
        let wrapped = WrappedPixels {
            width,
            height,
            channels: 3,
            pixels: &pixels,
        };
        let bgr = BGRImage::from_wrapped_pixels(wrapped);
        Ok(::autowsgr_native::recognition::locator::locate(&bgr))
    }

    #[pyfunction]
    fn recognize_enemy(images: Vec<PyReadonlyArray2<u8>>) -> PyResult<String> {
        let mut char_images: Vec<CharacterImage> = Vec::with_capacity(images.len());
        for img in &images {
            let shape = img.shape();
            let (height, width) = (shape[0], shape[1]);
            if height != ::autowsgr_native::recognition::recognize_enemy::HEIGHT
                || width != ::autowsgr_native::recognition::recognize_enemy::WIDTH
            {
                return Err(PyValueError::new_err(format!(
                    "expected grayscale image with shape (H, W), got shape {:?}",
                    shape
                )));
            }
            let pixels = if let Ok(slice) = img.as_slice() {
                Cow::Borrowed(slice)
            } else {
                Cow::Owned(img.to_owned_array().into_raw_vec_and_offset().0)
            };
            let wrapped = WrappedPixels {
                width,
                height,
                channels: 1,
                pixels: &pixels,
            };
            char_images.push(CharacterImage::from_wrapped_pixels(wrapped));
        }
        let result = ::autowsgr_native::recognition::recognize_enemy::recognize_enemy(&char_images);
        Ok(result
            .iter()
            .map(|vessel_type| vessel_type.as_english())
            .collect::<Vec<_>>()
            .join(" "))
    }

    #[pyfunction]
    fn recognize_map(img: PyReadonlyArray3<u8>) -> PyResult<String> {
        let shape = img.shape();
        if shape[2] != 3 {
            return Err(PyValueError::new_err(format!(
                "expected 3-channel BGR image, got {} channels",
                shape[2]
            )));
        }
        let (height, width) = (shape[0], shape[1]);
        let pixels = if let Ok(slice) = img.as_slice() {
            Cow::Borrowed(slice)
        } else {
            Cow::Owned(img.to_owned_array().into_raw_vec_and_offset().0)
        };
        let wrapped = WrappedPixels {
            width,
            height,
            channels: 3,
            pixels: &pixels,
        };
        let bgr = BGRImage::from_wrapped_pixels(wrapped);
        let result = ::autowsgr_native::recognition::recognize_map::recognize_map(&bgr);
        Ok(result.to_string())
    }
}
