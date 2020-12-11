#![feature(or_patterns)]
// TODO: consider verifying the file header

use std::{
    collections::HashMap,
    fs::File,
    io::{self, Read},
};

const FORM_FEED: u8 = b'\x0C';
const BACKSPACE: u8 = b'\x08';

#[derive(Debug)]
struct ParseError;

type PdfResult<T> = Result<T, ParseError>;

// enum PdfResult<Object> {
//     Pages,
//     Page,
//     Catalog,
//     Root,
//     Font,
// }

#[derive(Debug, Clone)]
enum Object {
    Null,
    True,
    False,
    Integer(i32),
    Real(f32),
    String(String),
    Name(String),
    Array(Vec<Self>),
    Stream(Vec<u8>),
    Dictionary(HashMap<String, Self>),
}

impl Object {
    pub fn assert_integer(self) -> PdfResult<i32> {
        match self {
            Self::Integer(i) => Ok(i),
            _ => Err(ParseError),
        }
    }

    pub fn assert_dict(self) -> PdfResult<HashMap<String, Self>> {
        match self {
            Self::Dictionary(d) => Ok(d),
            _ => Err(ParseError),
        }
    }
}

struct StreamDict {
    len: usize,
    filter: Option<Object>,
    decode_params: Option<Object>,
    f: Option<Object>,
    f_filter: Option<Object>,
    f_decode_params: Option<Object>,
    dl: Option<usize>,
}

impl StreamDict {
    pub fn from_dict(mut dict: HashMap<String, Object>) -> PdfResult<Self> {
        let len = dict
            .remove("Length")
            .map(Object::assert_integer)
            .ok_or(ParseError)?? as usize;

        let filter = dict.remove("Filter");
        let decode_params = dict.remove("DecodeParms");
        let f = dict.remove("F");
        let f_filter = dict.remove("FFilter");
        let f_decode_params = dict.remove("FDecodeParms");
        let dl = match dict.remove("DL").map(Object::assert_integer) {
            Some(Ok(v)) => Some(v as usize),
            Some(Err(e)) => return Err(e),
            None => None,
        };

        if !dict.is_empty() {
            todo!()
        }

        Ok(Self {
            len,
            filter,
            decode_params,
            f,
            f_filter,
            f_decode_params,
            dl,
        })
    }
}

struct Lexer {
    file: Vec<u8>,
    pos: usize,
    objects: Vec<Object>,
}

impl Lexer {
    pub fn new(p: &'static str) -> io::Result<Self> {
        let mut file = Vec::new();
        File::open(p)?.read_to_end(&mut file)?;

        Ok(Self {
            file,
            pos: 0,
            objects: Vec::new(),
        })
    }

    fn lex_object(&mut self) -> PdfResult<Object> {
        self.skip_whitespace();
        match self.peek_byte() {
            Some(b't') => self.lex_true(),
            Some(b'f') => self.lex_false(),
            Some(b'n') => self.lex_null(),
            Some(b'<') => self.lex_gt(),
            Some(b'+' | b'-' | b'0'..=b'9') => self.lex_number(),
            Some(b'(') => self.lex_string(),
            Some(b'/') => Ok(Object::Name(self.lex_name()?)),
            Some(b'[') => self.lex_array(),
            Some(b's') => {
                let stream_dict = self.objects.last().unwrap().clone().assert_dict()?;
                self.lex_stream(StreamDict::from_dict(stream_dict)?)
            }
            Some(b) => todo!("{:?}", b as char),
            None => todo!(),
        }
    }

    /// Assumes leading 't' has not been consumed
    fn lex_true(&mut self) -> PdfResult<Object> {
        self.expect_byte(b't')?;
        self.expect_byte(b'r')?;
        self.expect_byte(b'u')?;
        self.expect_byte(b'e')?;

        Ok(Object::True)
    }

    /// Assumes leading 'f' has not been consumed
    fn lex_false(&mut self) -> PdfResult<Object> {
        self.expect_byte(b'f')?;
        self.expect_byte(b'a')?;
        self.expect_byte(b'l')?;
        self.expect_byte(b's')?;
        self.expect_byte(b'e')?;

        Ok(Object::False)
    }

    /// Assumes leading 'n' has not been consumed
    fn lex_null(&mut self) -> PdfResult<Object> {
        self.expect_byte(b'n')?;
        self.expect_byte(b'u')?;
        self.expect_byte(b'l')?;
        self.expect_byte(b'l')?;

        Ok(Object::Null)
    }

    fn lex_gt(&mut self) -> PdfResult<Object> {
        match self.peek_byte_offset(1) {
            Some(b'<') => self.lex_dict(),
            Some(b'0'..=b'9' | b'A'..=b'F') => self.lex_hex_string(),
            Some(..) | None => todo!(),
        }
    }

