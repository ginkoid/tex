use std::io::{self, Write};
use std::process::{Command, Stdio};

#[derive(Copy, Clone)]
enum Code {
    Ok = 0,
    ErrLatex = 1,
    ErrMupdf = 2,
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
    .args([
        "-interaction=scrollmode",
        "-halt-on-error",
        "-fmt=preamble",
        "-output-directory=/tmp",
        "-jobname=job",
    ])
    .stdin(Stdio::inherit())
    .output()?;
    if !latex.status.success() {
        io::stdout().write_all(&latex.stdout)?;
        io::stdout().write_all(&latex.stderr)?;
        write_code(Code::ErrLatex)?;
        return Ok(());
    }
    let mutool = Command::new("./mutool")
        .args([
            "draw",
            "-r440",
            "-crgb",
            "-Fpng",
            "-q",
            "-o-",
            "/tmp/job.pdf",
        ])
        .stdout(Stdio::inherit())
        .stderr(Stdio::piped())
        .output()?;
    if !mutool.status.success() {
        io::stdout().write_all(&mutool.stderr)?;
        write_code(Code::ErrMupdf)?;
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
