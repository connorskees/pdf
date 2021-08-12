use core::panic;
use std::{borrow::Cow, collections::HashMap, convert::TryFrom};

use crate::{error::PdfResult, lex::LexBase};

pub(crate) use error::{PostScriptError, PostScriptResult};

use self::{
    font::Type1PostscriptFont,
    lexer::PostScriptLexer,
    object::{
        Access, ArrayIndex, Container, DictionaryIndex, PostScriptArray, PostScriptDictionary,
        PostScriptObject, PostScriptString, Procedure, StringIndex,
    },
};

pub mod charstring;
mod decode;
mod error;
pub mod font;
mod lexer;
mod object;

#[derive(Debug)]
pub(crate) struct PostscriptInterpreter<'a> {
    lexer: PostScriptLexer<'a>,

    // We must maintain references to composite objects, rather than storing them
    // by value
    arrays: Container<ArrayIndex, PostScriptArray>,
    dictionaries: Container<DictionaryIndex, PostScriptDictionary>,

    operand_stack: Vec<PostScriptObject>,
    dictionary_stack: Vec<DictionaryIndex>,
    execution_stack: Vec<()>,

    pub fonts: HashMap<PostScriptString, Type1PostscriptFont>,
}

fn gen_system_dict() -> PostScriptDictionary {
    let mut system_dict = PostScriptDictionary::new();

    system_dict.insert(
        PostScriptString::from_bytes(b"Abs".to_vec()),
        PostScriptObject::Operator(PostscriptOperator::Abs),
    );
    system_dict.insert(
        PostScriptString::from_bytes(b"Add".to_vec()),
        PostScriptObject::Operator(PostscriptOperator::Add),
    );
    system_dict.insert(
        PostScriptString::from_bytes(b"Dict".to_vec()),
        PostScriptObject::Operator(PostscriptOperator::Dict),
    );
    system_dict.insert(
        PostScriptString::from_bytes(b"Begin".to_vec()),
        PostScriptObject::Operator(PostscriptOperator::Begin),
    );
    system_dict.insert(
        PostScriptString::from_bytes(b"Dup".to_vec()),
        PostScriptObject::Operator(PostscriptOperator::Dup),
    );
    system_dict.insert(
        PostScriptString::from_bytes(b"Def".to_vec()),
        PostScriptObject::Operator(PostscriptOperator::Def),
    );
    system_dict.insert(
        PostScriptString::from_bytes(b"ReadOnly".to_vec()),
        PostScriptObject::Operator(PostscriptOperator::ReadOnly),
    );
    system_dict.insert(
        PostScriptString::from_bytes(b"ExecuteOnly".to_vec()),
        PostScriptObject::Operator(PostscriptOperator::ExecuteOnly),
    );
    system_dict.insert(
        PostScriptString::from_bytes(b"NoAccess".to_vec()),
        PostScriptObject::Operator(PostscriptOperator::NoAccess),
    );
    system_dict.insert(
        PostScriptString::from_bytes(b"False".to_vec()),
        PostScriptObject::Operator(PostscriptOperator::False),
    );
    system_dict.insert(
        PostScriptString::from_bytes(b"True".to_vec()),
        PostScriptObject::Operator(PostscriptOperator::True),
    );
    system_dict.insert(
        PostScriptString::from_bytes(b"End".to_vec()),
        PostScriptObject::Operator(PostscriptOperator::End),
    );
    system_dict.insert(
        PostScriptString::from_bytes(b"CurrentFile".to_vec()),
        PostScriptObject::Operator(PostscriptOperator::CurrentFile),
    );
    system_dict.insert(
        PostScriptString::from_bytes(b"EExec".to_vec()),
        PostScriptObject::Operator(PostscriptOperator::EExec),
    );
    system_dict.insert(
        PostScriptString::from_bytes(b"ArrayStart".to_vec()),
        PostScriptObject::Operator(PostscriptOperator::ArrayStart),
    );
    system_dict.insert(
        PostScriptString::from_bytes(b"ArrayEnd".to_vec()),
        PostScriptObject::Operator(PostscriptOperator::ArrayEnd),
    );
    system_dict.insert(
        PostScriptString::from_bytes(b"ProcedureStart".to_vec()),
        PostScriptObject::Operator(PostscriptOperator::ProcedureStart),
    );
    system_dict.insert(
        PostScriptString::from_bytes(b"ProcedureEnd".to_vec()),
        PostScriptObject::Operator(PostscriptOperator::ProcedureEnd),
    );
    system_dict.insert(
        PostScriptString::from_bytes(b"CurrentDict".to_vec()),
        PostScriptObject::Operator(PostscriptOperator::CurrentDict),
    );
    system_dict.insert(
        PostScriptString::from_bytes(b"String".to_vec()),
        PostScriptObject::Operator(PostscriptOperator::String),
    );
    system_dict.insert(
        PostScriptString::from_bytes(b"Exch".to_vec()),
        PostScriptObject::Operator(PostscriptOperator::Exch),
    );
    system_dict.insert(
        PostScriptString::from_bytes(b"ReadString".to_vec()),
        PostScriptObject::Operator(PostscriptOperator::ReadString),
    );
    system_dict.insert(
        PostScriptString::from_bytes(b"Pop".to_vec()),
        PostScriptObject::Operator(PostscriptOperator::Pop),
    );
    system_dict.insert(
        PostScriptString::from_bytes(b"Put".to_vec()),
        PostScriptObject::Operator(PostscriptOperator::Put),
    );

    system_dict
}

