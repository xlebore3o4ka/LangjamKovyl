package main

import (
	"fmt"
	"graft/codegen"
	"graft/lexer"
	"graft/parser"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
)

func main() {
	if len(os.Args) < 2 {
		fmt.Fprintf(os.Stderr, "Usage: graftc <file.gft> [--run]\n")
		os.Exit(1)
	}

	inputFile := os.Args[1]
	doRun := false

	for i := 2; i < len(os.Args); i++ {
		if os.Args[i] == "--run" {
			doRun = true
		}
	}

	base := filepath.Base(inputFile)
	name := strings.TrimSuffix(base, filepath.Ext(base))

	data, err := os.ReadFile(inputFile)
	if err != nil {
		fmt.Fprintf(os.Stderr, "error: cannot read %s: %v\n", inputFile, err)
		os.Exit(1)
	}

	lex := lexer.New(string(data))
	tokens, err := lex.Tokenize()
	if err != nil {
		fmt.Fprintf(os.Stderr, "lexer error: %v\n", err)
		os.Exit(1)
	}

	prog, err := parser.New(tokens).Parse()
	if err != nil {
		fmt.Fprintf(os.Stderr, "parser error: %v\n", err)
		os.Exit(1)
	}

	moduleName := name
	erlangCode := codegen.Generate(prog, moduleName)

	erlFile := name + ".erl"
	if err := os.WriteFile(erlFile, []byte(erlangCode), 0644); err != nil {
		fmt.Fprintf(os.Stderr, "error: cannot write %s: %v\n", erlFile, err)
		os.Exit(1)
	}
	fmt.Printf("graftc: %s -> %s\n", inputFile, erlFile)

	erlc := exec.Command("erlc", erlFile)
	erlc.Stdout = os.Stdout
	erlc.Stderr = os.Stderr
	if err := erlc.Run(); err != nil {
		fmt.Fprintf(os.Stderr, "error: erlc failed: %v\n", err)
		os.Exit(1)
	}
	fmt.Printf("graftc: %s -> %s\n", erlFile, name+".beam")

	os.Remove(erlFile)

	if doRun {
		beamDir, _ := filepath.Abs(".")
		erl := exec.Command("erl", "-noshell", "-pa", beamDir, "-eval", name+":main(), halt().")
		erl.Stdin = os.Stdin
		erl.Stdout = os.Stdout
		erl.Stderr = os.Stderr
		if err := erl.Run(); err != nil {
			fmt.Fprintf(os.Stderr, "error: runtime failed: %v\n", err)
			os.Exit(1)
		}
	}
}
