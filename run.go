package main

import (
	"encoding/binary"
	"errors"
	"fmt"
	"io"
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

func readNum() (uint32, error) {
	b := make([]byte, 4)
	if _, err := io.ReadFull(os.Stdin, b); err != nil {
		return 0, fmt.Errorf("read num: %w", err)
	}
	return binary.BigEndian.Uint32(b), nil
}

func writeNum(num uint32) {
	b := make([]byte, 4)
	binary.BigEndian.PutUint32(b, num)
	os.Stdout.Write(b)
}

func run() error {
	req, err := readNum()
	if err != nil {
		return err
	}
	if req != reqRender {
		return errors.New("must use render type")
	}
	texLen, err := readNum()
	if err != nil {
		return err
	}
	texFile, err := os.Create("/tmp/job.tex")
	if err != nil {
		return err
	}
	if _, err := io.CopyN(texFile, os.Stdin, int64(texLen)); err != nil {
		return err
	}
	texCmd := exec.Command(
		"/app/texlive/texdir/bin/x86_64-linux/pdflatex",
		"-interaction=nonstopmode",
		"-halt-on-error",
		"-fmt=preamble",
		"-output-directory=/tmp",
		"job.tex",
	)
	if texOut, err := texCmd.CombinedOutput(); err != nil {
		writeNum(resTexErr)
		writeNum(uint32(len(texOut)))
		os.Stdout.Write(texOut)
		return nil
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
	gsOut, err := gsCmd.Output()
	if err != nil {
		return fmt.Errorf("exec gs: %w", err)
	}
	writeNum(resPng)
	writeNum(uint32(len(gsOut)))
	os.Stdout.Write(gsOut)
	return nil
}

func main() {
	if err := run(); err != nil {
		fmt.Fprintf(os.Stderr, "error: %v\n", err)
		os.Exit(1)
	}
}
