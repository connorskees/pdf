/// Generic trait for parsing binary formats
pub trait BinaryParser {
    fn buffer(&self) -> &[u8];
    fn cursor(&self) -> usize;
    fn cursor_mut(&mut self) -> &mut usize;

    fn next(&mut self) -> anyhow::Result<u8> {
        let b = self.buffer().get(self.cursor()).copied();
        if b.is_some() {
            *self.cursor_mut() += 1;
        }
        b.ok_or(anyhow::anyhow!("unexpected eof"))
    }

    fn peek(&mut self) -> Option<u8> {
        self.buffer().get(self.cursor()).copied()
    }

    fn parse_u16(&mut self) -> anyhow::Result<u16> {
        let b1 = self.next()?;
        let b2 = self.next()?;

        Ok(u16::from_be_bytes([b1, b2]))
    }

    fn parse_i16(&mut self) -> anyhow::Result<i16> {
        let b1 = self.next()?;
        let b2 = self.next()?;

        Ok(i16::from_be_bytes([b1, b2]))
    }

    fn parse_u32_bytes(&mut self) -> anyhow::Result<[u8; 4]> {
        Ok(self.parse_u32()?.to_be_bytes())
    }

    fn parse_u32(&mut self) -> anyhow::Result<u32> {
        let b1 = self.next()?;
        let b2 = self.next()?;
        let b3 = self.next()?;
        let b4 = self.next()?;

        Ok(u32::from_be_bytes([b1, b2, b3, b4]))
    }

    fn parse_i32(&mut self) -> anyhow::Result<i32> {
        let b1 = self.next()?;
        let b2 = self.next()?;
        let b3 = self.next()?;
        let b4 = self.next()?;

        Ok(i32::from_be_bytes([b1, b2, b3, b4]))
    }

    fn parse_u64(&mut self) -> anyhow::Result<u64> {
        let b1 = self.next()?;
        let b2 = self.next()?;
        let b3 = self.next()?;
        let b4 = self.next()?;
        let b5 = self.next()?;
        let b6 = self.next()?;
        let b7 = self.next()?;
        let b8 = self.next()?;

        Ok(u64::from_be_bytes([b1, b2, b3, b4, b5, b6, b7, b8]))
    }

    fn parse_i64(&mut self) -> anyhow::Result<i64> {
        let b1 = self.next()?;
        let b2 = self.next()?;
        let b3 = self.next()?;
        let b4 = self.next()?;
        let b5 = self.next()?;
        let b6 = self.next()?;
        let b7 = self.next()?;
        let b8 = self.next()?;

        Ok(i64::from_be_bytes([b1, b2, b3, b4, b5, b6, b7, b8]))
    }
}