#[rustfmt::skip]
static STANDARD_ENCODING: &[Option<&str>] = &[
    /*\00x*/ None, None, None, None, None, None, None, None,
    /*\01x*/ None, None, None, None, None, None, None, None,
    /*\02x*/ None, None, None, None, None, None, None, None,
    /*\03x*/ None, None, None, None, None, None, None, None,
    /*\04x*/ Some("space"), Some("exclam"), Some("quotedbl"), Some("numbersign"),
             Some("dollar"), Some("percent"), Some("ampersand"), Some("quoteright"),
    /*\05x*/ Some("parenleft"), Some("parenright"), Some("asterisk"), Some("plus"),
             Some("comma"), Some("hyphen"), Some("period"), Some("slash"),
    /*\06x*/ Some("zero"), Some("one"), Some("two"), Some("three"),
             Some("four"), Some("five"), Some("six"), Some("seven"),
    /*\07x*/ Some("eight"), Some("nine"), Some("colon"), Some("semicolon"),
             Some("less"), Some("equal"), Some("greater"), Some("question"),
    /*\10x*/ Some("at"), Some("A"), Some("B"), Some("C"),
             Some("D"), Some("E"), Some("F"), Some("G"),
    /*\11x*/ Some("H"), Some("I"), Some("J"), Some("K"),
             Some("L"), Some("M"), Some("N"), Some("O"),
    /*\12x*/ Some("P"), Some("Q"), Some("R"), Some("S"),
             Some("T"), Some("U"), Some("V"), Some("W"),
    /*\13x*/ Some("X"), Some("Y"), Some("Z"), Some("bracketleft"),
             Some("backslash"), Some("bracketright"), Some("asciicircum"), Some("underscore"),
    /*\14x*/ Some("quoteleft"), Some("a"), Some("b"), Some("c"),
             Some("d"), Some("e"), Some("f"), Some("g"),
    /*\15x*/ Some("h"), Some("i"), Some("j"), Some("k"),
             Some("l"), Some("m"), Some("n"), Some("o"),
    /*\16x*/ Some("p"), Some("q"), Some("r"), Some("s"),
             Some("t"), Some("u"), Some("v"), Some("w"),
    /*\17x*/ Some("x"), Some("y"), Some("z"), Some("braceleft"),
             Some("bar"), Some("braceright"), Some("asciitilde"), None,
    /*\20x*/ None, None, None, None, None, None, None, None,
    /*\21x*/ None, None, None, None, None, None, None, None,
    /*\22x*/ None, None, None, None, None, None, None, None,
    /*\23x*/ None, None, None, None, None, None, None, None,
    /*\24x*/ None, Some("exclamdown"), Some("cent"), Some("sterling"),
             Some("fraction"), Some("yen"), Some("florin"), Some("section"),
    /*\25x*/ Some("currency"), Some("quotesingle"), Some("quotedblleft"), Some("guillemotleft"),
             Some("guilsinglleft"), Some("guilsinglright"), Some("fi"), Some("fl"),
    /*\26x*/ None, Some("endash"), Some("dagger"), Some("daggerdbl"),
             Some("periodcentered"), None, Some("paragraph"), Some("bullet"),
    /*\27x*/ Some("quotesinglbase"), Some("quotedblbase"), Some("quotedblright"), Some("guillemotright"),
             Some("ellipsis"), Some("perthousand"), None, Some("questiondown"),
    /*\30x*/ None, Some("grave"), Some("acute"), Some("circumflex"),
             Some("tilde"), Some("macron2"), Some("breve"), Some("dotaccent"),
    /*\31x*/ Some("dieresis"), None, Some("ring"), Some("cedilla"),
             None, Some("hungarumlaut"), Some("ogonek"), Some("caron"),
    /*\32x*/ Some("emdash"), None, None, None, None, None, None, None,
    /*\33x*/ None, None, None, None, None, None, None, None,
    /*\34x*/ None, Some("AE"), None, Some("ordfeminine"), None, None, None, None,
    /*\35x*/ Some("Lslash"), Some("Oslash"), Some("oe"), Some("ordmasculine"), None, None, None, None,
    /*\36x*/ None, Some("ae"), None, None, None, Some("dotlessi"), None, None,
    /*\37x*/ Some("lslash"), Some("oslash"), Some("OE"), Some("germandbls"), None, None, None, None,
];

fn gen_standard_encoding_vector(interpreter: &mut PostscriptInterpreter) -> PostScriptArray {
    PostScriptArray::from_objects(
        STANDARD_ENCODING
            .iter()
            .map(|name| match name {
                &Some(s) => interpreter
                    .intern_string(PostScriptString::from_bytes(s.to_owned().into_bytes())),
                None => PostScriptObject::Null,
            })
            .collect(),
    )
}

impl<'a> PostscriptInterpreter<'a> {
    pub fn new(buffer: &'a [u8]) -> Self {
        let mut interpreter = Self {
            lexer: PostScriptLexer::new(Cow::Borrowed(buffer)),
            operand_stack: Vec::new(),
            dictionary_stack: Vec::new(),
            execution_stack: Vec::new(),
            arrays: Container::new(),
            dictionaries: Container::new(),
            fonts: HashMap::new(),
        };

        let system_dict = interpreter.new_dict(gen_system_dict());
        let global_dict = interpreter.new_dict(PostScriptDictionary::new());
        let user_dict = {
            let mut dict = PostScriptDictionary::new();

            let standard_encoding = gen_standard_encoding_vector(&mut interpreter);

            dict.insert(
                PostScriptString::from_bytes(b"StandardEncoding".to_vec()),
                PostScriptObject::Array(interpreter.arrays.insert(standard_encoding)),
            );

            dict.insert(
                PostScriptString::from_bytes(b"systemdict".to_vec()),
                PostScriptObject::Dictionary(system_dict),
            );

            interpreter.new_dict(dict)
        };

        interpreter.push_dict_stack(system_dict);
        interpreter.push_dict_stack(global_dict);
        interpreter.push_dict_stack(user_dict);

        interpreter
    }

