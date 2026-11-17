use super::{OperationError, invalid};

const N: usize = 11;
const P: i32 = 3;
const Q: i32 = 32;
type Polynomial = [i32; N];

fn decode(bytes: &[u8], modulus: i32) -> Result<Polynomial, OperationError> {
    let raw: [u8; N] = bytes.try_into().map_err(|_| invalid())?;
    let poly = raw.map(i32::from);
    if poly.iter().any(|&value| value >= modulus) {
        return Err(invalid());
    }
    Ok(poly)
}

fn multiply(left: &Polynomial, right: &Polynomial) -> Polynomial {
    let mut product = [0; N];
    for (i, &a) in left.iter().enumerate() {
        for (j, &b) in right.iter().enumerate() {
            product[(i + j) % N] += a * b;
        }
    }
    product
}

fn centered(value: i32, modulus: i32) -> i32 {
    (value + modulus / 2).rem_euclid(modulus) - modulus / 2
}

pub(super) fn encrypt(
    input: &[u8],
    key: &[u8],
    [positive, negative]: [usize; 2],
) -> Result<Vec<u8>, OperationError> {
    let message = decode(input, P)?.map(|value| centered(value, P));
    let public = decode(key, Q)?;
    let mut blind = [0; N];
    blind[positive] = 1;
    blind[negative] = -1;
    let masked = multiply(&blind, &public);
    Ok(masked
        .iter()
        .zip(message)
        .map(|(a, b)| (a + b).rem_euclid(Q) as u8)
        .collect())
}

pub(super) fn decrypt(input: &[u8], key: &[u8]) -> Result<Vec<u8>, OperationError> {
    let ciphertext = decode(input, Q)?;
    let (secret, inverse) = key.split_at(N);
    let secret = decode(secret, P)?.map(|value| centered(value, P));
    let inverse = decode(inverse, P)?;
    let mut identity = [0; N];
    identity[0] = 1;
    if secret.iter().filter(|&&value| value != 0).count() > 5
        || multiply(&inverse, &secret).map(|value| value.rem_euclid(P)) != identity
    {
        return Err(invalid());
    }
    let centered_product = multiply(&secret, &ciphertext).map(|value| centered(value, Q));
    Ok(multiply(&inverse, &centered_product)
        .map(|value| value.rem_euclid(P) as u8)
        .to_vec())
}
