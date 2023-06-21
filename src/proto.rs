use anyhow::{bail, Result, Error};

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
    fn try_from(n: u32) -> Result<Code> {
        Ok(match n {
            0 => Code::Ok,
            1 => Code::ErrTex,
            2 => Code::ErrMupdf,
            3 => Code::ErrInternal,
            _ => bail!("invalid code"),
        })
    }
}

pub struct Response {
    pub code: Code,
    pub data: Vec<u8>,
}
