//! <https://www.itu.int/rec/T-REC-T.6-198811-I/en>

use std::{collections::BTreeMap, fmt};

use bitvec::{bitvec, field::BitField, prelude::Msb0, slice::BitSlice, vec::BitVec};
use once_cell::sync::Lazy;

use crate::PdfResult;

#[derive(Debug, FromObj, Copy, Clone)]

pub struct CcitDecoderParams {
    /// A code identifying the encoding scheme used:
    ///
    /// <0: Pure two-dimensional encoding (Group 4)
    /// =0: Pure one-dimensional encoding (Group 3, 1-D)
    /// >0: Mixed one- and two-dimensional encoding (Group 3, 2-D), in which a
    ///     line encoded one-dimensionally may be followed by at most K - 1 lines
    ///     encoded two-dimensionally
    ///
    /// Default value: 0
    #[field("K", default = 0)]
    pub k: i32,

    /// A flag indicating whether end-of-line bit patterns shall be present in the
    /// encoding.
    ///
    /// The CCITTFaxDecode filter shall always accept end-of-line bit patterns. If
    /// EndOfLine is true end-of-line bit patterns shall be present
    ///
    /// Default value: false
    #[field("EndOfLine", default = false)]
    pub end_of_line: bool,

    /// A flag indicating whether the filter shall expect extra 0 bits before each
    /// encoded line so that the line begins on a byte boundary.
    ///
    /// If true, the filter shall skip over encoded bits to begin decoding each
    /// line at a byte boundary. If false, the filter shall not expect
    /// extra bits in the encoded representation
    ///
    /// Default value: false
    #[field("EncodedByteAlign", default = false)]
    pub encoded_byte_align: bool,

    /// The width of the image in pixels
    ///
    /// If the value is not a multiple of 8, the filter shall adjust the width of
    /// the unencoded image to the next multiple of 8 so that each line
    /// starts on a byte boundary
    ///
    /// Default value: 1728
    #[field("Columns", default = 1728)]
    pub columns: i32,

    /// The height of the image in scan lines
    ///
    /// If the value is 0 or absent, the image’s height is not predetermined, and
    /// the encoded data shall be terminated by an end-of-block bit pattern
    /// or by the end of the filter’s data
    ///
    /// Default value: 0
    #[field("Rows", default = 0)]
    pub rows: i32,

    /// A flag indicating whether the filter shall expect the encoded data to be
    /// terminated by an end-of-block pattern, overriding the Rows
    /// parameter. If false, the filter shall stop when it has decoded the
    /// number of lines indicated by Rows or when its data has been
    /// exhausted, whichever occurs first. The end-of-block pattern shall
    /// be the CCITT end-of-facsimile-block (EOFB) or return-to-control
    /// (RTC) appropriate for the K parameter.
    ///
    /// Default value: true.
    #[field("EndOfBlock", default = true)]
    pub end_of_block: bool,

    /// A flag indicating whether 1 bits shall be interpreted as black pixels and
    /// 0 bits as white pixels, the reverse of the normal PDF convention
    /// for image data.
    ///
    /// Default value: false.
    #[field("BlackIs1", default = false)]
    pub black_is_1: bool,

    /// The number of damaged rows of data that shall be tolerated before an error
    /// occurs.
    ///
    /// This entry shall apply only if EndOfLine is true and K is non-negative.
    /// Tolerating a damaged row shall mean locating its end in the encoded
    /// data by searching for an EndOfLine pattern and then substituting
    /// decoded data from the previous row if the previous row was not
    /// damaged, or a white scan line if the previous row was also damaged.
    ///
    /// Default value: 0.
    #[field("DamagedRowsBeforeError", default = 0)]
    pub damaged_rows_before_error: i32,
}

#[derive(Debug, Copy, Clone)]
enum Mode {
    Pass,
    Horizontal,
    V0,
    VR1,
    VR2,
    VR3,
    VL1,
    VL2,
    VL3,
    Uncompressed,
}

#[derive(Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd)]
struct DecodeKey(u32, usize);

impl fmt::Debug for DecodeKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("DecodeKey")
            .field(&if self.1 > 0 {
                format!("{:0>width$b}", self.0, width = self.1)
            } else {
                String::new()
            })
            .field(&self.1)
            .finish()
    }
}