    pub fn run(&mut self) -> PdfResult<()> {
        while let Some(obj) = self.lexer.next() {
            self.execute_token(obj?)?;
        }

        Ok(())
    }

    fn execute_token(&mut self, tok: PostScriptObject) -> PdfResult<()> {
        match tok {
            PostScriptObject::Operator(op) => self.execute(op)?,
            PostScriptObject::Literal(lit) => {
                let obj = self.get_key(&lit)?;

                match obj {
                    PostScriptObject::Procedure(proc) => {
                        self.execute_procedure(proc)?;
                    }
                    obj => self.push(obj),
                }
            }
            obj => self.push(obj),
        }

        Ok(())
    }

    fn execute(&mut self, op: PostscriptOperator) -> PdfResult<()> {
        match op {
            PostscriptOperator::Dict => self.dict(),
            PostscriptOperator::Begin => self.begin(),
            PostscriptOperator::Dup => self.dup(),
            PostscriptOperator::Def => self.def(),
            PostscriptOperator::False => Ok(self.push(PostScriptObject::Bool(false))),
            PostscriptOperator::True => Ok(self.push(PostScriptObject::Bool(true))),
            PostscriptOperator::End => self.end(),
            PostscriptOperator::ReadOnly => self.modify_access(Access::ReadOnly),
            PostscriptOperator::ExecuteOnly => self.modify_access(Access::ExecuteOnly),
            PostscriptOperator::NoAccess => self.modify_access(Access::None),
            PostscriptOperator::Array => self.array(),
            PostscriptOperator::ArrayStart => self.array_start(),
            PostscriptOperator::ArrayEnd => self.array_end(),
            PostscriptOperator::ProcedureStart => self.procedure_start(),
            PostscriptOperator::ProcedureEnd => todo!(),
            PostscriptOperator::CurrentDict => self.current_dict(),
            PostscriptOperator::CurrentFile => self.current_file(),
            PostscriptOperator::EExec => self.eexec(),
            PostscriptOperator::String => self.string(),
            PostscriptOperator::Exch => self.exch(),
            PostscriptOperator::ReadString => self.readstring(),
            PostscriptOperator::Pop => {
                self.pop()?;

                Ok(())
            }
            PostscriptOperator::Put => self.put(),
            PostscriptOperator::Known => self.known(),
            PostscriptOperator::Not => self.not(),
            PostscriptOperator::Get => self.get(),
            PostscriptOperator::Exec => self.exec(),
            PostscriptOperator::IfElse => self.if_else(),
            PostscriptOperator::Lt => self.lt(),
            PostscriptOperator::Index => self.index(),
            PostscriptOperator::DefineFont => self.define_font(),
            PostscriptOperator::Mark => self.mark(),
            PostscriptOperator::CloseFile => self.close_file(),
            op @ (PostscriptOperator::Abs | PostscriptOperator::Add) => todo!("{:?}", op),
        }
    }

    fn execute_procedure(&mut self, proc: Procedure) -> PdfResult<()> {
        for tok in proc.inner {
            self.execute_token(tok)?;
        }

        Ok(())
    }

    fn define_font(&mut self) -> PdfResult<()> {
        let font = self.pop_dict()?;
        let key = self.pop_name()?;

        let dict = self.get_dict(font).clone();

        let font_dict = Type1PostscriptFont::from_dict(dict, self)?;

        self.fonts.insert(key, font_dict);

        self.push(PostScriptObject::Dictionary(font));

        Ok(())
    }

    fn index(&mut self) -> PdfResult<()> {
        let n = usize::try_from(self.pop_int()?)?;

        let mut objs = Vec::new();

        for _ in 0..n {
            objs.push(self.pop()?);
        }

        let end = self.pop()?;

        objs.push(end.clone());

        objs.reverse();

        for obj in objs {
            self.push(obj);
        }

        self.push(end);

        Ok(())
    }

    fn array(&mut self) -> PdfResult<()> {
        let len = usize::try_from(self.pop_int()?)?;

        self.push_arr(vec![PostScriptObject::Null; len]);

        Ok(())
    }

    fn lt(&mut self) -> PdfResult<()> {
        match self.pop()? {
            PostScriptObject::Int(i2) => {
                let i1 = self.pop_int()?;

                self.push(PostScriptObject::Bool(i1 < i2));
            }
            PostScriptObject::String(s2) => {
                let s1 = self.pop_string()?;

                self.push(PostScriptObject::Bool(self.get_str(s1) < self.get_str(s2)));
            }
            _ => return Err(PostScriptError::TypeCheck.into()),
        }

        Ok(())
    }

    fn if_else(&mut self) -> PdfResult<()> {
        let proc_two = self.pop_procedure()?;
        let proc_one = self.pop_procedure()?;
        let b = self.pop_bool()?;

        if b {
            self.execute_procedure(proc_one)?;
        } else {
            self.execute_procedure(proc_two)?;
        }

        Ok(())
    }

    fn exec(&mut self) -> PdfResult<()> {
        let obj = self.pop()?;

        self.execute_token(obj)
    }

