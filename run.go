package main

import (
	"encoding/binary"
	"io"
	"log"
	"os"
	"os/exec"
)

const (
	reqRender = iota
)
const (
	resPng = iota
	resTexErr
)

func readNum() uint32 {
	b := make([]byte, 4)
	if _, err := io.ReadFull(os.Stdin, b); err != nil {
		log.Fatalf("read num: %v", err)
	}
	return binary.BigEndian.Uint32(b)
}

func writeNum(num uint32) {
	b := make([]byte, 4)
	binary.BigEndian.PutUint32(b, num)
	os.Stdout.Write(b)
}

func main() {
	if readNum() != reqRender {
		log.Fatal("must use render type")
	}
	tex := make([]byte, readNum())
	if _, err := io.ReadFull(os.Stdin, tex); err != nil {
		log.Fatalf("read data: %v", err)
	}
	if err := os.WriteFile("/tmp/job.tex", tex, 0400); err != nil {
		log.Fatalf("write tex: %v", err)
	}
	texCmd := exec.Command("./texlive/texdir/bin/x86_64-linux/pdflatex", "-interaction=nonstopmode", "-halt-on-error", "-fmt=preamble", "-output-directory=/tmp", "job.tex")
	if texOut, err := texCmd.CombinedOutput(); err != nil {
		writeNum(resTexErr)
		writeNum(uint32(len(texOut)))
		os.Stdout.Write(texOut)
		return
	}
	gsCmd := exec.Command("./gs", "-q", "-sstdout=%stderr", "-dBATCH", "-dNOPAUSE", "-dSAFER", "-sOutputFile=-", "-dMaxBitmap=10485760", "-dTextAlphaBits=4", "-dGraphicsAlphaBits=4", "-r440", "-sDEVICE=png16m", "/tmp/job.pdf")
	gsOut, err := gsCmd.Output()
	if err != nil {
		log.Fatalf("exec gs: %v", err)
	}
	writeNum(resPng)
	writeNum(uint32(len(gsOut)))
	os.Stdout.Write(gsOut)
}