impl fmt::Display for DecodeKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&if self.1 > 0 {
            format!("{:0>width$b}", self.0, width = self.1)
        } else {
            String::new()
        })
    }
}

struct CcitGroup4Decoder<'a> {
    reference_line: BitVec<u8, Msb0>,
    stream: &'a BitSlice<u8, Msb0>,
    params: CcitDecoderParams,
    cursor: usize,
    key_len: usize,
    current_row_len: usize,

    /// true = black, false = white
    current_color: bool,
    makeup_len: usize,
    output: BitVec<u8, Msb0>,
}

impl<'a> CcitGroup4Decoder<'a> {
    pub fn new(stream: &'a [u8], params: CcitDecoderParams) -> Self {
        let bitstream = BitSlice::<u8, Msb0>::from_slice(stream);

        Self {
            reference_line: bitvec![u8, Msb0; 0; params.columns as usize],
            stream: &bitstream,
            params,
            cursor: 0,
            key_len: 1,
            current_color: false,
            makeup_len: 0,
            output: BitVec::new(),
            current_row_len: 0,
        }
    }

    fn key(&mut self) -> DecodeKey {
        if self.key_len == 0 {
            return DecodeKey(0, 0);
        }

        if self.cursor + self.key_len > self.stream.len() || self.key_len > 16 {
            dbg!(self.cursor, self.key_len, self.stream.len());
            panic!();
        }

        DecodeKey(
            self.stream[self.cursor..self.cursor + self.key_len].load_be::<u32>(),
            self.key_len,
        )
    }

    fn lookup_makeup(&mut self) -> Option<usize> {
        let key = self.key();
        if self.current_color {
            BLACK_MAKEUP_TABLE.get(&key).copied()
        } else {
            WHITE_MAKEUP_TABLE.get(&key).copied()
        }
    }

    fn lookup_terminator(&mut self) -> Option<usize> {
        let key = self.key();
        if self.current_color {
            BLACK_TERMINATOR_TABLE.get(&key).copied()
        } else {
            WHITE_TERMINATOR_TABLE.get(&key).copied()
        }
    }

    fn lookup_code(&mut self) {
        loop {
            if let Some(len) = self.lookup_terminator() {
                self.output
                    .extend(&bitvec![u8, Msb0; self.current_color as u8; self.makeup_len + len]);

                self.current_row_len += self.makeup_len + len;
                self.makeup_len = 0;
                self.current_color = !self.current_color;
                self.reset_key();
                break;
            } else if let Some(len) = self.lookup_makeup() {
                self.makeup_len += len;
                self.reset_key();
            } else {
                self.key_len += 1;
            }
        }
    }

    fn decode_horizontal(&mut self) {
        self.lookup_code();
        self.lookup_code();
    }

    fn get_ref_line_at(&self, idx: usize) -> bool {
        self.reference_line.get(idx).map(|v| *v).unwrap_or(false)
    }

    fn b1(&self) -> usize {
        // if self.reference_line.is_empty() {
        //     return self.params.columns as usize;
        // }

        dbg!(self.current_row_len, self.params.columns as usize);

        // self.cursor points at a0
        let mut b1_cursor = self.current_row_len;

        // if color immediately above a0 isn't the same, skip until we encounter
        // range of same color
        while self.reference_line[b1_cursor] != self.current_color {
            b1_cursor += 1;
        }

        // cursor is now b0. skip to next range to find b1
        while self.reference_line[b1_cursor] != self.current_color {
            b1_cursor += 1;
        }

        b1_cursor
    }

    fn decode_vertical(&mut self, b1_offset: isize) {
        let b1 = self.b1();
        let run_len = (b1 as isize - self.current_row_len as isize + b1_offset) as usize - 1;
        self.output
            .extend(&bitvec![u8, Msb0; self.current_color as u8; run_len]);
        self.current_row_len += run_len;
        self.makeup_len = 0;
        self.current_color = !self.current_color;
        self.reset_key();
    }

