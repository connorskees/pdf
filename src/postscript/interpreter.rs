use std::{
    borrow::Cow,
    collections::HashMap,
    ops::{Add, Div, Mul, Sub},
};

use crate::{error::PdfResult, lex::LexBase};

use super::{
    builtin::{gen_standard_encoding_vector, gen_system_dict},
    decode,
    error::{PostScriptError, PostScriptResult},
    font::Type1PostscriptFont,
    lexer::{ident_token_from_bytes, PostScriptLexer},
    object::{
        Access, ArrayIndex, Container, DictionaryIndex, PostScriptArray, PostScriptDictionary,
        PostScriptObject, PostScriptString, StringIndex,
    },
    operator::PostscriptOperator,
};

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

    resources: PostScriptDictionary,

    pub fonts: HashMap<PostScriptString, Type1PostscriptFont>,

    /// Whether or not we're interpreting an external pfb file
    pub(super) in_pfb: bool,
}

/// Operator methods
impl<'a> PostscriptInterpreter<'a> {
    pub fn new(mut buffer: &'a [u8]) -> Self {
        // skip .pfb section header
        let in_pfb = if buffer.first() == Some(&0x80) {
            assert_eq!(buffer[1], 0x01);
            buffer = &buffer[6..];
            true
        } else {
            false
        };

        let mut interpreter = Self {
            lexer: PostScriptLexer::new(Cow::Borrowed(buffer)),
            operand_stack: Vec::new(),
            dictionary_stack: Vec::new(),
            execution_stack: Vec::new(),
            arrays: Container::new(),
            dictionaries: Container::new(),
            fonts: HashMap::new(),
            resources: PostScriptDictionary::new(),
            in_pfb,
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

        interpreter.get_dict_mut(user_dict).insert(
            PostScriptString::from_bytes(b"userdict".to_vec()),
            PostScriptObject::Dictionary(user_dict),
        );

        let errordict = interpreter.new_dict(PostScriptDictionary::new());
        interpreter.get_dict_mut(system_dict).insert(
            PostScriptString::from_bytes(b"errordict".to_vec()),
            PostScriptObject::Dictionary(errordict),
        );

        let font_directory = interpreter.new_dict(PostScriptDictionary::new());
        interpreter.get_dict_mut(system_dict).insert(
            PostScriptString::from_bytes(b"FontDirectory".to_vec()),
            PostScriptObject::Dictionary(font_directory),
        );

        let findfont = interpreter.new_array(PostScriptArray::new_procedure(vec![
            PostScriptObject::Name(PostScriptString::from_bytes(b"Font".to_vec())),
            PostScriptObject::Operator(PostscriptOperator::FindResource),
        ]));
        interpreter.get_dict_mut(system_dict).insert(
            PostScriptString::from_bytes(b"findfont".to_vec()),
            PostScriptObject::Array(findfont),
        );

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
                    PostScriptObject::Array(proc) => {
                        let proc = self.get_arr(proc).clone();
                        // todo: probably need a better check to determine whether
                        // it's a procedure?
                        if proc.access() == Access::ExecuteOnly {
                            self.execute_procedure(proc.into_inner())?;
                        } else {
                            self.push(obj);
                        }
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
            PostscriptOperator::Gt => self.cmp(|a, b| a > b, |a, b| a > b),
            PostscriptOperator::Ge => self.cmp(|a, b| a >= b, |a, b| a >= b),
            PostscriptOperator::Lt => self.cmp(|a, b| a < b, |a, b| a < b),
            PostscriptOperator::Le => self.cmp(|a, b| a <= b, |a, b| a <= b),
            PostscriptOperator::Ceiling => self.float_op(f32::ceil),
            PostscriptOperator::Floor => self.float_op(f32::floor),
            PostscriptOperator::Round => self.float_op(f32::round),
            PostscriptOperator::MaxLength => self.max_length(),
            PostscriptOperator::Length => self.length(),
            PostscriptOperator::Cvx => self.cvx(),
            PostscriptOperator::Copy => self.copy(),
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
            PostscriptOperator::If => self.if_op(),
            PostscriptOperator::IfElse => self.if_else(),
            PostscriptOperator::Index => self.index(),
            PostscriptOperator::DefineFont => self.define_font(),
            PostscriptOperator::Mark => self.mark(),
            PostscriptOperator::CloseFile => self.close_file(),
            PostscriptOperator::For => self.for_loop(),
            PostscriptOperator::Add => self.arith(i32::checked_add, f32::add),
            PostscriptOperator::Sub => self.arith(i32::checked_sub, f32::sub),
            PostscriptOperator::Mul => self.arith(i32::checked_mul, f32::mul),
            PostscriptOperator::Div => self.arith(i32::checked_div, f32::div),
            PostscriptOperator::Idiv => self.idiv(),
            PostscriptOperator::Count => self.count(),
            PostscriptOperator::Eq => self.eq(),
            PostscriptOperator::Ne => self.ne(),
            PostscriptOperator::Type => self.object_type(),
            PostscriptOperator::Bind => self.bind(),
            PostscriptOperator::And => self.and(),
            PostscriptOperator::Or => self.or(),
            PostscriptOperator::InternalDict => self.internal_dict(),
            op @ (PostscriptOperator::DefineResource
            | PostscriptOperator::UndefineResource
            | PostscriptOperator::FindResource
            | PostscriptOperator::FindColorRendering
            | PostscriptOperator::ResourceStatus
            | PostscriptOperator::ResourceForAll) => {
                todo!("unimplemented resource operator {:?}", op)
            }
            op @ PostscriptOperator::Abs => todo!("{:?}", op),
            PostscriptOperator::Save | PostscriptOperator::Restore => todo!(),
        }
    }

    pub(super) fn execute_procedure(&mut self, proc: Vec<PostScriptObject>) -> PdfResult<()> {
        for tok in proc {
            self.execute_token(tok)?;
        }

        Ok(())
    }

    fn cmp(
        &mut self,
        cmp_int: impl Fn(i32, i32) -> bool,
        cmp_str: impl Fn(&PostScriptString, &PostScriptString) -> bool,
    ) -> PdfResult<()> {
        match self.pop()? {
            PostScriptObject::Int(i2) => {
                let i1 = self.pop_int()?;

                self.push(PostScriptObject::Bool(cmp_int(i1, i2)));
            }
            PostScriptObject::String(s2) => {
                let s1 = self.pop_string()?;

                self.push(PostScriptObject::Bool(cmp_str(
                    self.get_str(s1),
                    self.get_str(s2),
                )));
            }
            obj => anyhow::bail!("expected int or string, found {:?}", obj),
        }

        Ok(())
    }

    fn arith(
        &mut self,
        checked: impl Fn(i32, i32) -> Option<i32>,
        real: impl Fn(f32, f32) -> f32,
    ) -> PdfResult<()> {
        let n2 = self.pop()?;
        let n1 = self.pop()?;

        if n1.is_int() && n2.is_int() {
            let n1 = n1.into_int()?;
            let n2 = n2.into_int()?;

            match checked(n1, n2) {
                Some(result) => self.push(PostScriptObject::Int(result)),
                None => self.push(PostScriptObject::Float(real(n1 as f32, n2 as f32))),
            }

            return Ok(());
        }

        let n1 = n1.into_float()?;
        let n2 = n2.into_float()?;

        self.push(PostScriptObject::Float(real(n1, n2)));

        Ok(())
    }

    fn idiv(&mut self) -> PdfResult<()> {
        let n2 = self.pop_number()?;
        let n1 = self.pop_number()?;

        self.push(PostScriptObject::Int((n1 / n2) as i32));

        Ok(())
    }

    fn for_loop(&mut self) -> PdfResult<()> {
        let proc = self.pop_procedure()?;
        let limit = self.pop_number()?;
        let increment = self.pop_number()?;
        let initial = self.pop_number()?;

        let mut control = initial;

        let should_terminate = |control: f32| {
            if increment.is_sign_positive() {
                control > limit
            } else {
                control < limit
            }
        };

        while !should_terminate(control) {
            self.push(PostScriptObject::Float(control));

            let proc = self.get_arr(proc).clone().into_inner();
            self.execute_procedure(proc)?;

            control += increment;
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

    fn if_op(&mut self) -> PdfResult<()> {
        let proc = self.pop_procedure()?;
        let b = self.pop_bool()?;

        if b {
            let proc = self.get_arr(proc).clone().into_inner();
            self.execute_procedure(proc)?;
        }

        Ok(())
    }

    fn if_else(&mut self) -> PdfResult<()> {
        let proc_two = self.pop_procedure()?;
        let proc_one = self.pop_procedure()?;
        let b = self.pop_bool()?;

        let proc = if b {
            self.get_arr(proc_one).clone().into_inner()
        } else {
            self.get_arr(proc_two).clone().into_inner()
        };

        self.execute_procedure(proc)?;

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
                    _ => anyhow::bail!(PostScriptError::TypeCheck),
                })?;

                let val = self.get_arr_mut(arr).get(idx)?.clone();
                self.push(val);
            }
            PostScriptObject::Dictionary(dict) => {
                let key = match key_or_idx {
                    PostScriptObject::Name(name) => name,
                    _ => anyhow::bail!(PostScriptError::TypeCheck),
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
                    _ => anyhow::bail!(PostScriptError::TypeCheck),
                })?;

                let val = self.get_str_mut(s).get(idx)?;
                self.push(PostScriptObject::Int(i32::from(val)));
            }
            _ => anyhow::bail!(PostScriptError::TypeCheck),
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
            obj => anyhow::bail!("expected bool or int in `not`, found {:?}", obj),
        }

        Ok(())
    }

    fn internal_dict(&mut self) -> PdfResult<()> {
        let n = self.pop_int()?;
        assert_eq!(n, 1183615869);

        let dict = self.new_dict(PostScriptDictionary::new());

        // todo: don't recreate on each execution
        self.push(PostScriptObject::Dictionary(dict));

        Ok(())
    }

    fn or(&mut self) -> PdfResult<()> {
        match self.pop()? {
            PostScriptObject::Bool(b1) => {
                let b2 = self.pop_bool()?;
                self.push(PostScriptObject::Bool(b1 || b2));
            }
            PostScriptObject::Int(i1) => {
                let i2 = self.pop_int()?;
                self.push(PostScriptObject::Int(i1 | i2));
            }
            obj => anyhow::bail!("expected int or bool in `or`, found {:?}", obj),
        }

        Ok(())
    }

    fn and(&mut self) -> PdfResult<()> {
        match self.pop()? {
            PostScriptObject::Bool(b1) => {
                let b2 = self.pop_bool()?;
                self.push(PostScriptObject::Bool(b1 && b2));
            }
            PostScriptObject::Int(i1) => {
                let i2 = self.pop_int()?;
                self.push(PostScriptObject::Int(i1 & i2));
            }
            obj => anyhow::bail!("expected int or bool in `and`, found {:?}", obj),
        }

        Ok(())
    }

    fn known(&mut self) -> PdfResult<()> {
        let key = self.pop_name()?;
        let dict = self.pop_dict()?;

        self.push(PostScriptObject::Bool(self.get_dict(dict).contains(&key)));

        Ok(())
    }

    fn count(&mut self) -> PdfResult<()> {
        let count = self.operand_stack.len();

        self.push(PostScriptObject::Int(count as i32));

        Ok(())
    }

    fn objects_equal(&self, a: PostScriptObject, b: PostScriptObject) -> bool {
        match (a, b) {
            (PostScriptObject::Array(arr1), PostScriptObject::Array(arr2)) => arr1 == arr2,
            (PostScriptObject::Dictionary(dict1), PostScriptObject::Dictionary(dict2)) => {
                dict1 == dict2
            }
            (PostScriptObject::Bool(b1), PostScriptObject::Bool(b2)) => b1 == b2,
            (PostScriptObject::Null, PostScriptObject::Null) => true,
            (PostScriptObject::String(string), PostScriptObject::Name(name))
            | (PostScriptObject::Name(name), PostScriptObject::String(string)) => {
                let string = self.get_str(string);
                &name == string
            }
            (PostScriptObject::String(string1), PostScriptObject::String(string2)) => {
                let string1 = self.get_str(string1);
                let string2 = self.get_str(string2);
                string1 == string2
            }
            (PostScriptObject::Name(name1), PostScriptObject::Name(name2)) => name1 == name2,
            (PostScriptObject::Int(int1), PostScriptObject::Int(int2)) => int1 == int2,
            (PostScriptObject::Int(int), PostScriptObject::Float(float))
            | (PostScriptObject::Float(float), PostScriptObject::Int(int)) => float == int as f32,
            (PostScriptObject::Float(float1), PostScriptObject::Float(float2)) => float1 == float2,
            (PostScriptObject::Literal(literal1), PostScriptObject::Literal(literal2)) => {
                literal1 == literal2
            }
            (PostScriptObject::Mark, PostScriptObject::Mark) => todo!(),
            (PostScriptObject::File, PostScriptObject::File) => todo!(),
            (PostScriptObject::Operator(..), PostScriptObject::Operator(..)) => todo!(),
            _ => false,
        }
    }

    fn eq(&mut self) -> PdfResult<()> {
        let a = self.pop()?;
        let b = self.pop()?;

        let equals = self.objects_equal(a, b);

        self.push(PostScriptObject::Bool(equals));

        Ok(())
    }

    fn ne(&mut self) -> PdfResult<()> {
        let a = self.pop()?;
        let b = self.pop()?;

        let equals = !self.objects_equal(a, b);

        self.push(PostScriptObject::Bool(equals));

        Ok(())
    }

    fn object_type(&mut self) -> PdfResult<()> {
        let obj = self.pop()?;

        let ty: &[u8] = match obj {
            PostScriptObject::Null => b"nulltype",
            PostScriptObject::Int(_) => b"integertype",
            PostScriptObject::Float(_) => b"realtype",
            PostScriptObject::Name(_) => b"nametype",
            PostScriptObject::Bool(_) => b"booleantype",
            PostScriptObject::String(_) => b"stringtype",
            PostScriptObject::Array(_) => b"arraytype",
            PostScriptObject::Mark => b"marktype",
            PostScriptObject::File => b"filetype",
            PostScriptObject::Dictionary(_) => b"dicttype",
            PostScriptObject::Literal(_) | PostScriptObject::Operator(_) => b"operatortype",
            // todo: packedarraytype, fonttype, gstatetype, savetype
        };

        let name = PostScriptString::from_bytes(ty.to_vec());

        self.push(PostScriptObject::Name(name));

        Ok(())
    }

    fn bind(&mut self) -> PdfResult<()> {
        let proc_idx = self.pop_arr()?;
        let proc = self.get_arr(proc_idx);

        println!("`bind` operator not fully implemented");

        for obj in proc.as_inner() {
            if let PostScriptObject::Name(..) = obj {
                // todo
            }
        }

        self.push(PostScriptObject::Array(proc_idx));

        Ok(())
    }

    fn put(&mut self) -> PdfResult<()> {
        let value = self.pop()?;
        let key_or_idx = self.pop()?;
        let composite_obj = self.pop()?;

        match composite_obj {
            PostScriptObject::String(s) => {
                let idx = usize::try_from(key_or_idx.into_int()?)?;

                let ch = u8::try_from(match value {
                    PostScriptObject::Int(i) => i,
                    PostScriptObject::Float(f) => f.round() as i32,
                    _ => anyhow::bail!(PostScriptError::TypeCheck),
                })?;

                self.get_str_mut(s).put(idx, ch);
            }
            PostScriptObject::Dictionary(dict) => {
                let key = match key_or_idx {
                    PostScriptObject::Name(name) => name,
                    _ => anyhow::bail!(PostScriptError::TypeCheck),
                };

                self.get_dict_mut(dict).insert(key, value);
            }
            PostScriptObject::Array(arr) => {
                let idx = usize::try_from(key_or_idx.into_int()?)?;

                self.get_arr_mut(arr).put(idx, value);
            }
            _ => anyhow::bail!(PostScriptError::TypeCheck),
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
        self.lexer.skip_whitespace();
        let mut buffer = &self.lexer.buffer[self.lexer.cursor..];

        // skip .pfb section header
        if buffer[0] == 0x80 && self.in_pfb {
            assert_eq!(buffer[0], 0x80);
            assert_eq!(buffer[1], 0x02);
            let len = u32::from_le_bytes([buffer[2], buffer[3], buffer[4], buffer[5]]);
            buffer = &buffer[6..len as usize + 6];
        }

        // println!("{}", String::from_utf8_lossy(&decode::decrypt(buffer)));
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
                    let proc = self.lex_procedure()?;
                    let arr_idx = self.arrays.insert(PostScriptArray::new_procedure(proc));
                    objs.push(PostScriptObject::Array(arr_idx));
                }
                PostScriptObject::Operator(PostscriptOperator::ProcedureEnd) => break,
                obj => objs.push(obj),
            }
        }

        Ok(objs)
    }

    fn procedure_start(&mut self) -> PdfResult<()> {
        let proc = self.lex_procedure()?;

        self.push_procedure(proc);

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
                self.get_dict_mut(dict).set_access(access);

                obj = PostScriptObject::Dictionary(dict);
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

        self.get_dict_mut(dict).insert(key, value);

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
            _ => anyhow::bail!(PostScriptError::TypeCheck),
        };

        self.push_dict_stack(dict);

        Ok(())
    }

    fn cvx(&mut self) -> PdfResult<()> {
        let obj = self.pop()?;

        match obj {
            PostScriptObject::Name(name) | PostScriptObject::Literal(name) => {
                self.push(ident_token_from_bytes(name.as_bytes())?);
            }
            PostScriptObject::Array(arr_idx) => {
                let arr = self.get_arr_mut(arr_idx);
                arr.set_access(Access::ExecuteOnly);
                self.push(PostScriptObject::Array(arr_idx));
            }
            PostScriptObject::Null => todo!(),
            PostScriptObject::Int(_) => todo!(),
            PostScriptObject::Float(_) => todo!(),
            PostScriptObject::Bool(_) => todo!(),
            PostScriptObject::String(_) => todo!(),
            PostScriptObject::Mark => todo!(),
            PostScriptObject::File => todo!(),
            PostScriptObject::Dictionary(_) => todo!(),
            PostScriptObject::Operator(_) => todo!(),
        }

        Ok(())
    }

    fn float_op(&mut self, func: impl Fn(f32) -> f32) -> PdfResult<()> {
        let n = self.pop()?;

        if n.is_int() {
            self.push(n);
            return Ok(());
        }

        let n = func(n.into_float()?);

        self.push(PostScriptObject::Float(n));

        Ok(())
    }

    fn max_length(&mut self) -> PdfResult<()> {
        let dict_idx = self.pop_dict()?;
        let dict = self.get_dict(dict_idx);
        let capacity = dict.capacity();

        self.push(PostScriptObject::Int(capacity as i32));

        Ok(())
    }

    fn length(&mut self) -> PdfResult<()> {
        let len = match self.pop()? {
            PostScriptObject::Name(s) => s.len(),
            PostScriptObject::String(s) => self.get_str(s).len(),
            PostScriptObject::Array(a) => self.get_arr(a).len(),
            PostScriptObject::Dictionary(d) => self.get_dict(d).len(),
            obj => anyhow::bail!("expected name, string, array, or dict; found {:?}", obj),
        };

        self.push(PostScriptObject::Int(len as i32));

        Ok(())
    }

    fn copy(&mut self) -> PdfResult<()> {
        match self.pop()? {
            PostScriptObject::Int(i) => {
                let mut to_dup = Vec::new();

                for _ in 0..i {
                    let obj = self.pop()?;
                    to_dup.push(obj);
                }

                to_dup.reverse();

                for obj in to_dup.clone() {
                    self.push(obj);
                }
                for obj in to_dup {
                    self.push(obj);
                }
            }
            PostScriptObject::Array(..)
            | PostScriptObject::String(..)
            | PostScriptObject::Dictionary(..) => {
                todo!("postscript copy not implemented for composite objects")
            }
            n => anyhow::bail!("expected integer or composite object, got {:?}", n),
        }

        Ok(())
    }

    fn dict(&mut self) -> PdfResult<()> {
        let n = match self.pop()? {
            PostScriptObject::Int(i) => usize::try_from(i),
            obj => anyhow::bail!("expected dict size, found {:?}", obj),
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

/// Utils
impl<'a> PostscriptInterpreter<'a> {
    pub(super) fn push(&mut self, obj: PostScriptObject) {
        self.operand_stack.push(obj);
    }

    fn push_arr(&mut self, arr: Vec<PostScriptObject>) {
        let idx = self.arrays.insert(PostScriptArray::from_objects(arr));
        self.operand_stack.push(PostScriptObject::Array(idx));
    }

    fn push_procedure(&mut self, arr: Vec<PostScriptObject>) {
        let idx = self.arrays.insert(PostScriptArray::new_procedure(arr));
        self.operand_stack.push(PostScriptObject::Array(idx));
    }

    pub(super) fn get_arr(&mut self, k: ArrayIndex) -> &PostScriptArray {
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

    pub(super) fn intern_string(&mut self, str: PostScriptString) -> PostScriptObject {
        assert!(!str.is_empty(), "implies a bug. TODO: remove");

        let idx = self.lexer.strings.insert(str);
        PostScriptObject::String(idx)
    }

    pub(super) fn get_str(&self, k: StringIndex) -> &PostScriptString {
        self.lexer.strings.get(&k).unwrap()
    }

    fn get_str_mut(&mut self, k: StringIndex) -> &mut PostScriptString {
        self.lexer.strings.get_mut(&k).unwrap()
    }

    pub(super) fn pop(&mut self) -> PostScriptResult<PostScriptObject> {
        self.operand_stack
            .pop()
            .ok_or(anyhow::anyhow!(PostScriptError::StackUnderflow))
    }

    fn pop_int(&mut self) -> PdfResult<i32> {
        match self.pop()? {
            PostScriptObject::Int(i) => Ok(i),
            PostScriptObject::Float(f) => Ok(f.round() as i32),
            obj => anyhow::bail!("expected int or float, found {:?}", obj),
        }
    }

    fn pop_name(&mut self) -> PdfResult<PostScriptString> {
        match self.pop()? {
            PostScriptObject::Name(name) => Ok(name),
            obj => anyhow::bail!("expected name, found {:?}", obj),
        }
    }

    fn pop_string(&mut self) -> PdfResult<StringIndex> {
        match self.pop()? {
            PostScriptObject::String(s) => Ok(s),
            obj => anyhow::bail!("expected string, found {:?}", obj),
        }
    }

    fn pop_dict(&mut self) -> PdfResult<DictionaryIndex> {
        match self.pop()? {
            PostScriptObject::Dictionary(dict) => Ok(dict),
            obj => anyhow::bail!("expected dict, found {:?}", obj),
        }
    }

    fn pop_arr(&mut self) -> PdfResult<ArrayIndex> {
        match self.pop()? {
            PostScriptObject::Array(arr) => Ok(arr),
            obj => anyhow::bail!("expected array, found {:?}", obj),
        }
    }

    fn pop_bool(&mut self) -> PdfResult<bool> {
        match self.pop()? {
            PostScriptObject::Bool(b) => Ok(b),
            obj => anyhow::bail!("expected bool, found {:?}", obj),
        }
    }

    fn pop_number(&mut self) -> PdfResult<f32> {
        match self.pop()? {
            PostScriptObject::Int(n) => Ok(n as f32),
            PostScriptObject::Float(n) => Ok(n),
            obj => anyhow::bail!("expected number, found {:?}", obj),
        }
    }

    fn pop_procedure(&mut self) -> PdfResult<ArrayIndex> {
        self.pop_arr()
    }

    fn pop_file(&mut self) -> PdfResult<PostScriptObject> {
        match self.pop()? {
            obj @ PostScriptObject::File => Ok(obj),
            obj => anyhow::bail!("expected file, found {:?}", obj),
        }
    }

    fn new_dict(&mut self, dict: PostScriptDictionary) -> DictionaryIndex {
        self.dictionaries.insert(dict)
    }

    fn new_array(&mut self, arr: PostScriptArray) -> ArrayIndex {
        self.arrays.insert(arr)
    }

    fn push_dict_stack(&mut self, dict: DictionaryIndex) {
        self.dictionary_stack.push(dict);
    }

    fn pop_dict_stack(&mut self) -> PostScriptResult<DictionaryIndex> {
        self.dictionary_stack
            .pop()
            .ok_or(anyhow::anyhow!(PostScriptError::DictStackUnderflow))
    }

    fn get_current_dict(&mut self) -> PostScriptResult<DictionaryIndex> {
        self.dictionary_stack
            .last()
            .cloned()
            .ok_or(anyhow::anyhow!(PostScriptError::DictStackUnderflow))
    }

    pub(super) fn get_dict(&self, key: DictionaryIndex) -> &PostScriptDictionary {
        self.dictionaries.get(&key).unwrap()
    }

    fn get_dict_mut(&mut self, key: DictionaryIndex) -> &mut PostScriptDictionary {
        self.dictionaries.get_mut(&key).unwrap()
    }

    fn get_key(&mut self, key: &PostScriptString) -> PdfResult<PostScriptObject> {
        for &dict in self.dictionary_stack.iter().rev() {
            if let Some(obj) = self.get_dict(dict).get(key) {
                return Ok(obj.clone());
            }
        }

        anyhow::bail!(PostScriptError::Undefined { key: key.clone() })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /// Assert the next operand on the stack is a string with the given contents
    macro_rules! assert_string {
        ($interpreter:ident, $str:literal) => {
            let s = $interpreter.pop_string().unwrap();
            let resolved = $interpreter.get_str(s);
            assert_eq!(resolved, &PostScriptString::from_bytes($str.to_vec()));
        };
    }

    #[test]
    fn add_two_integers() {
        let mut interpreter = PostscriptInterpreter::new(b"1 2 add");

        interpreter.run().unwrap();

        assert_eq!(interpreter.pop().unwrap(), PostScriptObject::Int(3));
        assert!(interpreter.pop().is_err());
    }

    #[test]
    fn add_two_floats() {
        let mut interpreter = PostscriptInterpreter::new(b"1.0 2.0 add");

        interpreter.run().unwrap();

        assert_eq!(interpreter.pop().unwrap(), PostScriptObject::Float(3.0));
        assert!(interpreter.pop().is_err());
    }

    #[test]
    fn add_int_and_float() {
        let mut interpreter = PostscriptInterpreter::new(b"1 2.0 add");

        interpreter.run().unwrap();

        assert_eq!(interpreter.pop().unwrap(), PostScriptObject::Float(3.0));
        assert!(interpreter.pop().is_err());
    }

    #[test]
    fn add_float_and_int() {
        let mut interpreter = PostscriptInterpreter::new(b"1.0 2 add");

        interpreter.run().unwrap();

        assert_eq!(interpreter.pop().unwrap(), PostScriptObject::Float(3.0));
        assert!(interpreter.pop().is_err());
    }

    #[test]
    fn known_name_exists() {
        let mut interpreter = PostscriptInterpreter::new(
            b"
            /mydict 5 dict def
            mydict /total 0 put
            mydict /total known
        ",
        );

        interpreter.run().unwrap();

        assert_eq!(interpreter.pop().unwrap(), PostScriptObject::Bool(true));
        assert!(interpreter.pop().is_err());
    }

    #[test]
    fn push_number() {
        let mut interpreter = PostscriptInterpreter::new(b"5");

        interpreter.run().unwrap();

        assert_eq!(interpreter.pop().unwrap(), PostScriptObject::Int(5));
        assert!(interpreter.pop().is_err());
    }

    #[test]
    fn push_name() {
        let mut interpreter = PostscriptInterpreter::new(b"/name");

        interpreter.run().unwrap();

        let name = interpreter.pop_name().unwrap();

        assert_eq!(name.as_bytes(), b"name");
        assert!(interpreter.pop().is_err());
    }

    #[test]
    fn known_name_dne() {
        let mut interpreter = PostscriptInterpreter::new(
            b"
                /mydict 5 dict def
                mydict /total 0 put
                mydict /badname known
            ",
        );

        interpreter.run().unwrap();

        assert_eq!(interpreter.pop().unwrap(), PostScriptObject::Bool(false));
        assert!(interpreter.pop().is_err());
    }

    #[test]
    fn for_loop_basic_add() {
        let mut interpreter = PostscriptInterpreter::new(b"0 1 1 4 {add} for");

        interpreter.run().unwrap();

        assert_eq!(interpreter.pop().unwrap(), PostScriptObject::Float(10.0));
        assert!(interpreter.pop().is_err());
    }

    #[test]
    fn dict_contains_standard_encoding() {
        let mut interpreter = PostscriptInterpreter::new(
            b"
            3 dict begin
            /FontName /FZJRZA+SFSS2488 def
            /Encoding StandardEncoding def
            /PaintType 0 def
            currentdict end
        ",
        );

        interpreter.run().unwrap();
    }

    #[test]
    fn for_loop_empty_proc() {
        let mut interpreter = PostscriptInterpreter::new(b"1 2 6 { } for");

        interpreter.run().unwrap();

        assert_eq!(interpreter.pop().unwrap(), PostScriptObject::Float(5.0));
        assert_eq!(interpreter.pop().unwrap(), PostScriptObject::Float(3.0));
        assert_eq!(interpreter.pop().unwrap(), PostScriptObject::Float(1.0));
        assert!(interpreter.pop().is_err());
    }

    #[test]
    fn for_loop_negative_and_decimal_incremental() {
        let mut interpreter = PostscriptInterpreter::new(b"3 -.5 1 { } for");

        interpreter.run().unwrap();

        assert_eq!(interpreter.pop().unwrap(), PostScriptObject::Float(1.0));
        assert_eq!(interpreter.pop().unwrap(), PostScriptObject::Float(1.5));
        assert_eq!(interpreter.pop().unwrap(), PostScriptObject::Float(2.0));
        assert_eq!(interpreter.pop().unwrap(), PostScriptObject::Float(2.5));
        assert_eq!(interpreter.pop().unwrap(), PostScriptObject::Float(3.0));
        assert!(interpreter.pop().is_err());
    }

    #[test]
    fn operator_inside_array_is_executed() {
        let mut interpreter = PostscriptInterpreter::new(b"[1 2 add]");

        interpreter.run().unwrap();

        assert_eq!(interpreter.operand_stack.len(), 1);

        let arr = interpreter.pop_arr().unwrap();
        let arr = interpreter.get_arr(arr);

        assert_eq!(arr.as_inner(), &[PostScriptObject::Int(3)]);
    }

    #[test]
    #[ignore = "copy not yet implemented for composite objects"]
    fn copy_composite() {
        let mut interpreter = PostscriptInterpreter::new(
            b"
            /a1 [1 2 3] def
            a1 dup length array copy
        ",
        );

        interpreter.run().unwrap();

        assert_eq!(interpreter.operand_stack.len(), 1);

        let arr = interpreter.pop_arr().unwrap();
        let arr = interpreter.get_arr(arr);

        assert_eq!(
            arr.as_inner(),
            &[
                PostScriptObject::Int(1),
                PostScriptObject::Int(2),
                PostScriptObject::Int(3)
            ]
        );
    }

    #[test]
    fn copy_non_composite_len_2() {
        let mut interpreter = PostscriptInterpreter::new(
            b"
            (a) (b) (c) 2 copy
        ",
        );

        interpreter.run().unwrap();

        assert_string!(interpreter, b"c");
        assert_string!(interpreter, b"b");
        assert_string!(interpreter, b"c");
        assert_string!(interpreter, b"b");
        assert_string!(interpreter, b"a");
        assert!(interpreter.pop().is_err());
    }

    #[test]
    fn copy_non_composite_len_0() {
        let mut interpreter = PostscriptInterpreter::new(
            b"
            (a) (b) (c) 0 copy
        ",
        );

        interpreter.run().unwrap();

        assert_string!(interpreter, b"c");
        assert_string!(interpreter, b"b");
        assert_string!(interpreter, b"a");
        assert!(interpreter.pop().is_err());
    }

    #[test]
    fn getting_internal_dict_doesnt_crash() {
        let mut interpreter = PostscriptInterpreter::new(
            b"
            /Private 17 dict dup begin
            /ND{noaccess def}executeonly def
            systemdict /internaldict known
            {
                1183615869 systemdict /internaldict get exec
                /StemSnapLength 2 copy known { get 8 lt } { pop pop true } ifelse
            }
            { true } ifelse { pop [49 57] } if ND
        ",
        );

        interpreter.run().unwrap();
    }

    #[test]
    #[should_panic]

    fn unknown_operator() {
        let mut interpreter = PostscriptInterpreter::new(
            b"
            aaaa
        ",
        );

        interpreter.run().unwrap();
    }
}
