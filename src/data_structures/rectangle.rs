use crate::{catalog::assert_len, error::PdfResult, objects::Object, FromObj, Resolve};

#[derive(Debug, Clone, Copy)]
pub struct Rectangle {
    lower_left_x: f32,
    lower_left_y: f32,
    upper_right_x: f32,
    upper_right_y: f32,
}

impl<'a> FromObj<'a> for Rectangle {
    fn from_obj(obj: Object<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        let mut arr = resolver.assert_arr(obj)?;

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

impl Rectangle {
    pub fn width(&self) -> f32 {
        self.upper_right_x - self.lower_left_x
    }

    pub fn height(&self) -> f32 {
        self.upper_right_y - self.lower_left_y
    }
}
