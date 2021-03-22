package main

import (
	"encoding/binary"
	"errors"
	"io"
	"io/ioutil"
	"os"
	"os/exec"
)

const (
	requestRender = iota
)
const (
	responsePng = iota
	responseTexError
)

func readNum() uint32 {
	b := make([]byte, 4)
	if _, err := io.ReadFull(os.Stdin, b); err != nil {
		panic(err)
	}
	return binary.BigEndian.Uint32(b)
}

func writeNum(num uint32) {
	b := make([]byte, 4)
	binary.BigEndian.PutUint32(b, num)
	os.Stdout.Write(b)
}

func main() {
	if readNum() != requestRender {
		panic(errors.New("must use render type"))
	}
	tex := make([]byte, readNum())
	if _, err := io.ReadFull(os.Stdin, tex); err != nil {
		panic(err)
	}
	if err := ioutil.WriteFile("/tmp/job.tex", tex, 0400); err != nil {
		panic(err)
	}
	texCmd := exec.Command(
		"./texlive/texdir/bin/x86_64-linux/pdflatex", "-interaction=nonstopmode",
		"-halt-on-error", "-fmt=preamble", "-output-directory=/tmp", "job.tex",
	)
	if texOut, err := texCmd.CombinedOutput(); err != nil {
		writeNum(responseTexError)
		writeNum(uint32(len(texOut)))
		os.Stdout.Write(texOut)
		return
	}
	gsCmd := exec.Command(
		"./gs", "-q", "-sstdout=%stderr", "-dBATCH", "-dNOPAUSE", "-dSAFER",
		"-sOutputFile=-", "-dMaxBitmap=10485760", "-dTextAlphaBits=4",
		"-dGraphicsAlphaBits=4", "-r440", "-sDEVICE=png16m", "/tmp/job.pdf",
	)
	if gsOut, err := gsCmd.Output(); err != nil {
		panic(err)
	} else {
		writeNum(responsePng)
		writeNum(uint32(len(gsOut)))
		os.Stdout.Write(gsOut)
	}
}