    fn decode_row(&mut self) {
        // loop {
        match self.read_mode() {
            Mode::Pass => todo!(),
            Mode::Horizontal => self.decode_horizontal(),
            Mode::V0 => self.decode_vertical(0),
            Mode::VR1 => self.decode_vertical(1),
            Mode::VR2 => self.decode_vertical(2),
            Mode::VR3 => self.decode_vertical(3),
            Mode::VL1 => self.decode_vertical(-1),
            Mode::VL2 => self.decode_vertical(-2),
            Mode::VL3 => self.decode_vertical(-3),
            Mode::Uncompressed => todo!(),
        }

        //     assert_eq!(self.reference_line.len(), self.params.columns as usize);

        //     dbg!(
        //         self.current_row_len,
        //         // self.params.columns,
        //         // self.reference_line.len()
        //     );

        //     if self.current_row_len == self.params.columns as usize {
        //         self.reference_line =
        //             self.output[self.output.len() - self.current_row_len..].to_bitvec();
        //         self.current_row_len = 0;
        //         break;
        //     }

        //     if self.current_row_len > self.params.columns as usize {
        //         dbg!(self.current_row_len);
        //         panic!();
        //     }

        //     if self.key() == END_OF_FACSIMILE_BLOCK {
        //         panic!("EOFB");
        //     }
        // }
    }

    fn read_mode(&mut self) -> Mode {
        loop {
            let key = self.key();

            if let Some(mode) = MODE_TABLE.get(&key) {
                dbg!(mode);
                self.reset_key();
                return *mode;
            } else {
                self.key_len += 1;
            }
        }
    }

    fn reset_key(&mut self) {
        self.cursor += self.key_len;
        self.key_len = 1;
    }
}

pub(crate) fn decode_ccitt_group_4(stream: &[u8], params: CcitDecoderParams) -> PdfResult<Vec<u8>> {
    if stream.len() != 1468 {
        return Ok(Vec::new());
    }

    let mut decoder = CcitGroup4Decoder::new(stream, params);

    while decoder.key() != END_OF_FACSIMILE_BLOCK
    // && decoder.cursor + decoder.key_len < decoder.stream.len()
    {
        decoder.decode_row();
        //     // decoder.attempt_lookup();
    }

    // // dbg!(decoder.output.len());

    // // todo: params.black_is_1
    // Ok(decoder.output.into_iter().map(|b| b as u8).collect())
    Ok(Vec::new())
}

static WHITE_MAKEUP_TABLE: Lazy<BTreeMap<DecodeKey, usize>> = Lazy::new(|| {
    let mut map = BTreeMap::new();
    map.insert(DecodeKey(0b11011, "11011".len()), 64);
    map.insert(DecodeKey(0b10010, "10010".len()), 128);
    map.insert(DecodeKey(0b010111, "010111".len()), 192);
    map.insert(DecodeKey(0b0110111, "0110111".len()), 256);
    map.insert(DecodeKey(0b00110110, "00110110".len()), 320);
    map.insert(DecodeKey(0b00110111, "00110111".len()), 384);
    map.insert(DecodeKey(0b01100100, "01100100".len()), 448);
    map.insert(DecodeKey(0b01100101, "01100101".len()), 512);
    map.insert(DecodeKey(0b01101000, "01101000".len()), 576);
    map.insert(DecodeKey(0b01100111, "01100111".len()), 640);
    map.insert(DecodeKey(0b011001100, "011001100".len()), 704);
    map.insert(DecodeKey(0b011001101, "011001101".len()), 768);
    map.insert(DecodeKey(0b011010010, "011010010".len()), 832);
    map.insert(DecodeKey(0b011010011, "011010011".len()), 896);
    map.insert(DecodeKey(0b011010100, "011010100".len()), 960);
    map.insert(DecodeKey(0b011010101, "011010101".len()), 1024);
    map.insert(DecodeKey(0b011010110, "011010110".len()), 1088);
    map.insert(DecodeKey(0b011010111, "011010111".len()), 1152);
    map.insert(DecodeKey(0b011011000, "011011000".len()), 1216);
    map.insert(DecodeKey(0b011011001, "011011001".len()), 1280);
    map.insert(DecodeKey(0b011011010, "011011010".len()), 1344);
    map.insert(DecodeKey(0b011011011, "011011011".len()), 1408);
    map.insert(DecodeKey(0b010011000, "010011000".len()), 1472);
    map.insert(DecodeKey(0b010011001, "010011001".len()), 1536);
    map.insert(DecodeKey(0b010011010, "010011010".len()), 1600);
    map.insert(DecodeKey(0b011000, "011000".len()), 1664);
    map.insert(DecodeKey(0b010011011, "010011011".len()), 1728);
    map.insert(DecodeKey(0b00000001000, "00000001000".len()), 1792);
    map.insert(DecodeKey(0b00000001100, "00000001100".len()), 1856);
    map.insert(DecodeKey(0b00000001101, "00000001101".len()), 1920);
    map.insert(DecodeKey(0b000000010010, "000000010010".len()), 1984);
    map.insert(DecodeKey(0b000000010011, "000000010011".len()), 2048);
    map.insert(DecodeKey(0b000000010100, "000000010100".len()), 2112);
    map.insert(DecodeKey(0b000000010101, "000000010101".len()), 2176);
    map.insert(DecodeKey(0b000000010110, "000000010110".len()), 2240);
    map.insert(DecodeKey(0b000000010111, "000000010111".len()), 2304);
    map.insert(DecodeKey(0b000000011100, "000000011100".len()), 2368);
    map.insert(DecodeKey(0b000000011101, "000000011101".len()), 2432);
    map.insert(DecodeKey(0b000000011110, "000000011110".len()), 2496);
    map.insert(DecodeKey(0b000000011111, "000000011111".len()), 2560);
    map
});