    fn get(&mut self) -> PdfResult<()> {
        let key_or_idx = self.pop()?;
        let container = self.pop()?;

        match container {
            PostScriptObject::Array(arr) => {
                let idx = usize::try_from(match key_or_idx {
                    PostScriptObject::Int(i) => i,
                    _ => return Err(PostScriptError::TypeCheck.into()),
                })?;

                let val = self.get_arr_mut(arr).get(idx)?.clone();
                self.push(val);
            }
            PostScriptObject::Dictionary(dict) => {
                let key = match key_or_idx {
                    PostScriptObject::Name(name) => name,
                    _ => return Err(PostScriptError::TypeCheck.into()),
                };

                let val = self
                    .get_dict(dict)
                    .get(&key)
                    .ok_or(PostScriptError::Undefined { key })?
                    .clone();

                self.push(val);
            }
            PostScriptObject::String(s) => {
                let idx = usize::try_from(match key_or_idx {
                    PostScriptObject::Int(i) => i,
                    _ => return Err(PostScriptError::TypeCheck.into()),
                })?;

                let val = self.get_str_mut(s).get(idx)?;
                self.push(PostScriptObject::Int(i32::from(val)));
            }
            _ => return Err(PostScriptError::TypeCheck.into()),
        }

        Ok(())
    }

    fn not(&mut self) -> PdfResult<()> {
        match self.pop()? {
            PostScriptObject::Bool(b) => {
                self.push(PostScriptObject::Bool(!b));
            }
            PostScriptObject::Int(i) => {
                self.push(PostScriptObject::Int(!i));
            }
            _ => return Err(PostScriptError::TypeCheck.into()),
        }

        Ok(())
    }

    fn known(&mut self) -> PdfResult<()> {
        // todo: or name?
        let key = self.pop_string()?;
        let dict = self.pop_dict()?;

        self.push(PostScriptObject::Bool(
            self.get_dict(dict).contains(self.get_str(key)),
        ));

        Ok(())
    }

    fn put(&mut self) -> PdfResult<()> {
        let value = self.pop()?;
        let key_or_idx = self.pop()?;
        let composite_obj = self.pop()?;

        match composite_obj {
            PostScriptObject::String(s) => {
                let idx = usize::try_from(match key_or_idx {
                    PostScriptObject::Int(i) => i,
                    _ => return Err(PostScriptError::TypeCheck.into()),
                })?;

                let ch = u8::try_from(match value {
                    PostScriptObject::Int(i) => i,
                    _ => return Err(PostScriptError::TypeCheck.into()),
                })?;

                self.get_str_mut(s).put(idx, ch);
            }
            PostScriptObject::Dictionary(dict) => {
                let key = match key_or_idx {
                    PostScriptObject::Name(name) => name,
                    _ => return Err(PostScriptError::TypeCheck.into()),
                };

                self.get_dict_mut(&dict).insert(key, value);
            }
            PostScriptObject::Array(arr) => {
                let idx = usize::try_from(match key_or_idx {
                    PostScriptObject::Int(i) => i,
                    _ => return Err(PostScriptError::TypeCheck.into()),
                })?;

                self.get_arr_mut(arr).put(idx, value);
            }
            _ => return Err(PostScriptError::TypeCheck.into()),
        }

        Ok(())
    }

    fn readstring(&mut self) -> PdfResult<()> {
        let string = self.pop_string()?;
        let _file = self.pop_file()?;

        // self.lexer.skip_whitespace();
        let n = self.lexer.next_byte();

        assert_eq!(n, Some(b' '));

        let capacity = self.get_str(string).capacity();

        let (contents, found_eof) = self.lexer.get_range_from_cursor(capacity);

        // SAFETY: contents is never modified by extending a string
        // todo: not necessary at all
        let contents = unsafe { &*(contents as *const _) };
        self.get_str_mut(string).write(contents);

        *self.lexer.cursor_mut() += contents.len();

        self.push(PostScriptObject::String(string));
        self.push(PostScriptObject::Bool(found_eof));

        Ok(())
    }

    fn exch(&mut self) -> PdfResult<()> {
        let obj2 = self.pop()?;
        let obj1 = self.pop()?;

        self.push(obj2);
        self.push(obj1);

        Ok(())
    }

    fn string(&mut self) -> PdfResult<()> {
        let len = usize::try_from(self.pop_int()?)?;

        self.push_str(PostScriptString::with_capacity(len));

        Ok(())
    }

    fn eexec(&mut self) -> PdfResult<()> {
        let _file = self.pop_file()?;
        let buffer = self.lexer.buffer_from_cursor();

        // println!("{}", String::from_utf8_lossy(&decode::decrypt(buffer)[4..]));
        let decoded_buffer = decode::decrypt(buffer)[4..].to_vec();
        self.lexer.reset_buffer(Cow::Owned(decoded_buffer));

        Ok(())
    }

    fn current_file(&mut self) -> PdfResult<()> {
        // todo: execution stack? also implement files
        self.push(PostScriptObject::File);

        Ok(())
    }

    fn current_dict(&mut self) -> PdfResult<()> {
        let current_dict = self.pop_dict_stack()?;

        self.push(PostScriptObject::Dictionary(current_dict));
        self.push_dict_stack(current_dict);

        Ok(())
    }

    fn lex_procedure(&mut self) -> PdfResult<Vec<PostScriptObject>> {
        let mut objs = Vec::new();

        while let Some(tok) = self.lexer.next() {
            match tok? {
                PostScriptObject::Operator(PostscriptOperator::ProcedureStart) => {
                    objs.push(PostScriptObject::Procedure(Procedure::from_toks(
                        self.lex_procedure()?,
                    )));
                }
                PostScriptObject::Operator(PostscriptOperator::ProcedureEnd) => break,
                obj => objs.push(obj),
            }
        }

        Ok(objs)
    }

