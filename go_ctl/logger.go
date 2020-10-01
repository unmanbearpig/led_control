package main

import (
	"fmt"
)

type Logger struct {
	description string
	inputPorts []Port
}

func MakeLogger(description string, inputPorts []Port) Logger {
	return Logger{description, inputPorts}
}

func (l *Logger) Name() string {
	return l.description
}

func (l *Logger) Inputs() []Port {
	return l.inputPorts
}

func (l *Logger) Outputs() []Port {
	return l.inputPorts
}

func (l *Logger) Xfer(inputs []float64) []float64 {
	if len(inputs) != len(l.inputPorts) {
		panic(fmt.Sprint(
			"Logger: invalid number of inputs: ",
			len(inputs),
			" instead of ",
			len(l.inputPorts)))
	}

	for i, p := range l.inputPorts {
		fmt.Printf("%s: %20.20f\n", p.Description, inputs[i])
	}

	return inputs
}