static BLACK_MAKEUP_TABLE: Lazy<BTreeMap<DecodeKey, usize>> = Lazy::new(|| {
    let mut map = BTreeMap::new();
    map.insert(DecodeKey(0b0000001111, "0000001111".len()), 64);
    map.insert(DecodeKey(0b000011001000, "000011001000".len()), 128);
    map.insert(DecodeKey(0b000011001001, "000011001001".len()), 192);
    map.insert(DecodeKey(0b000001011011, "000001011011".len()), 256);
    map.insert(DecodeKey(0b000000110011, "000000110011".len()), 320);
    map.insert(DecodeKey(0b000000110100, "000000110100".len()), 384);
    map.insert(DecodeKey(0b000000110101, "000000110101".len()), 448);
    map.insert(DecodeKey(0b0000001101100, "0000001101100".len()), 512);
    map.insert(DecodeKey(0b0000001101101, "0000001101101".len()), 576);
    map.insert(DecodeKey(0b0000001001010, "0000001001010".len()), 640);
    map.insert(DecodeKey(0b0000001001011, "0000001001011".len()), 704);
    map.insert(DecodeKey(0b0000001001100, "0000001001100".len()), 768);
    map.insert(DecodeKey(0b0000001001101, "0000001001101".len()), 832);
    map.insert(DecodeKey(0b0000001110010, "0000001110010".len()), 896);
    map.insert(DecodeKey(0b0000001110011, "0000001110011".len()), 960);
    map.insert(DecodeKey(0b0000001110100, "0000001110100".len()), 1024);
    map.insert(DecodeKey(0b0000001110101, "0000001110101".len()), 1088);
    map.insert(DecodeKey(0b0000001110110, "0000001110110".len()), 1152);
    map.insert(DecodeKey(0b0000001110111, "0000001110111".len()), 1216);
    map.insert(DecodeKey(0b0000001010010, "0000001010010".len()), 1280);
    map.insert(DecodeKey(0b0000001010011, "0000001010011".len()), 1344);
    map.insert(DecodeKey(0b0000001010100, "0000001010100".len()), 1408);
    map.insert(DecodeKey(0b0000001010101, "0000001010101".len()), 1472);
    map.insert(DecodeKey(0b0000001011010, "0000001011010".len()), 1536);
    map.insert(DecodeKey(0b0000001011011, "0000001011011".len()), 1600);
    map.insert(DecodeKey(0b0000001100100, "0000001100100".len()), 1664);
    map.insert(DecodeKey(0b0000001100101, "0000001100101".len()), 1728);
    map.insert(DecodeKey(0b00000001000, "00000001000".len()), 1792);
    map.insert(DecodeKey(0b00000001100, "00000001100".len()), 1856);
    map.insert(DecodeKey(0b00000001101, "00000001101".len()), 1920);
    map.insert(DecodeKey(0b000000010010, "000000010010".len()), 1984);
    map.insert(DecodeKey(0b000000010011, "000000010011".len()), 2048);
    map.insert(DecodeKey(0b000000010100, "000000010100".len()), 2112);
    map.insert(DecodeKey(0b000000010101, "000000010101".len()), 2176);
    map.insert(DecodeKey(0b000000010110, "000000010110".len()), 2240);
    map.insert(DecodeKey(0b000000010111, "000000010111".len()), 2304);
    map.insert(DecodeKey(0b000000011100, "000000011100".len()), 2368);
    map.insert(DecodeKey(0b000000011101, "000000011101".len()), 2432);
    map.insert(DecodeKey(0b000000011110, "000000011110".len()), 2496);
    map.insert(DecodeKey(0b000000011111, "000000011111".len()), 2560);
    map
});

