package main

import (
	"fmt"
)

type Add struct {
	name string
	inputChannels uint64
	inputPorts []Port
}

func MakeAdd(name string, numChannels uint64) Add {
	ports := make([]Port, numChannels)
	for i := uint64(0); i < numChannels; i++ {
		ports[i] = Port{i, fmt.Sprint("add input ", i)}
	}

	return Add{name, numChannels, ports}
}

func (a *Add) Name() string {
	return fmt.Sprintf("Add %s (%v channels)", a.name, a.inputChannels)
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
