use std::{
    io::{self, Write},
    process::{Command, Stdio},
};
use tex::proto;

pub fn write_response(response: proto::Response) -> io::Result<()> {
    if response.data.len() > u32::MAX as usize {
        panic!("response too long")
    }
    let mut stdout = std::io::stdout().lock();
    stdout.write_all(&<u32>::from(response.code).to_be_bytes())?;
    stdout.write_all(&(response.data.len() as u32).to_be_bytes())?;
    stdout.write_all(&response.data)?;
    Ok(())
}

fn run() -> io::Result<proto::Response> {
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
        let mut data = latex.stdout;
        data.extend(&latex.stderr);
        return Ok(proto::Response {
            code: proto::Code::ErrTex,
            data,
        });
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
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;
    if !mutool.status.success() {
        return Ok(proto::Response {
            code: proto::Code::ErrMupdf,
            data: mutool.stderr,
        });
    }
    Ok(proto::Response {
        code: proto::Code::Ok,
        data: mutool.stdout,
    })
}

fn main() {
    let response = run().unwrap_or_else(|e| proto::Response {
        code: proto::Code::ErrInternal,
        data: e.to_string().into_bytes(),
    });
    write_response(response).unwrap();
}
