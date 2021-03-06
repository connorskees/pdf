pub(crate) fn decode_ascii_hex(stream: &[u8]) -> Vec<u8> {
    let mut buffer = Vec::with_capacity(stream.len() / 2);

    let mut iter = stream.iter().filter(|&&b| !b.is_ascii_whitespace());

    loop {
        let mut n = match iter.next() {
            Some(&c @ b'0'..=b'9') => c - b'0',
            Some(&c @ b'A'..=b'F') => c - b'A' + 10,
            Some(&c @ b'a'..=b'f') => c - b'a' + 10,
            Some(b'>') | None => break,
            Some(..) => todo!(),
        } as u16;

        n *= 16;

        n += match iter.next() {
            Some(&c @ b'0'..=b'9') => c - b'0',
            Some(&c @ b'A'..=b'F') => c - b'A' + 10,
            Some(&c @ b'a'..=b'f') => c - b'a' + 10,
            Some(b'>') | None => break,
            Some(..) => todo!(),
        } as u16;

        buffer.extend_from_slice(&n.to_be_bytes());
    }

    buffer
}

fn decode_ascii_85_digit(digit: u8, n: &mut u32, count: &mut u8, result: &mut Vec<u8>) {
    *n *= 85;

    if digit == b'z' {
        if *count == 0 {
            result.extend_from_slice(&[0, 0, 0, 0]);
        } else {
            todo!()
        }
    } else {
        *n += (digit - b'!') as u32;
    }

    *count += 1;
}

pub(crate) fn decode_ascii_85(mut stream: &[u8]) -> Vec<u8> {
    if stream.starts_with(b"<~") {
        stream = &stream[2..];
    }

    let mut buffer = Vec::with_capacity((stream.len() / 5) * 4);

    let mut iter = stream.iter().filter(|&&b| !b.is_ascii_whitespace());

    let mut n: u32 = 0;
    let mut count = 0;

    while let Some(&digit) = iter.next() {
        if digit == b'~' {
            if iter.next() != Some(&b'>') {
                todo!()
            }

            break;
        }

        decode_ascii_85_digit(digit, &mut n, &mut count, &mut buffer);

        if count == 5 {
            buffer.extend_from_slice(&n.to_be_bytes());
            count = 0;
            n = 0;
        }
    }

    if count != 0 {
        let to_remove = 5 - count as usize;
        while count != 5 {
            decode_ascii_85_digit(b'u', &mut n, &mut count, &mut buffer);
        }

        buffer.extend_from_slice(&n.to_be_bytes());

        buffer.drain((buffer.len() - to_remove)..);
    }

    buffer
}

#[cfg(test)]
mod test {
    use super::decode_ascii_85;

    #[test]
    fn ascii_85() {
        assert_eq!(
            decode_ascii_85(b"<~9jqo^F*2M7/c~>"),
            [77, 97, 110, 32, 115, 117, 114, 101, 46],
        );

        assert_eq!(
            String::from_utf8(decode_ascii_85(
                br#"9jqo^BlbD-BleB1DJ+*+F(f,q/0JhKF<GL>Cj@.4Gp$d7F!,L7@<6@)/0JDEF<G%<+EV:2F!,
            O<DJ+*.@<*K0@<6L(Df-\0Ec5e;DffZ(EZee.Bl.9pF"AGXBPCsi+DGm>@3BB/F*&OCAfu2/AKY
            i(DIb:@FD,*)+C]U=@3BN#EcYf8ATD3s@q?d$AftVqCh[NqF<G:8+EV:.+Cf>-FD5W8ARlolDIa
            l(DId<j@<?3r@:F%a+D58'ATD4$Bl@l3De:,-DJs`8ARoFb/0JMK@qB4^F!,R<AKZ&-DfTqBG%G
            >uD.RTpAKYo'+CT/5+Cei#DII?(E,9)oF*2M7/c~>"#
            ))
            .unwrap(),
            r#"Man is distinguished, not only by his reason, but by this singular passion from other animals, which is a lust of the mind, that by a perseverance of delight in the continued and indefatigable generation of knowledge, exceeds the short vehemence of any carnal pleasure."#
        );
    }
}
