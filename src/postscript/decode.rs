const C1: u16 = 52845;
const C2: u16 = 22719;

// todo: no copy
fn decrypt_inner(cipher: &[u8], mut r: u16) -> Vec<u8> {
    let mut decoded = Vec::new();

    for &c in cipher {
        decoded.push(c ^ (r >> 8) as u8);
        r = (c as u16).wrapping_add(r).wrapping_mul(C1).wrapping_add(C2);
    }

    decoded
}

pub(super) fn decrypt(cipher: &[u8]) -> Vec<u8> {
    decrypt_inner(cipher, 55665)
}

pub(super) fn decrypt_charstring(cipher: &[u8]) -> Vec<u8> {
    decrypt_inner(cipher, 4330)
}