static MODE_TABLE: Lazy<BTreeMap<DecodeKey, Mode>> = Lazy::new(|| {
    let mut map = BTreeMap::new();
    map.insert(DecodeKey(0b0001, "0001".len()), Mode::Pass);
    map.insert(DecodeKey(0b001, "001".len()), Mode::Horizontal);
    map.insert(DecodeKey(0b1, "1".len()), Mode::V0);
    map.insert(DecodeKey(0b011, "011".len()), Mode::VR1);
    map.insert(DecodeKey(0b000011, "000011".len()), Mode::VR2);
    map.insert(DecodeKey(0b0000011, "0000011".len()), Mode::VR3);
    map.insert(DecodeKey(0b010, "010".len()), Mode::VL1);
    map.insert(DecodeKey(0b000010, "000010".len()), Mode::VL2);
    map.insert(DecodeKey(0b0000010, "0000010".len()), Mode::VL3);
    map.insert(
        DecodeKey(0b0000001111, "0000001111".len()),
        Mode::Uncompressed,
    );
    map
});

static WHITE_TERMINATOR_TABLE: Lazy<BTreeMap<DecodeKey, usize>> = Lazy::new(|| {
    let mut map = BTreeMap::new();
    map.insert(DecodeKey(0b00110101, "00110101".len()), 0);
    map.insert(DecodeKey(0b000111, "000111".len()), 1);
    map.insert(DecodeKey(0b0111, "0111".len()), 2);
    map.insert(DecodeKey(0b1000, "1000".len()), 3);
    map.insert(DecodeKey(0b1011, "1011".len()), 4);
    map.insert(DecodeKey(0b1100, "1100".len()), 5);
    map.insert(DecodeKey(0b1110, "1110".len()), 6);
    map.insert(DecodeKey(0b1111, "1111".len()), 7);
    map.insert(DecodeKey(0b10011, "10011".len()), 8);
    map.insert(DecodeKey(0b10100, "10100".len()), 9);
    map.insert(DecodeKey(0b00111, "00111".len()), 10);
    map.insert(DecodeKey(0b01000, "01000".len()), 11);
    map.insert(DecodeKey(0b001000, "001000".len()), 12);
    map.insert(DecodeKey(0b000011, "000011".len()), 13);
    map.insert(DecodeKey(0b110100, "110100".len()), 14);
    map.insert(DecodeKey(0b110101, "110101".len()), 15);
    map.insert(DecodeKey(0b101010, "101010".len()), 16);
    map.insert(DecodeKey(0b101011, "101011".len()), 17);
    map.insert(DecodeKey(0b0100111, "0100111".len()), 18);
    map.insert(DecodeKey(0b0001100, "0001100".len()), 19);
    map.insert(DecodeKey(0b0001000, "0001000".len()), 20);
    map.insert(DecodeKey(0b0010111, "0010111".len()), 21);
    map.insert(DecodeKey(0b0000011, "0000011".len()), 22);
    map.insert(DecodeKey(0b0000100, "0000100".len()), 23);
    map.insert(DecodeKey(0b0101000, "0101000".len()), 24);
    map.insert(DecodeKey(0b0101011, "0101011".len()), 25);
    map.insert(DecodeKey(0b0010011, "0010011".len()), 26);
    map.insert(DecodeKey(0b0100100, "0100100".len()), 27);
    map.insert(DecodeKey(0b0011000, "0011000".len()), 28);
    map.insert(DecodeKey(0b00000010, "00000010".len()), 29);
    map.insert(DecodeKey(0b00000011, "00000011".len()), 30);
    map.insert(DecodeKey(0b00011010, "00011010".len()), 31);
    map.insert(DecodeKey(0b00011011, "00011011".len()), 32);
    map.insert(DecodeKey(0b00010010, "00010010".len()), 33);
    map.insert(DecodeKey(0b00010011, "00010011".len()), 34);
    map.insert(DecodeKey(0b00010100, "00010100".len()), 35);
    map.insert(DecodeKey(0b00010101, "00010101".len()), 36);
    map.insert(DecodeKey(0b00010110, "00010110".len()), 37);
    map.insert(DecodeKey(0b00010111, "00010111".len()), 38);
    map.insert(DecodeKey(0b00101000, "00101000".len()), 39);
    map.insert(DecodeKey(0b00101001, "00101001".len()), 40);
    map.insert(DecodeKey(0b00101010, "00101010".len()), 41);
    map.insert(DecodeKey(0b00101011, "00101011".len()), 42);
    map.insert(DecodeKey(0b00101100, "00101100".len()), 43);
    map.insert(DecodeKey(0b00101101, "00101101".len()), 44);
    map.insert(DecodeKey(0b00000100, "00000100".len()), 45);
    map.insert(DecodeKey(0b00000101, "00000101".len()), 46);
    map.insert(DecodeKey(0b00001010, "00001010".len()), 47);
    map.insert(DecodeKey(0b00001011, "00001011".len()), 48);
    map.insert(DecodeKey(0b01010010, "01010010".len()), 49);
    map.insert(DecodeKey(0b01010011, "01010011".len()), 50);
    map.insert(DecodeKey(0b01010100, "01010100".len()), 51);
    map.insert(DecodeKey(0b01010101, "01010101".len()), 52);
    map.insert(DecodeKey(0b00100100, "00100100".len()), 53);
    map.insert(DecodeKey(0b00100101, "00100101".len()), 54);
    map.insert(DecodeKey(0b01011000, "01011000".len()), 55);
    map.insert(DecodeKey(0b01011001, "01011001".len()), 56);
    map.insert(DecodeKey(0b01011010, "01011010".len()), 57);
    map.insert(DecodeKey(0b01011011, "01011011".len()), 58);
    map.insert(DecodeKey(0b01001010, "01001010".len()), 59);
    map.insert(DecodeKey(0b01001011, "01001011".len()), 60);
    map.insert(DecodeKey(0b00110010, "00110010".len()), 61);
    map.insert(DecodeKey(0b00110011, "00110011".len()), 62);
    map.insert(DecodeKey(0b00110100, "00110100".len()), 63);
    map
});