    fn lex_dict(&mut self) -> PdfResult<Object> {
        self.expect_byte(b'<')?;
        self.expect_byte(b'<')?;
        self.skip_whitespace();

        let mut dict = HashMap::new();

        while let Some(b) = self.peek_byte() {
            if b == b'>' {
                self.next_byte();
                self.expect_byte(b'>')?;
                break;
            }

            let name = self.lex_name()?;
            let value = self.lex_object()?;
            dict.insert(name, value);
        }

        Ok(Object::Dictionary(dict))
    }

    fn lex_hex_string(&mut self) -> PdfResult<Object> {
        self.expect_byte(b'<')?;

        let mut string = String::new();

        while let Some(b) = self.next_byte() {
            if b == b'>' {
                break;
            }

            if !b.is_ascii_hexdigit() {
                todo!()
            }

            let mut this_byte = Self::hex_byte_to_digit(b);

            this_byte *= 16;

            match self.next_byte() {
                Some(b @ (b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z')) => {
                    this_byte += Self::hex_byte_to_digit(b)
                }
                Some(..) | None => todo!(),
            }

            string.push(this_byte as char);
        }

        Ok(Object::String(string))
    }

    fn lex_number(&mut self) -> PdfResult<Object> {
        let negative = match self.peek_byte() {
            Some(b'+') => {
                self.next_byte();
                1
            }
            Some(b'-') => {
                self.next_byte();
                -1
            }
            Some(..) => 1,
            None => unreachable!(),
        };

        let whole_number = self.lex_whole_number();

        Ok(if self.peek_byte() == Some(b'.') {
            self.next_byte();
            let decimal_number = format!("{}.{}", whole_number, self.lex_whole_number());
            Object::Real(decimal_number.parse::<f32>().unwrap() * negative as f32)
        } else {
            Object::Integer(whole_number.parse::<i32>().unwrap() * negative)
        })
    }

    // TODO: throw error on empty string
    fn lex_whole_number(&mut self) -> String {
        let mut whole_number = String::new();

        while let Some(b) = self.peek_byte() {
            if !b.is_ascii_digit() {
                break;
            }

            self.next_byte();

            whole_number.push(b as char);
        }

        whole_number
    }

    // TODO: figure out how parens work
    fn lex_string(&mut self) -> PdfResult<Object> {
        self.expect_byte(b'(')?;

        let mut string = String::new();
        let mut num_open_parens = 0;

        while let Some(b) = self.peek_byte() {
            match b {
                b')' if num_open_parens == 0 => {
                    self.next_byte();
                    break;
                }
                b')' => {
                    num_open_parens -= 1;
                    string.push(')');
                }
                b'(' => {
                    num_open_parens += 1;
                    string.push('(');
                }
                b'\\' => {
                    self.next_byte();
                    match self.next_byte() {
                        Some(b'n') => string.push('\n'),
                        Some(b'r') => string.push('\r'),
                        Some(b't') => string.push('\t'),
                        Some(b'b') => string.push(BACKSPACE as char),
                        Some(b'f') => string.push(FORM_FEED as char),
                        Some(b'(') => string.push('('),
                        Some(b')') => string.push(')'),
                        Some(b'\\') => string.push('\\'),
                        // TODO: do we skip whitespace after `\` in multiline string?
                        Some(b'\n' | b'\r') => self.skip_whitespace(),
                        // TODO: octal escape
                        Some(..) | None => todo!(),
                    }
                }
                _ => {
                    string.push(b as char);
                }
            }
        }

        Ok(Object::String(string))
    }

    fn lex_name(&mut self) -> PdfResult<String> {
        self.expect_byte(b'/')?;

        let mut name = String::new();

        while let Some(b) = self.peek_byte() {
            if !Self::is_regular(b) {
                break;
            }

            self.next_byte();

            if b == b'#' {
                let mut val;

                match self.next_byte() {
                    Some(n @ b'0'..=b'9') => val = n - b'0',
                    Some(..) | None => todo!(),
                }

                match self.next_byte() {
                    Some(n @ b'0'..=b'9') => {
                        val *= 16;
                        val += n - b'0';
                    }
                    Some(..) | None => todo!(),
                }

                name.push(val as char);
            } else {
                name.push(b as char);
            }
        }

        Ok(name)
    }

    fn lex_array(&mut self) -> PdfResult<Object> {
        let mut arr = Vec::new();
        self.expect_byte(b'[')?;
        while let Some(b) = self.peek_byte() {
            if b == b']' {
                self.next_byte();
                break;
            }

            arr.push(self.lex_object()?);
        }

        Ok(Object::Array(arr))
    }

    fn lex_stream(&mut self, stream_dict: StreamDict) -> PdfResult<Object> {
        self.expect_byte(b's')?;
        self.expect_byte(b't')?;
        self.expect_byte(b'r')?;
        self.expect_byte(b'e')?;
        self.expect_byte(b'a')?;
        self.expect_byte(b'm')?;
        self.expect_eol()?;

        let stream = self.get_byte_range(self.pos, self.pos + stream_dict.len);
        self.pos += stream_dict.len;

        self.expect_eol()?;

        self.expect_byte(b'e')?;
        self.expect_byte(b'n')?;
        self.expect_byte(b'd')?;
        self.expect_byte(b's')?;
        self.expect_byte(b't')?;
        self.expect_byte(b'r')?;
        self.expect_byte(b'e')?;
        self.expect_byte(b'a')?;
        self.expect_byte(b'm')?;
        self.expect_eol()?;

        Ok(Object::Stream(stream))
    }

    fn skip_whitespace(&mut self) {
        while let Some(b) = self.peek_byte() {
            if Self::is_whitespace(b) {
                self.next_byte();
            } else if b == b'%' {
                self.next_byte();
                self.skip_comment();
            } else {
                break;
            }
        }
    }

    fn next_byte(&mut self) -> Option<u8> {
        self.file.get(self.pos).cloned().map(|b| {
            self.pos += 1;
            b
        })
    }

    fn peek_byte(&self) -> Option<u8> {
        self.file.get(self.pos).cloned()
    }

    fn peek_byte_offset(&self, offset: usize) -> Option<u8> {
        self.file.get(self.pos + offset).cloned()
    }

    /// `start` is inclusive, `end` is exclusive
    /// 0 indexed
    fn get_byte_range(&self, start: usize, end: usize) -> Vec<u8> {
        if start == end {
            return Vec::new();
        }

        self.file[start..end].to_vec()
    }

    fn expect_byte(&mut self, b: u8) -> PdfResult<()> {
        match self.next_byte() {
            Some(b2) if b == b2 => Ok(()),
            Some(..) | None => Err(ParseError),
        }
    }

    fn expect_eol(&mut self) -> PdfResult<()> {
        match self.next_byte() {
            Some(b'\n') => Ok(()),
            Some(b'\r') => {
                if self.peek_byte() == Some(b'\n') {
                    self.next_byte();
                }
                Ok(())
            }
            Some(..) | None => todo!(),
        }
    }

    /// Whitespace chars are defined as
    ///
    /// * NUL             0x0
    /// * Horizontal tab  0x9
    /// * Line feed       0xa
    /// * Form feed       0xc
    /// * Carriage return 0xd
    /// * Space           0x20
    ///
    fn is_whitespace(b: u8) -> bool {
        matches!(b, b'\0' | 0x9 | b'\n' | FORM_FEED | b'\r' | b' ')
    }

    fn is_delimiter(b: u8) -> bool {
        matches!(
            b,
            b'(' | b')' | b'<' | b'>' | b'[' | b']' | b'{' | b'}' | b'/' | b'%'
        )
    }

    fn is_regular(b: u8) -> bool {
        !Self::is_whitespace(b) && !Self::is_delimiter(b)
    }

    fn next_is_eol(&self) -> bool {
        match self.peek_byte() {
            Some(b'\r' | b'\n') => true,
            Some(..) => false,
            None => true,
        }
    }

    fn hex_byte_to_digit(b: u8) -> u8 {
        match b {
            b'0'..=b'9' => b - b'0',
            b'a'..=b'z' => b - b'a',
            b'A'..=b'Z' => b - b'A',
            _ => todo!(),
        }
    }

    /// Assumes the leading `%` has already been consumed
    fn skip_comment(&mut self) {
        while !self.next_is_eol() {
            self.next_byte();
        }
    }

    pub fn run(mut self) -> PdfResult<Vec<Object>> {
        while self.peek_byte().is_some() {
            dbg!("hiiii");
            self.lex_object()?;
        }

        Ok(self.objects)
    }
}

struct Parser {
    lexer: Lexer,
}

impl Parser {
    pub fn new(p: &'static str) -> io::Result<Self> {
        let lexer = Lexer::new(p)?;

        Ok(Self { lexer })
    }
}

fn main() -> io::Result<()> {
    let mut parser = Parser::new("hello-world.pdf")?;

    dbg!(parser.lexer.run().unwrap());

    // let mut line = Vec::new();

    // loop {
    //     line.clear();
    //     if file.read_vectored(b'\n', &mut line)? == 0 {
    //         break;
    //     }

    //     if line.starts_with(b"%%EOF") {
    //         break;
    //     }

    //     if line.starts_with(b"%") {
    //         continue;
    //     }
    // }

    // for line in file.lines() {

    // }

    Ok(())
}
