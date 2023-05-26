use crate::font::Glyph;

use super::{
    graphics_state::TrueTypeGraphicsState, instruction::TrueTypeInstruction, TrueTypeFontFile,
};

struct InstructionStream {
    buffer: Vec<u8>,
    cursor: usize,
}

impl InstructionStream {
    pub fn new(buffer: Vec<u8>) -> Self {
        Self { buffer, cursor: 0 }
    }

    fn next_instruction(&mut self) -> Option<TrueTypeInstruction> {
        let b = self.buffer.get(self.cursor)?;
        self.cursor += 1;

        match b {
            b => todo!("{:?}", b),
        }
    }
}

pub struct TrueTypeInterpreter<'a, 'b> {
    instruction_stream: InstructionStream,
    interpreter_stack: Vec<u32>,
    graphics_state: TrueTypeGraphicsState,
    ttf_file: &'b mut TrueTypeFontFile<'a>,
    storage_area: Vec<u32>,
}

impl<'a, 'b> TrueTypeInterpreter<'a, 'b> {
    pub fn new(ttf_file: &'b mut TrueTypeFontFile<'a>) -> Self {
        let storage_area_size = ttf_file.max_storage();
        Self {
            instruction_stream: InstructionStream::new(Vec::new()),
            interpreter_stack: Vec::new(),
            graphics_state: TrueTypeGraphicsState::default(),
            ttf_file,
            storage_area: vec![0; storage_area_size as usize],
        }
    }

    pub fn render_glyph(&mut self, char_code: u32) -> Option<Glyph> {
        let _instructions = self.ttf_file.glyph(char_code).unwrap();
        todo!()
    }

    pub fn reset(&mut self) {
        self.interpreter_stack.clear();
        self.graphics_state = TrueTypeGraphicsState::default();
        self.storage_area.fill(0);
    }
}
