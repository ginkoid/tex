package main

import (
	"encoding/binary"
	"os"
	"os/exec"
)

type code uint32

const (
	codeOk code = iota
	codeErrTex
	codeErrGs
)

func writeCode(n code) {
	b := make([]byte, 4)
	binary.BigEndian.PutUint32(b, uint32(n))
	os.Stdout.Write(b)
}

func main() {
	texCmd := exec.Command(
		"/app/texlive/texdir/bin/x86_64-linux/pdflatex",
		"-interaction=scrollmode",
		"-halt-on-error",
		"-fmt=preamble",
		"-output-directory=/tmp",
		"-jobname=job",
	)
	texCmd.Stdin = os.Stdin
	if texOut, err := texCmd.CombinedOutput(); err != nil {
		os.Stdout.Write(texOut)
		writeCode(codeErrTex)
		return
	}
	gsCmd := exec.Command(
		"/app/gs",
		"-q",
		"-sstdout=%stderr",
		"-dBATCH",
		"-dNOPAUSE",
		"-dSAFER",
		"-sOutputFile=-",
		"-dMaxBitmap=10485760",
		"-dTextAlphaBits=4",
		"-dGraphicsAlphaBits=4",
		"-r440",
		"-sDEVICE=png16m",
		"/tmp/job.pdf",
	)
	gsCmd.Stdout = os.Stdout
	if err := gsCmd.Run(); err != nil {
		writeCode(codeErrGs)
		return
	}
	writeCode(codeOk)
}
