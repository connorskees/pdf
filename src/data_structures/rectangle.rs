use crate::{catalog::assert_len, error::PdfResult, objects::Object, Resolve};

#[derive(Debug, Clone, Copy)]
pub struct Rectangle {
    lower_left_x: f32,
    lower_left_y: f32,
    upper_right_x: f32,
    upper_right_y: f32,
}

impl Rectangle {
    pub(crate) fn from_arr(mut arr: Vec<Object>, resolver: &mut dyn Resolve) -> PdfResult<Self> {
        assert_len(&arr, 4)?;

        let upper_right_y = resolver.assert_number(arr.pop().unwrap())?;
        let upper_right_x = resolver.assert_number(arr.pop().unwrap())?;
        let lower_left_y = resolver.assert_number(arr.pop().unwrap())?;
        let lower_left_x = resolver.assert_number(arr.pop().unwrap())?;

        Ok(Rectangle {
            lower_left_x,
            lower_left_y,
            upper_right_x,
            upper_right_y,
        })
    }
}