static BLACK_TERMINATOR_TABLE: Lazy<BTreeMap<DecodeKey, usize>> = Lazy::new(|| {
    let mut map = BTreeMap::new();
    map.insert(DecodeKey(0b0000110111, "0000110111".len()), 0);
    map.insert(DecodeKey(0b010, "010".len()), 1);
    map.insert(DecodeKey(0b11, "11".len()), 2);
    map.insert(DecodeKey(0b10, "10".len()), 3);
    map.insert(DecodeKey(0b011, "011".len()), 4);
    map.insert(DecodeKey(0b0011, "0011".len()), 5);
    map.insert(DecodeKey(0b0010, "0010".len()), 6);
    map.insert(DecodeKey(0b00011, "00011".len()), 7);
    map.insert(DecodeKey(0b000101, "000101".len()), 8);
    map.insert(DecodeKey(0b000100, "000100".len()), 9);
    map.insert(DecodeKey(0b0000100, "0000100".len()), 10);
    map.insert(DecodeKey(0b0000101, "0000101".len()), 11);
    map.insert(DecodeKey(0b0000111, "0000111".len()), 12);
    map.insert(DecodeKey(0b00000100, "00000100".len()), 13);
    map.insert(DecodeKey(0b00000111, "00000111".len()), 14);
    map.insert(DecodeKey(0b000011000, "000011000".len()), 15);
    map.insert(DecodeKey(0b0000010111, "0000010111".len()), 16);
    map.insert(DecodeKey(0b0000011000, "0000011000".len()), 17);
    map.insert(DecodeKey(0b0000001000, "0000001000".len()), 18);
    map.insert(DecodeKey(0b00001100111, "00001100111".len()), 19);
    map.insert(DecodeKey(0b00001101000, "00001101000".len()), 20);
    map.insert(DecodeKey(0b00001101100, "00001101100".len()), 21);
    map.insert(DecodeKey(0b00000110111, "00000110111".len()), 22);
    map.insert(DecodeKey(0b00000101000, "00000101000".len()), 23);
    map.insert(DecodeKey(0b00000010111, "00000010111".len()), 24);
    map.insert(DecodeKey(0b00000011000, "00000011000".len()), 25);
    map.insert(DecodeKey(0b000011001010, "000011001010".len()), 26);
    map.insert(DecodeKey(0b000011001011, "000011001011".len()), 27);
    map.insert(DecodeKey(0b000011001100, "000011001100".len()), 28);
    map.insert(DecodeKey(0b000011001101, "000011001101".len()), 29);
    map.insert(DecodeKey(0b000001101000, "000001101000".len()), 30);
    map.insert(DecodeKey(0b000001101001, "000001101001".len()), 31);
    map.insert(DecodeKey(0b000001101010, "000001101010".len()), 32);
    map.insert(DecodeKey(0b000001101011, "000001101011".len()), 33);
    map.insert(DecodeKey(0b000011010010, "000011010010".len()), 34);
    map.insert(DecodeKey(0b000011010011, "000011010011".len()), 35);
    map.insert(DecodeKey(0b000011010100, "000011010100".len()), 36);
    map.insert(DecodeKey(0b000011010101, "000011010101".len()), 37);
    map.insert(DecodeKey(0b000011010110, "000011010110".len()), 38);
    map.insert(DecodeKey(0b000011010111, "000011010111".len()), 39);
    map.insert(DecodeKey(0b000001101100, "000001101100".len()), 40);
    map.insert(DecodeKey(0b000001101101, "000001101101".len()), 41);
    map.insert(DecodeKey(0b000011011010, "000011011010".len()), 42);
    map.insert(DecodeKey(0b000011011011, "000011011011".len()), 43);
    map.insert(DecodeKey(0b000001010100, "000001010100".len()), 44);
    map.insert(DecodeKey(0b000001010101, "000001010101".len()), 45);
    map.insert(DecodeKey(0b000001010110, "000001010110".len()), 46);
    map.insert(DecodeKey(0b000001010111, "000001010111".len()), 47);
    map.insert(DecodeKey(0b000001100100, "000001100100".len()), 48);
    map.insert(DecodeKey(0b000001100101, "000001100101".len()), 49);
    map.insert(DecodeKey(0b000001010010, "000001010010".len()), 50);
    map.insert(DecodeKey(0b000001010011, "000001010011".len()), 51);
    map.insert(DecodeKey(0b000000100100, "000000100100".len()), 52);
    map.insert(DecodeKey(0b000000110111, "000000110111".len()), 53);
    map.insert(DecodeKey(0b000000111000, "000000111000".len()), 54);
    map.insert(DecodeKey(0b000000100111, "000000100111".len()), 55);
    map.insert(DecodeKey(0b000000101000, "000000101000".len()), 56);
    map.insert(DecodeKey(0b000001011000, "000001011000".len()), 57);
    map.insert(DecodeKey(0b000001011001, "000001011001".len()), 58);
    map.insert(DecodeKey(0b000000101011, "000000101011".len()), 59);
    map.insert(DecodeKey(0b000000101100, "000000101100".len()), 60);
    map.insert(DecodeKey(0b000001011010, "000001011010".len()), 61);
    map.insert(DecodeKey(0b000001100110, "000001100110".len()), 62);
    map.insert(DecodeKey(0b000001100111, "000001100111".len()), 63);
    map
});

const END_OF_FACSIMILE_BLOCK: DecodeKey =
    DecodeKey(0b000000000001000000000001, "000000000001000000000001".len());
