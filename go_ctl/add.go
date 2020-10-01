package main

import (
	"fmt"
)

type Add struct {
	inputChannels uint64
	inputPorts []Port
}

func MakeAdd(numChannels uint64) Add {
	ports := make([]Port, numChannels)
	for i := uint64(0); i < numChannels; i++ {
		ports[i] = Port{i, fmt.Sprint("add input ", i)}
	}

	return Add{numChannels, ports}
}

func (a *Add) Name() string {
	return fmt.Sprint("Add (", a.inputChannels, " channels)")
}

func (a *Add) Inputs() []Port {
	return a.inputPorts
}

func (a *Add) Outputs() []Port {
	return []Port{ Port{0, "add result"} }
}

func (a *Add) Xfer(inputs []float64) []float64 {
	var result float64 = 0.0

	for _, f := range inputs {
		result += f
	}

	return []float64{result}
}
