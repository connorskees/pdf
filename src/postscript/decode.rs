const R: u16 = 55665;
const C1: u16 = 52845;
const C2: u16 = 22719;

pub fn decrypt(cipher: &[u8]) -> Vec<u8> {
    let mut decoded = Vec::new();
    let mut r: u16 = 55665;

    for &c in cipher {
        decoded.push(c ^ (r >> 8) as u8);
        r = (c as u16).wrapping_add(r).wrapping_mul(C1).wrapping_add(C2);
    }

    decoded
}

// todo: no copy
pub fn decrypt_charstring(cipher: &[u8]) -> Vec<u8> {
    let mut decoded = Vec::new();
    let mut r: u16 = 4330;

    for &c in cipher {
        decoded.push(c ^ (r >> 8) as u8);
        r = (c as u16).wrapping_add(r).wrapping_mul(C1).wrapping_add(C2);
    }

    decoded
}
