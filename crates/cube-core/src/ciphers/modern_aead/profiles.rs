//! One declaration selects each exact backend, lengths and message ceiling.
use super::*;
use aegis::compat::rustcrypto_traits_06::{Aegis128L, Aegis256};
type Runner = fn(&Settings, &[u8], &mut [u8], Option<&[u8]>) -> Result<Vec<u8>, OperationError>;
pub(super) struct Spec {
    pub name: &'static str,
    pub key: usize,
    pub nonce: usize,
    pub tag: usize,
    pub body_limit: usize,
    pub run: Runner,
}
macro_rules! profiles {
    ($(($variant:ident,$name:literal,$runner:path,$key:literal,$nonce:literal,$tag:literal,$limit:expr)),+$(,)?)=>{
        #[derive(Clone,Copy,Debug,Deserialize,JsonSchema)]
        pub enum Profile {$(#[serde(rename=$name)] $variant),+}
        const SPECS:&[Spec]=&[$(Spec{name:$name,key:$key,nonce:$nonce,tag:$tag,body_limit:$limit,run:$runner}),+];
        impl Profile {
            pub const NAMES:&[&str]=&[$($name),+];
            pub(super) fn spec(self)->&'static Spec {&SPECS[self as usize]}
        }
        pub(super) fn specs()->&'static [Spec]{SPECS}
    }
}
profiles! {
 (Aes128Gcm,"aes128_gcm",backend::standard::<aes_gcm::Aes128Gcm>,16,12,16,MAX_INPUT),
 (Aes256Gcm,"aes256_gcm",backend::standard::<aes_gcm::Aes256Gcm>,32,12,16,MAX_INPUT),
 (ChaCha20Poly1305,"chacha20poly1305_ietf",backend::standard::<chacha20poly1305::ChaCha20Poly1305>,32,12,16,MAX_INPUT),
 (XChaCha20Poly1305,"xchacha20poly1305",backend::standard::<chacha20poly1305::XChaCha20Poly1305>,32,24,16,MAX_INPUT),
 (Aes128GcmSiv,"aes128_gcm_siv",backend::standard::<aes_gcm_siv::Aes128GcmSiv>,16,12,16,MAX_INPUT),
 (Aes256GcmSiv,"aes256_gcm_siv",backend::standard::<aes_gcm_siv::Aes256GcmSiv>,32,12,16,MAX_INPUT),
 (AsconAead128,"ascon_aead128_nist_sp800_232",backend::standard::<ascon_aead::AsconAead128>,16,16,16,MAX_INPUT),
 (Aes128Ccm13Tag8,"aes128_ccm_nonce13_tag8",backend::standard::<ccm::Ccm<aes::Aes128,ccm::consts::U8,ccm::consts::U13>>,16,13,8,65535),
 (Aes128Ccm12Tag16,"aes128_ccm_nonce12_tag16",backend::standard::<ccm::Ccm<aes::Aes128,ccm::consts::U16,ccm::consts::U12>>,16,12,16,(1<<24)-1),
 (Aes256Ccm12Tag16,"aes256_ccm_nonce12_tag16",backend::standard::<ccm::Ccm<aes::Aes256,ccm::consts::U16,ccm::consts::U12>>,32,12,16,(1<<24)-1),
 (Aes128Eax,"aes128_eax_nonce16_tag16",backend::legacy_eax::<eax::Eax<aes08::Aes128>>,16,16,16,MAX_INPUT),
 (Aes256Eax,"aes256_eax_nonce16_tag16",backend::legacy_eax::<eax::Eax<aes08::Aes256>>,32,16,16,MAX_INPUT),
 (Aes128Ocb3,"aes128_ocb3_nonce12_tag16",backend::standard::<ocb3::Ocb3<aes::Aes128,ocb3::consts::U12>>,16,12,16,MAX_INPUT),
 (Aes256Ocb3,"aes256_ocb3_nonce12_tag16",backend::standard::<ocb3::Ocb3<aes::Aes256,ocb3::consts::U12>>,32,12,16,MAX_INPUT),
 (Aegis128L128,"aegis128l_rfc10032_tag128",backend::standard::<Aegis128L<16>>,16,16,16,MAX_INPUT),
 (Aegis128L256,"aegis128l_rfc10032_tag256",backend::standard::<Aegis128L<32>>,16,16,32,MAX_INPUT),
 (Aegis256128,"aegis256_rfc10032_tag128",backend::standard::<Aegis256<16>>,32,32,16,MAX_INPUT),
 (Aegis256256,"aegis256_rfc10032_tag256",backend::standard::<Aegis256<32>>,32,32,32,MAX_INPUT),
 (DeoxysII128,"deoxys_ii_128_128",backend::standard::<deoxys::DeoxysII128>,16,15,16,MAX_INPUT),
 (DeoxysII256,"deoxys_ii_256_128",backend::standard::<deoxys::DeoxysII256>,32,15,16,MAX_INPUT),
}
