use std::{borrow::Cow, cmp::min, io::Read};

use flate2::read::ZlibDecoder;

use crate::error::PdfResult;

/// <https://www.adobe.com/content/dam/acom/en/devnet/postscript/pdfs/TN5603.Filters.pdf>
#[derive(Debug, FromObj)]

pub struct FlateDecoderParams {
    /// The default value is 1 (Predictor::Unused)
    #[field("Predictor", default = Predictor::Unused)]
    predictor: Predictor,

    /// Specifies the number of samples in the sampled row.
    ///
    /// The value of this key only has an effect on the filter if
    /// the value of `predictor` is greater than 1.
    ///
    /// The default value is 1
    #[field("Columns", default = 1)]
    columns: u32,

    /// Specifies the number of interleaved color components in a sample.
    ///
    /// The value of this key only has an effect on the filter if
    /// the value of `predictor` is greater than 1
    ///
    /// The default value is 1
    #[field("Colors", default = 1)]
    colors: u32,

    /// The number of bits used to represent each component.
    ///
    /// The possible values are 1, 2, 4, 8, and 16
    ///
    /// The default value is 8
    #[field("BitsPerComponent", default = BitsPerComponent::Eight)]
    bits_per_component: BitsPerComponent,
}

impl FlateDecoderParams {
    const fn bits_per_pixel(&self) -> u32 {
        self.colors * self.bits_per_component as u32
    }

    pub const fn bytes_per_pixel(&self) -> u32 {
        self.bits_per_pixel() / 8
    }

    pub const fn bytes_per_row(&self) -> u32 {
        self.bytes_per_pixel() * self.columns
    }
}

#[derive(Debug)]
pub struct FlateDecoder {
    params: FlateDecoderParams,
    buffer: Vec<u8>,
}

#[pdf_enum(Integer)]
enum Predictor {
    /// No filter is applied *and* no byte precedes each row
    Unused = 1,

    /// No filter is applied
    None = 10,

    /// The pixel is subtracted by the pixel to the left of it
    Sub = 11,

    /// The pixel is subtracted by the pixel above it
    Up = 12,

    /// The pixel is subtracted by the average of the pixel to the left and above
    Average = 13,

    /// The pixel is subtracted by the pixel that comes out of a prediction algorithm
    Paeth = 14,

    /// A hybrid of all 4
    Optimum = 15,
}

#[pdf_enum(Integer)]
pub enum BitsPerComponent {
    One = 1,
    Two = 2,
    Four = 4,
    Eight = 8,
    Sixteen = 16,
}

impl FlateDecoder {
    pub fn new(buffer: Cow<[u8]>, params: FlateDecoderParams) -> PdfResult<Self> {
        let mut decoder = ZlibDecoder::new(&*buffer);
        let mut buffer = Vec::new();
        decoder.read_to_end(&mut buffer)?;

        Ok(Self { buffer, params })
    }

    pub fn decode(mut self) -> Vec<u8> {
        match self.params.predictor {
            Predictor::Unused => self.buffer.to_vec(),
            Predictor::None => todo!(),
            Predictor::Sub => {
                let bytes_per_row = self.params.bytes_per_row() as usize;
                let bpp = self.params.bytes_per_pixel();

                for i in (0..self.buffer.len()).step_by(bytes_per_row) {
                    Self::decode_sub(&mut self.buffer[i..=(i + bytes_per_row)], bpp);
                }

                todo!()
            }
            Predictor::Up => {
                let bytes_per_row = self.params.bytes_per_row() as usize + 1;

                let mut out = Vec::new();

                out.extend_from_slice(&self.buffer[1..bytes_per_row]);

                for i in (bytes_per_row..self.buffer.len()).step_by(bytes_per_row) {
                    let row_above = &out[(i - bytes_per_row - (i / bytes_per_row - 1))..];

                    let this_row = &mut self.buffer[(i + 1)..(i + bytes_per_row)];
                    Self::decode_up(this_row, row_above);

                    out.extend_from_slice(this_row);
                }

                out.to_vec()
            }
            _ => todo!(),
        }
    }