    fn procedure_start(&mut self) -> PdfResult<()> {
        let proc = self.lex_procedure()?;

        self.push(PostScriptObject::Procedure(Procedure::from_toks(proc)));

        Ok(())
    }

    fn array_end(&mut self) -> PdfResult<()> {
        let mut arr = Vec::new();

        loop {
            match self.pop()? {
                PostScriptObject::Mark => break,
                obj => arr.push(obj),
            }
        }

        self.push_arr(arr);

        Ok(())
    }

    fn array_start(&mut self) -> PdfResult<()> {
        self.push(PostScriptObject::Mark);

        Ok(())
    }

    fn modify_access(&mut self, access: Access) -> PdfResult<()> {
        let mut obj = self.pop()?;

        match obj {
            PostScriptObject::Dictionary(dict) => {
                self.get_dict_mut(&dict).set_access(access);

                obj = PostScriptObject::Dictionary(dict);
            }
            PostScriptObject::Procedure(mut procedure) => {
                procedure.set_access(access);

                obj = PostScriptObject::Procedure(procedure);
            }
            PostScriptObject::Array(arr) => {
                self.get_arr_mut(arr).set_access(access);

                obj = PostScriptObject::Array(arr);
            }
            PostScriptObject::String(s) => {
                self.get_str_mut(s).set_access(access);

                obj = PostScriptObject::String(s);
            }
            obj => todo!("make obj {:?}: {:?}", access, obj),
        }

        self.push(obj);

        Ok(())
    }

    fn end(&mut self) -> PdfResult<()> {
        self.pop_dict_stack()?;

        Ok(())
    }

    fn def(&mut self) -> PdfResult<()> {
        let value = self.pop()?;
        let key = self.pop_name()?;

        let dict = self.get_current_dict()?;

        self.get_dict_mut(&dict).insert(key, value);

        Ok(())
    }

    fn dup(&mut self) -> PdfResult<()> {
        let obj = self.pop()?;

        self.push(obj.clone());
        self.push(obj);

        Ok(())
    }

    fn begin(&mut self) -> PdfResult<()> {
        let dict = match self.pop()? {
            PostScriptObject::Dictionary(dict) => dict,
            _ => return Err(PostScriptError::TypeCheck.into()),
        };

        self.push_dict_stack(dict);

        Ok(())
    }

    fn dict(&mut self) -> PdfResult<()> {
        let n = match self.pop()? {
            PostScriptObject::Int(i) => usize::try_from(i),
            _ => return Err(PostScriptError::TypeCheck.into()),
        }?;

        let dict = self.new_dict(PostScriptDictionary::with_capacity(n));

        self.operand_stack.push(PostScriptObject::Dictionary(dict));

        Ok(())
    }

    /// Pushes a mark object on the operand stack
    ///
    /// All marks are identical, and the operand stack may contain any number
    /// of them at once
    fn mark(&mut self) -> PdfResult<()> {
        self.push(PostScriptObject::Mark);

        Ok(())
    }

    /// closes file, breaking the association between the file object and the
    /// underlying file
    ///
    /// For an output file, closefile first performs a flushfile operation. It
    /// may also take device-dependent actions, such as truncating a disk file
    /// to the current position or transmitting an end-of-file indication.
    /// Executing closefile on a file that has already been closed has no effect;
    /// it does not cause an error
    fn close_file(&mut self) -> PdfResult<()> {
        let _file = self.pop_file()?;

        Ok(())
    }
}

