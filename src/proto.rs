#[derive(Copy, Clone)]
pub enum Code {
    Ok = 0,
    ErrTex = 1,
    ErrMupdf = 2,
    ErrInternal = 3,
}
