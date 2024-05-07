use anyhow::{bail, Error, Result};

#[derive(Copy, Clone, Debug)]
pub enum Code {
    Ok = 0,
    ErrTex = 1,
    ErrMupdf = 2,
    ErrInternal = 3,
}

impl From<Code> for u32 {
    fn from(code: Code) -> Self {
        code as u32
    }
}

impl TryFrom<u32> for Code {
    type Error = Error;
    fn try_from(n: u32) -> Result<Self, Self::Error> {
        Ok(match n {
            0 => Self::Ok,
            1 => Self::ErrTex,
            2 => Self::ErrMupdf,
            3 => Self::ErrInternal,
            _ => bail!("invalid code: {}", n),
        })
    }
}

pub struct Response {
    pub code: Code,
    pub data: Vec<u8>,
}
