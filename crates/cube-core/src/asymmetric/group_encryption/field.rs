//! One pinned prime-order group; maintained constant-time scalar/field arithmetic.
use super::{failed, invalid};
use crate::{asymmetric::dh, errors::OperationError};
use crypto_bigint::{
    BoxedUint, Odd,
    modular::{BoxedMontyForm, BoxedMontyParams},
};
use rand_core::TryCryptoRng;
use std::sync::OnceLock;
use zeroize::Zeroizing;
pub(super) const WIDTH: usize = 256;
pub(super) const BITS: u32 = 2048;
pub(super) struct Group {
    pub(super) order: BoxedUint,
    field: BoxedMontyParams,
    scalars: BoxedMontyParams,
    pub(super) g1: BoxedMontyForm,
    pub(super) g2: BoxedMontyForm,
}
pub(super) fn group() -> &'static Group {
    static GROUP: OnceLock<Group> = OnceLock::new();
    GROUP.get_or_init(|| {
        let modulus = dh::ffdhe2048_modulus();
        let one = BoxedUint::one_with_precision(BITS);
        let order = modulus.wrapping_sub(&one).unbounded_shr_vartime(1);
        let field = BoxedMontyParams::new_vartime(Odd::new(modulus).expect("named odd prime"));
        let scalars =
            BoxedMontyParams::new_vartime(Odd::new(order.clone()).expect("named odd order"));
        let g1 = BoxedMontyForm::new(
            BoxedUint::from_be_slice(&[2], BITS).expect("generator"),
            &field,
        );
        let g2 = BoxedMontyForm::new(
            BoxedUint::from_be_hex(super::constants::GENERATOR2, BITS)
                .expect("pinned public generator"),
            &field,
        );
        Group {
            order,
            field,
            scalars,
            g1,
            g2,
        }
    })
}
impl Group {
    pub(super) fn element(
        &self,
        bytes: &[u8],
        identity: bool,
    ) -> Result<BoxedMontyForm, OperationError> {
        dh::subgroup_element(bytes, &self.field, identity).map_err(|_| failed())
    }
    pub(super) fn scalar(&self, bytes: &[u8]) -> Result<Zeroizing<BoxedUint>, OperationError> {
        if bytes.len() != WIDTH {
            return Err(invalid());
        }
        let n = Zeroizing::new(BoxedUint::from_be_slice(bytes, BITS).map_err(|_| invalid())?);
        if bool::from(n.is_zero()) || *n >= self.order {
            return Err(invalid());
        }
        Ok(n)
    }
    pub(super) fn random_scalar(
        &self,
        rng: &mut impl TryCryptoRng,
    ) -> Result<Zeroizing<BoxedUint>, OperationError> {
        for _ in 0..8 {
            let mut bytes = Zeroizing::new([0; WIDTH]);
            rng.try_fill_bytes(&mut bytes[..]).map_err(|_| failed())?;
            bytes[0] &= 0x7f;
            if let Ok(s) = self.scalar(&bytes[..]) {
                return Ok(s);
            }
        }
        Err(failed())
    }
    pub(super) fn product(&self, a: &BoxedUint, b: &BoxedUint) -> Zeroizing<BoxedUint> {
        let a = Zeroizing::new(BoxedMontyForm::new(a.clone(), &self.scalars));
        let b = Zeroizing::new(BoxedMontyForm::new(b.clone(), &self.scalars));
        Zeroizing::new(Zeroizing::new(&*a * &*b).retrieve())
    }
    pub(super) fn linear(
        &self,
        x: &BoxedUint,
        y: &BoxedUint,
        alpha: &BoxedUint,
    ) -> Zeroizing<BoxedUint> {
        let x = Zeroizing::new(BoxedMontyForm::new(x.clone(), &self.scalars));
        let y = Zeroizing::new(BoxedMontyForm::new(y.clone(), &self.scalars));
        let alpha = BoxedMontyForm::new(alpha.clone(), &self.scalars);
        let product = Zeroizing::new(&*y * &alpha);
        Zeroizing::new(Zeroizing::new(&*x + &*product).retrieve())
    }
    pub(super) fn inverse_exponent(&self, x: &BoxedUint) -> Zeroizing<BoxedUint> {
        Zeroizing::new(self.order.wrapping_sub(x))
    }
}
pub(super) fn encode(value: &BoxedMontyForm) -> Vec<u8> {
    Zeroizing::new(value.retrieve()).to_be_bytes().into_vec()
}
pub(super) fn encode_tuple(values: &[BoxedMontyForm]) -> Vec<u8> {
    values.iter().flat_map(encode).collect()
}