impl PostscriptInterpreter<'_> {
    fn push(&mut self, obj: PostScriptObject) {
        self.operand_stack.push(obj);
    }

    fn push_arr(&mut self, arr: Vec<PostScriptObject>) {
        let idx = self.arrays.insert(PostScriptArray::from_objects(arr));
        self.operand_stack.push(PostScriptObject::Array(idx));
    }

    fn get_arr(&mut self, k: ArrayIndex) -> &PostScriptArray {
        self.arrays.get(&k).unwrap()
    }

    fn get_arr_mut(&mut self, k: ArrayIndex) -> &mut PostScriptArray {
        self.arrays.get_mut(&k).unwrap()
    }

    fn push_str(&mut self, str: PostScriptString) {
        assert!(!str.is_empty(), "implies a bug. TODO: remove");

        let idx = self.lexer.strings.insert(str);
        self.operand_stack.push(PostScriptObject::String(idx));
    }

    fn intern_string(&mut self, str: PostScriptString) -> PostScriptObject {
        assert!(!str.is_empty(), "implies a bug. TODO: remove");

        let idx = self.lexer.strings.insert(str);
        PostScriptObject::String(idx)
    }

    fn get_str(&self, k: StringIndex) -> &PostScriptString {
        self.lexer.strings.get(&k).unwrap()
    }

    fn get_str_mut(&mut self, k: StringIndex) -> &mut PostScriptString {
        self.lexer.strings.get_mut(&k).unwrap()
    }

    fn pop(&mut self) -> PostScriptResult<PostScriptObject> {
        self.operand_stack
            .pop()
            .ok_or(PostScriptError::StackUnderflow)
    }

    fn pop_int(&mut self) -> PdfResult<i32> {
        match self.pop()? {
            PostScriptObject::Int(i) => Ok(i),
            _ => Err(PostScriptError::TypeCheck.into()),
        }
    }

    fn pop_name(&mut self) -> PdfResult<PostScriptString> {
        match self.pop()? {
            PostScriptObject::Name(name) => Ok(name),
            _ => Err(PostScriptError::TypeCheck.into()),
        }
    }

    fn pop_string(&mut self) -> PdfResult<StringIndex> {
        match self.pop()? {
            PostScriptObject::String(s) => Ok(s),
            _ => Err(PostScriptError::TypeCheck.into()),
        }
    }

    fn pop_dict(&mut self) -> PdfResult<DictionaryIndex> {
        match self.pop()? {
            PostScriptObject::Dictionary(dict) => Ok(dict),
            _ => Err(PostScriptError::TypeCheck.into()),
        }
    }

    fn pop_bool(&mut self) -> PdfResult<bool> {
        match self.pop()? {
            PostScriptObject::Bool(b) => Ok(b),
            _ => Err(PostScriptError::TypeCheck.into()),
        }
    }

    fn pop_procedure(&mut self) -> PdfResult<Procedure> {
        match self.pop()? {
            PostScriptObject::Procedure(proc) => Ok(proc),
            _ => Err(PostScriptError::TypeCheck.into()),
        }
    }

    fn pop_file(&mut self) -> PdfResult<PostScriptObject> {
        match self.pop()? {
            obj @ PostScriptObject::File => Ok(obj),
            _ => Err(PostScriptError::TypeCheck.into()),
        }
    }

    fn new_dict(&mut self, dict: PostScriptDictionary) -> DictionaryIndex {
        self.dictionaries.insert(dict)
    }

    fn push_dict_stack(&mut self, dict: DictionaryIndex) {
        self.dictionary_stack.push(dict);
    }

    fn pop_dict_stack(&mut self) -> PostScriptResult<DictionaryIndex> {
        self.dictionary_stack
            .pop()
            .ok_or(PostScriptError::DictStackUnderflow)
    }

    fn get_current_dict(&mut self) -> PostScriptResult<DictionaryIndex> {
        self.dictionary_stack
            .last()
            .cloned()
            .ok_or(PostScriptError::DictStackUnderflow)
    }

    fn get_dict(&self, key: DictionaryIndex) -> &PostScriptDictionary {
        self.dictionaries.get(&key).unwrap()
    }

    fn get_dict_mut(&mut self, key: &DictionaryIndex) -> &mut PostScriptDictionary {
        self.dictionaries.get_mut(key).unwrap()
    }

    fn get_key(&mut self, key: &PostScriptString) -> PdfResult<PostScriptObject> {
        for &dict in self.dictionary_stack.iter().rev() {
            if let Some(obj) = self.get_dict(dict).get(key) {
                return Ok(obj.clone());
            }
        }

        Err(PostScriptError::Undefined { key: key.clone() }.into())
    }
}

#[derive(Debug, Clone, Copy)]
pub(self) enum PostscriptOperator {
    Abs,
    Add,
    Dict,
    Begin,
    Dup,
    Def,
    ReadOnly,
    ExecuteOnly,
    NoAccess,
    False,
    True,
    End,
    CurrentFile,
    EExec,
    Array,
    ArrayStart,
    ArrayEnd,
    ProcedureStart,
    ProcedureEnd,
    CurrentDict,
    String,
    Exch,
    ReadString,
    Pop,
    Put,
    Known,
    Not,
    Get,
    Exec,
    IfElse,
    Lt,
    Index,
    DefineFont,
    Mark,
    CloseFile,
}

#[derive(Debug, Clone, Copy)]
pub(super) enum GraphicsOperator {
    // Starting and finishing
    /// Finishes a charstring outline definition and must be the last command
    /// in a character’s outline (except for accented characters defined using
    /// seac). When endchar is executed, Type 1 BuildChar performs several tasks.
    ///
    /// It executes a setcachedevice operation, using a bounding box it computes
    /// directly from the character outline and using the width information
    /// acquired from a previous hsbw or sbw operation. (Note that this is not
    /// the same order of events as in Type 3 Fonts.) BuildChar then calls a
    /// special version of fill or stroke depending on the value of PaintType
    /// in the font dictionary. The Type 1 font format supports only PaintType
    /// 0 (fill) and 2 (outline). Note that this single fill or stroke implies
    /// that there can be only one path (possibly containing several subpaths)
    /// that can be created to be filled or stroked by the endchar command
    EndChar,

    /// Sets the left sidebearing point at (sbx, 0) and sets the character width
    /// vector to (wx, 0) in character space. This command also sets the current
    /// point to (sbx, 0), but does not place the point in the character path.
    ///
    /// Use rmoveto for the first point in the path. The name hsbw stands for
    /// horizontal sidebearing and width; horizontal indicates that the y component
    /// of both the sidebearing and width is 0. Either sbw or hsbw must be used
    /// once as the first command in a character outline definition. It must be
    /// used only once. In non-marking characters, such as the space character,
    /// the left sidebearing point should be (0, 0)
    HorizontalSideBearingWidth,

    /// makes an accented character from two other characters in its font program.
    ///
    /// The asb argument is the x component of the left sidebearing of the accent;
    /// this value must be the same as the sidebearing value given in the hsbw
    /// or sbw command in the accent’s own charstring. The origin of the accent
    /// is placed at (adx, ady) relative to the origin of the base character. The
    /// bchar argument is the character code of the base character, and the achar
    /// argument is the character code of the accent character. Both bchar and
    /// achar are codes that these characters are assigned in the Adobe StandardEncoding
    /// vector, given in an Appendix in the PostScript Language Reference Manual.
    ///
    /// Furthermore, the characters represented by achar and bchar must be in the
    /// same positions in the font’s encoding vector as the positions they occupy
    /// in the Adobe StandardEncoding vector. If the name of both components of
    /// an accented character do not appear in the Adobe StandardEncoding vector,
    /// the accented character cannot be built using the seac command
    ///
    /// The FontBBox entry in the font dictionary must be large enough to accommodate
    /// both parts of the accented character. The sbw or hsbw command that begins
    /// the accented character must be the same as the corresponding command in
    /// the base character. Finally, seac is the last command in the charstring
    /// for the accented character because the accent and base characters’ charstrings
    /// each already end with their own endchar commands
    ///
    /// The use of this command saves space in a Type 1 font program, but its use
    /// is restricted to those characters whose parts are defined in the Adobe
    /// StandardEncoding vector. In situations where use of the seac command is
    /// not possible, use of Subrs subroutines is a more general means for creating
    /// accented characters
    StandardEncodingAccentedCharacter,

