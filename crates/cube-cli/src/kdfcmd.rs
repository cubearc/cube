//! Bounded exact-file CLI adaptation for secret KDF ports.
use clap::{Args, Subcommand, ValueEnum};
use cube_core::ciphers::kdf;
use std::{error::Error, io::Write, path::PathBuf};
use zeroize::Zeroizing;

#[derive(Clone, Copy, ValueEnum)]
pub(crate) enum Sha2 {
    Sha256,
    Sha512,
}
impl From<Sha2> for kdf::Sha2 {
    fn from(value: Sha2) -> Self {
        match value {
            Sha2::Sha256 => Self::Sha256,
            Sha2::Sha512 => Self::Sha512,
        }
    }
}
#[derive(Clone, Copy, ValueEnum)]
pub(crate) enum Digest {
    Md5,
    Sha256,
}

#[derive(Args)]
pub(crate) struct Files {
    #[arg(long)]
    secret_file: PathBuf,
    #[arg(long)]
    salt_file: PathBuf,
}
fn read(path: PathBuf, max: usize, id: &'static str) -> Result<Zeroizing<Vec<u8>>, Box<dyn Error>> {
    Ok(Zeroizing::new(
        crate::Input::file_or_stdin(Some(path)).read_bounded(max, id)?,
    ))
}
type SecretFiles = (Zeroizing<Vec<u8>>, Zeroizing<Vec<u8>>);
impl Files {
    fn read(self, salt_max: usize, id: &'static str) -> Result<SecretFiles, Box<dyn Error>> {
        Ok((
            read(self.secret_file, kdf::MAX_SECRET, id)?,
            read(self.salt_file, salt_max, id)?,
        ))
    }
}

#[derive(Subcommand)]
pub(crate) enum Command {
    /// Derive raw PBKDF2-HMAC bytes
    Pbkdf2 {
        #[command(flatten)]
        files: Files,
        #[arg(long, value_enum)]
        profile: Sha2,
        #[arg(long)]
        iterations: u32,
        #[arg(long)]
        output_bytes: u16,
    },
    /// Extract and expand with HKDF
    Hkdf {
        #[command(flatten)]
        files: Files,
        #[arg(long)]
        info_file: PathBuf,
        #[arg(long, value_enum)]
        profile: Sha2,
        #[arg(long)]
        output_bytes: u16,
    },
    /// Derive raw RFC 7914 scrypt bytes
    Scrypt {
        #[command(flatten)]
        files: Files,
        #[arg(long)]
        n: u32,
        #[arg(long)]
        r: u32,
        #[arg(long)]
        p: u32,
        #[arg(long)]
        output_bytes: u16,
    },
    /// Derive raw Argon2id v0x13 bytes
    Argon2id {
        #[command(flatten)]
        files: Files,
        #[arg(long)]
        memory_kib: u32,
        #[arg(long)]
        iterations: u32,
        #[arg(long)]
        lanes: u32,
        #[arg(long)]
        output_bytes: u16,
    },
    /// Derive legacy OpenSSL bytes, with the key followed by the IV
    EvpBytesToKey {
        #[command(flatten)]
        files: Files,
        #[arg(long, value_enum)]
        digest: Digest,
        #[arg(long)]
        count: u32,
        #[arg(long)]
        key_bytes: u16,
        #[arg(long)]
        iv_bytes: u16,
    },
}
pub(crate) fn run(command: Command, output: &mut impl Write) -> Result<(), Box<dyn Error>> {
    let bytes = match command {
        Command::Pbkdf2 {
            files,
            profile,
            iterations,
            output_bytes,
        } => {
            let (password, salt) = files.read(kdf::MAX_CONTEXT, kdf::PBKDF2)?;
            kdf::pbkdf2(
                &password,
                &salt,
                kdf::Pbkdf2 {
                    profile: profile.into(),
                    iterations,
                    output_bytes,
                },
            )?
        }
        Command::Hkdf {
            files,
            info_file,
            profile,
            output_bytes,
        } => {
            let (ikm, salt) = files.read(kdf::MAX_CONTEXT, kdf::HKDF)?;
            let info = read(info_file, kdf::MAX_CONTEXT, kdf::HKDF)?;
            kdf::hkdf(
                &ikm,
                &salt,
                &info,
                kdf::Hkdf {
                    profile: profile.into(),
                    output_bytes,
                },
            )?
        }
        Command::Scrypt {
            files,
            n,
            r,
            p,
            output_bytes,
        } => {
            let (password, salt) = files.read(kdf::MAX_SALT_HARD, kdf::SCRYPT)?;
            kdf::scrypt(
                &password,
                &salt,
                kdf::Scrypt {
                    n,
                    r,
                    p,
                    output_bytes,
                },
            )?
        }
        Command::Argon2id {
            files,
            memory_kib,
            iterations,
            lanes,
            output_bytes,
        } => {
            let (password, salt) = files.read(kdf::MAX_SALT_HARD, kdf::ARGON2ID)?;
            kdf::argon2id(
                &password,
                &salt,
                kdf::Argon2id {
                    memory_kib,
                    iterations,
                    lanes,
                    output_bytes,
                },
            )?
        }
        Command::EvpBytesToKey {
            files,
            digest,
            count,
            key_bytes,
            iv_bytes,
        } => {
            let (password, salt) = files.read(8, kdf::EVP_BYTES_TO_KEY)?;
            let digest = match digest {
                Digest::Md5 => kdf::LegacyDigest::Md5,
                Digest::Sha256 => kdf::LegacyDigest::Sha256,
            };
            let result = kdf::evp_bytes_to_key(
                &password,
                &salt,
                kdf::EvpBytesToKey {
                    digest,
                    count,
                    key_bytes,
                    iv_bytes,
                },
            )?;
            output.write_all(result.key())?;
            output.write_all(result.iv())?;
            return Ok(());
        }
    };
    output.write_all(bytes.as_slice())?;
    Ok(())
}
