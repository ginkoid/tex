package main

import (
	"encoding/binary"
	"io"
	"io/ioutil"
	"os"
	"os/exec"
)

const (
	responsePng = iota
	responseTexError
)

func writeCode(code uint32) {
	codeBytes := make([]byte, 4)
	binary.LittleEndian.PutUint32(codeBytes, code)
	os.Stdout.Write(codeBytes)
}

func main() {
	latexLenBytes := make([]byte, 4)
	if _, err := io.ReadFull(os.Stdin, latexLenBytes); err != nil {
		panic(err)
	}
	latexLen := binary.LittleEndian.Uint32(latexLenBytes)
	latexBytes := make([]byte, latexLen)
	if _, err := io.ReadFull(os.Stdin, latexBytes); err != nil {
		panic(err)
	}
	if err := ioutil.WriteFile("/tmp/job.tex", latexBytes, 0700); err != nil {
		panic(err)
	}
	latexCmd := exec.Command("/texlive/texdir/bin/x86_64-linux/pdflatex", "-interaction=nonstopmode", "-halt-on-error", "-fmt=/preamble", "-output-directory=/tmp", "job.tex")
	if latexOut, err := latexCmd.CombinedOutput(); err != nil {
		writeCode(responseTexError)
		os.Stdout.Write(latexOut)
		return
	}
	gsCmd := exec.Command("/gs", "-q", "-dBATCH", "-dNOPAUSE", "-dSAFER", "-sOutputFile=-", "-dMaxBitmap=10485760", "-dTextAlphaBits=4", "-dGraphicsAlphaBits=4", "-r440", "-sDEVICE=pngalpha", "/tmp/job.pdf")
	if gsOut, err := gsCmd.Output(); err != nil {
		panic(err)
	} else {
		writeCode(responsePng)
		os.Stdout.Write(gsOut)
	}
}
