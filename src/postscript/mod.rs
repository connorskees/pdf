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

mod decode;
mod error;
mod font;
mod lexer;
mod object;

#[derive(Debug)]
pub struct PostscriptInterpreter<'a> {
    lexer: PostScriptLexer<'a>,

    // We must maintain references to composite objects
    arrays: Container<ArrayIndex, PostScriptArray>,
    dictionaries: Container<DictionaryIndex, PostScriptDictionary>,

    operand_stack: Vec<PostScriptObject>,
    dictionary_stack: Vec<DictionaryIndex>,
    execution_stack: Vec<()>,

    fonts: HashMap<PostScriptString, Type1PostscriptFont>,
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

            dict.insert(
                PostScriptString::from_bytes(b"StandardEncoding".to_vec()),
                PostScriptObject::Array(interpreter.arrays.insert(PostScriptArray::new())),
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

    // TODO: OPERANDS SWAPPED
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

                let ch = u8::try_from(match value.clone() {
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

                self.get_dict_mut(&dict).insert(key, value.clone());
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

        println!("{}", String::from_utf8_lossy(&decode::decrypt(buffer)[4..]));
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

        self.push(PostScriptObject::Dictionary(current_dict.clone()));
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

#[derive(Debug)]
enum GraphicsOperator {
    // Starting and finishing
    EndChar,
    HorizontalSideBearingWidth,
    StandardEncodingAccentedCharacter,
    SideBearingWidth,

    // Path construction
    ClosePath,
    HorizontalLineTo,
    HorizontalMoveTo,
    HorizontalVerticalCurveTo,
    RelativeLineTo,
    RelativeMoveTo,
    RelativeRelativeCurveTo,
    VerticalHorizontalCurveTo,
    VerticalLineTo,
    VerticalMoveTo,

    // Hint commands
    DotSection,
    HorizontalStem,
    HorizontalStem3,
    VerticalStem,
    VerticalStem3,

    // Arithmetic
    Div,

    // Subroutine
    CallOtherSubroutine,
    CallSubroutine,
    Pop,
    Return,
    SetCurrentPoint,
}