    /// sets the left sidebearing point to (sbx, sby) and sets the character
    /// width vector to (wx, wy) in character space. This command also sets the
    /// current point to (sbx, sby), but does not place the point in the character
    /// path. Use rmoveto for the first point in the path. The name sbw stands
    /// for sidebearing and width; the x and y components of both the left
    /// sidebearing and width must be specified. If the y components of both the
    /// left sidebearing and the width are 0, then the hsbw command should be used.
    ///
    /// Either sbw or hsbw must be used once as the first command in a character
    /// outline definition. It must be used only once
    SideBearingWidth,

    // Path construction
    /// `closepath` closes a subpath. Adobe strongly recommends that all character
    /// subpaths end with a `closepath` command, otherwise when an outline is stroked
    /// (by setting PaintType equal to 2) you may get unexpected behavior where
    /// lines join. Note that, unlike the `closepath` command in the PostScript
    /// language, this command does not reposition the current point. Any subsequent
    /// rmoveto must be relative to the current point in force before the Type
    /// 1 font format `closepath` command was given. Make sure that any subpath
    /// section formed by the `closepath` command intended to be zero length, is
    /// zero length. If not, the `closepath` command may cause a “spike” or “hangnail”
    /// (if the subpath doubles back onto itself) with unexpected results
    ClosePath,

    /// Equivalent to `dx 0 rlineto`
    HorizontalLineTo,

    /// Equivalent to `dx 0 rmoveto`
    HorizontalMoveTo,

    /// Equivalent to `dx1 0 dx2 dy2 0 dy3 rrcurveto`
    ///
    /// This command eliminates two arguments from an rrcurveto call when the
    /// first Bézier tangent is horizontal and the second Bézier tangent is
    /// vertical
    HorizontalVerticalCurveTo,

    /// appends a straight line segment to the current path, starting from the
    /// current point and extending dx user space units horizontally and dy units
    /// vertically. That is, the operands dx and dy are interpreted as relative
    /// displacements from the current point rather than as absolute coordinates.
    ///
    /// In all other respects, the behavior of rlineto is identical to that of lineto.
    ///
    /// If the current point is undefined because the current path is empty, a
    /// `nocurrentpoint` error occurs
    RelativeLineTo,

    /// starts a new subpath of the current path by displacing the coordinates
    /// of the current point dx user space units horizontally and dy units
    /// vertically, without connecting it to the previous current point. That
    /// is, the operands dx and dy are interpreted as relative displacements
    /// from the current point rather than as absolute coordinates. In all other
    /// respects, the behavior of rmoveto is identical to that of moveto
    ///
    /// If the current point is undefined because the current path is empty, a
    /// `nocurrentpoint` error occurs
    RelativeMoveTo,

    /// Whereas the arguments to the rcurveto operator in the PostScript language
    /// are all relative to the current point, the arguments to rrcurveto are
    /// relative to each other.
    ///
    /// Equivalent to `dx1 dy1 (dx1+dx2) (dy1+dy2) (dx1+dx2+dx3) (dy1+dy2+dy3) rcurveto`
    ///
    /// `rcurveto` docs:
    /// appends a section of a cubic Bézier curve to the current path in the same
    /// manner as curveto. However, the operands are interpreted as relative
    /// displacements from the current point rather than as absolute coordinates.
    /// That is, rcurveto constructs a curve between the current point (x0, y0)
    /// and the endpoint (x0 + dx3, y0 + dy3), using (x0 + dx1, y0 + dy1) and
    /// (x0 + dx2, y0 + dy2) as the Bézier control points. In all other respects,
    /// the behavior of rcurveto is identical to that of curveto
    ///
    /// `curveto` docs:
    /// appends a section of a cubic Bézier curve to the current path between the
    /// current point (x0, y0) and the endpoint (x3, y3), using (x1, y1) and (x2,
    /// y2) as the Bézier control points. The endpoint (x3, y3) becomes the new
    /// current point. If the current point is undefined because the current path
    /// is empty, a nocurrentpoint error occurs.
    RelativeRelativeCurveTo,

    /// Equivalent to `0 dy1 dx2 dy2 dx3 0 rrcurveto`.
    ///
    /// This command eliminates two arguments from an `rrcurveto` call when the
    /// first Bézier tangent is vertical and the second Bézier tangent is
    /// horizontal
    VerticalHorizontalCurveTo,

    /// Equivalent to `0 dy rlineto`
    VerticalLineTo,

    /// Equivalent to `0 dy rmoveto`
    VerticalMoveTo,

    // Hint commands
    /// brackets an outline section for the dots in letters such as “i”,“ j”,
    /// and “!”. This is a hint command that indicates that a section of a charstring
    /// should be understood as describing such a feature, rather than as part
    /// of the main outline
    DotSection,

    /// declares the vertical range of a horizontal stem zone between the y
    /// coordinates y and y+dy, where y is relative to the y coordinate of the
    /// left sidebearing point. Horizontal stem zones within a set of stem hints
    /// for a single character may not overlap other horizontal stem zones. Use
    /// hint replacement to avoid stem hint overlaps
    HorizontalStem,

