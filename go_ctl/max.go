package main

import (
	"fmt"
)

type Max struct {
	inputChannels uint64
	inputPorts []Port
}

func MakeMax(numChannels uint64) Max {
	ports := make([]Port, numChannels)
	for i := uint64(0); i < numChannels; i++ {
		ports[i] = Port{i, fmt.Sprint("add input ", i)}
	}

	return Max{numChannels, ports}
}

func (a *Max) Name() string {
	return fmt.Sprint("Max (", a.inputChannels, " channels)")
}

func (a *Max) Inputs() []Port {
	return a.inputPorts
}

func (a *Max) Outputs() []Port {
	return []Port{ Port{0, "add result"} }
}

func (a *Max) Xfer(inputs []float64) []float64 {
	if len(inputs) != len(a.Inputs()) {
		panic(fmt.Sprintf("Max expected %d inputs, got %d values", len(a.Inputs()), len(inputs)))
	}

	result := inputs[0]

	for _, f := range inputs {
		if f > result {
			result = f
		}
	}

	return []float64{result}
}
