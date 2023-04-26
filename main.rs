use std::io::{self, Write};
use std::process::{Command, Stdio};

#[derive(Copy, Clone)]
enum Code {
    Ok = 0,
    ErrLatex = 1,
    ErrGs = 2,
    ErrInternal = 3,
}

fn write_code(code: Code) -> io::Result<()> {
    let bytes = (code as u32).to_be_bytes();
    io::stdout().write_all(&bytes)
}

fn run() -> io::Result<()> {
    let latex = Command::new(format!(
        "./texlive/texdir/bin/{}-linux/pdflatex",
        std::env::consts::ARCH
    ))
    .stdin(Stdio::inherit())
    .args([
        "-interaction=scrollmode",
        "-halt-on-error",
        "-fmt=preamble",
        "-output-directory=/tmp",
        "-jobname=job",
    ])
    .output()?;
    if !latex.status.success() {
        io::stdout().write_all(&latex.stdout)?;
        io::stdout().write_all(&latex.stderr)?;
        write_code(Code::ErrLatex)?;
        return Ok(());
    }
    let gs = Command::new("./gs")
        .stdout(Stdio::inherit())
        .args([
            "-q",
            "-sstdout=%stderr",
            "-dBATCH",
            "-dNOPAUSE",
            "-sOutputFile=-",
            "-dMaxBitmap=10485760",
            "-dTextAlphaBits=4",
            "-dGraphicsAlphaBits=4",
            "-r440",
            "-sDEVICE=png16m",
            "/tmp/job.pdf",
        ])
        .status()?;
    if !gs.success() {
        write_code(Code::ErrGs)?;
        return Ok(());
    }
    write_code(Code::Ok)?;
    Ok(())
}

fn main() {
    run().unwrap_or_else(|e| {
        eprintln!("{}", e);
        write_code(Code::ErrInternal).unwrap();
    });
}