    /// declares the vertical ranges of three horizontal stem zones between the
    /// y coordinates `y0` and `y0 + dy0`, `y1` and `y1 + dy1`, and between `y2`
    /// and `y2 + dy2`, where `y0`, `y1` and `y2` are all relative to the y
    /// coordinate of the left sidebearing point. The hstem3 command sorts these
    /// zones by the y values to obtain the lowest, middle and highest zones,
    /// called ymin, ymid and ymax respectively. The corresponding dy values are
    /// called dymin, dymid and dymax. These stems and the counters between them
    /// will all be controlled. These coordinates must obey certain restrictions:
    ///
    ///     • dymin = dymax
    ///
    ///     • The distance from ymin + dymin/2 to ymid + dymid/2 must equal the
    ///       distance from ymid + dymid/2 to ymax + dymax/2. In other words,
    ///       the distance from the center of the bottom stem to the center of
    ///       the middle stem must be the same as the distance from the center
    ///       of the middle stem to the center of the top stem.
    ///
    /// If a charstring uses an hstem3 command in the hints for a character, the
    /// charstring must not use hstem commands and it must use the same hstem3
    /// command consistently if hint replacement is performed.
    ///
    /// The hstem3 command is especially suited for controlling the stems and
    /// counters of symbols with three horizontally oriented features with equal
    /// vertical widths and with equal white space between these features, such
    /// as the mathematical equivalence symbol or the division symbol.
    HorizontalStem3,

    /// declares the horizontal range of a vertical stem zone between the x
    /// coordinates x and x+dx, where x is relative to the x coordinate of the
    /// left sidebearing point. Vertical stem zones within a set of stem hints
    /// for a single character may not overlap other vertical stem zones. Use
    /// hint replacement to avoid stem hint overlap
    VerticalStem,

    /// declares the horizontal ranges of three vertical stem zones between the
    /// x coordinates x0 and x0 + dx0, x1 and x1 + dx1, and x2 and x2 + dx2, where
    /// x0, x1 and x2 are all relative to the x coordinate of the left sidebearing
    /// point. The vstem3 command sorts these zones by the x values to obtain the
    /// leftmost, middle and rightmost zones, called xmin, xmid and xmax respectively.
    /// The corresponding dx values are called dxmin, dxmid and dxmax. These stems
    /// and the counters between them will all be controlled. These coordinates
    /// must obey certain restrictions described as follows:
    ///
    ///     • dxmin = dxmax
    ///
    ///     • The distance from xmin + dxmin/2 to xmid + dxmid/2 must equal the
    ///       distance from xmid + dxmid/2 to xmax + dxmax/2. In other words, the
    ///       distance from the center of the left stem to the center of the
    ///       middle stem must be the same as the distance from the center of the
    ///       middle stem to the center of the right stem
    ///
    /// If a charstring uses a vstem3 command in the hints for a character, the
    /// charstring must not use vstem commands and it must use the same vstem3
    /// command consistently if hint replacement is performed
    ///
    /// The vstem3 command is especially suited for controlling the stems and
    /// counters of characters such as a lower case “m.”
    VerticalStem3,

    // Arithmetic
    /// divides `num1` by `num2`, producing a result that is always a real number
    /// even if both operands are integers
    Div,

    // Subroutine
    /// a mechanism used by Type 1 BuildChar to make calls on the PostScript
    /// interpreter. Arguments argn through arg1 are pushed onto the PostScript
    /// interpreter operand stack, and the PostScript language procedure in the
    /// othersubr# position in the OtherSubrs array in the Private dictionary (or
    /// a built-in function equivalent to this procedure) is executed. Note that
    /// the argument order will be reversed when pushed onto the PostScript
    /// interpreter operand stack. After the arguments are pushed onto the
    /// PostScript interpreter operand stack, the PostScript interpreter performs
    /// a begin operation on systemdict followed by a begin operation on the font
    /// dictionary prior to executing the OtherSubrs entry. When the OtherSubrs
    /// entry completes its execution, the PostScript interpreter performs two
    /// end operations prior to returning to Type 1 BuildChar charstring execution.
    ///
    /// Use pop commands to retrieve results from the PostScript operand stack
    /// back to the Type 1 BuildChar operand stack
    CallOtherSubroutine,

    /// calls a charstring subroutine with index subr# from the Subrs array in
    /// the Private dictionary. Each element of the Subrs array is a charstring
    /// encoded and encrypted like any other charstring. Arguments pushed on the
    /// Type 1 BuildChar operand stack prior to calling the subroutine, and results
    /// pushed on this stack by the subroutine, act according to the manner in
    /// which the subroutine is coded. These subroutines are generally used to
    /// encode sequences of path commands that are repeated throughout the font
    /// program, for example, serif outline sequences. Subroutine calls may be
    /// nested 10 deep
    CallSubroutine,

    /// removes a number from the top of the PostScript interpreter operand stack
    /// and pushes that number onto the Type 1 BuildChar operand stack. This
    /// command is used only to retrieve a result from an OtherSubrs procedure
    Pop,

    /// returns from a Subrs array charstring subroutine (that had been called
    /// with a callsubr command) and continues execution in the calling charstring
    Return,

    /// sets the current point in the Type 1 font format BuildChar to (x, y) in
    /// absolute character space coordinates without performing a charstring
    /// moveto command. This establishes the current point for a subsequent relative
    /// path building command. The setcurrentpoint command is used only in
    /// conjunction with results from OtherSubrs procedures
    SetCurrentPoint,
}