    fn decode_sub(this_row: &mut [u8], bpp: u32) {
        // start at 1 because first pixel is unchanged
        let mut pixel = bpp;

        while (pixel as usize) < this_row.len() {
            for channel_idx in 0..bpp {
                let this_idx = pixel + channel_idx;
                let prev_idx = this_idx - bpp;
                this_row[this_idx as usize] =
                    this_row[this_idx as usize].wrapping_add(this_row[prev_idx as usize]);
            }

            pixel += bpp;
        }
    }

    fn decode_up(this_row: &mut [u8], row_above: &[u8]) {
        assert_eq!(this_row.len(), row_above.len());

        for idx in 0..this_row.len() {
            this_row[idx] = this_row[idx].wrapping_add(row_above[idx]);
        }
    }

    fn average(this_row: &[u8], row_above: Option<&Vec<Vec<u8>>>, chunk_size: u8) -> Vec<Vec<u8>> {
        let mut this_row_chunks: Vec<Vec<u8>> = this_row
            .chunks(chunk_size as usize)
            .map(Vec::from)
            .collect();
        for pixel_idx in 0..this_row_chunks.len() {
            for rgba_idx in 0..this_row_chunks[pixel_idx].len() {
                let a = if pixel_idx == 0 {
                    0
                } else {
                    this_row_chunks[pixel_idx - 1][rgba_idx]
                };
                let b: u8 = if let Some(val) = row_above {
                    val[pixel_idx][rgba_idx]
                } else {
                    0
                };
                this_row_chunks[pixel_idx][rgba_idx] = this_row_chunks[pixel_idx][rgba_idx]
                    .wrapping_add(((u16::from(a) + u16::from(b)) / 2) as u8);
            }
        }
        this_row_chunks
    }

    fn paeth(
        this_row: &[u8],
        row_above: Option<&Vec<Vec<u8>>>,
        chunk_size: u8,
        reverse: bool,
    ) -> Vec<Vec<u8>> {
        let mut this_row_chunks: Vec<Vec<u8>> = this_row
            .chunks(chunk_size as usize)
            .map(Vec::from)
            .collect();
        let is_first_row: bool = row_above.is_none();
        let placeholder: &Vec<Vec<u8>> = &Vec::new();
        let above: &Vec<Vec<u8>> = if let Some(val) = row_above {
            val
        } else {
            placeholder
        };
        for pixel_idx in 0..this_row_chunks.len() {
            for rgba_idx in 0..this_row_chunks[pixel_idx].len() {
                let p: u8 = if pixel_idx == 0 {
                    // the first pixel has no neighbors to the left, so we treat `a` and `c` as 0
                    // paeth_predictor(0, b, 0) = b, so we can just directly set `p = b`
                    if is_first_row {
                        0
                    } else {
                        above[pixel_idx][rgba_idx]
                    } // above
                } else {
                    let a = this_row_chunks[pixel_idx - 1][rgba_idx]; // left
                    let b = if is_first_row {
                        0
                    } else {
                        above[pixel_idx][rgba_idx]
                    }; // above
                    let c = if is_first_row {
                        0
                    } else {
                        above[pixel_idx - 1][rgba_idx]
                    }; // above left

                    Self::paeth_predictor(i16::from(a), i16::from(b), i16::from(c))
                };
                if reverse {
                    this_row_chunks[pixel_idx][rgba_idx] =
                        this_row_chunks[pixel_idx][rgba_idx].wrapping_add(p);
                } else {
                    this_row_chunks[pixel_idx][rgba_idx] =
                        this_row_chunks[pixel_idx][rgba_idx].wrapping_sub(p);
                }
            }
        }
        this_row_chunks
    }

    fn paeth_predictor(a: i16, b: i16, c: i16) -> u8 {
        let p = a + b - c;
        let pa = (p - a).abs();
        let pb = (p - b).abs();
        let pc = (p - c).abs();

        match min(min(pa, pb), pc) {
            // order here for ties is important
            diff if diff == pa => a as u8,
            diff if diff == pb => b as u8,
            diff if diff == pc => c as u8,
            _ => unreachable!(),
        }
    }
}
