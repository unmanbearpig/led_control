package main

import (
	"fmt"
)

type Tee struct {
	outputChannels uint64
}

func MakeTee(numChannels uint64) Tee {
	return Tee{numChannels}
}

func (t *Tee) Name() string {
	return fmt.Sprintf("Tee (%d channel)", t.outputChannels)
}

func (t *Tee) Inputs() []Port {
	return []Port{Port{0, "input channel"}}
}

func (t *Tee) Outputs() []Port {
	ports := make([]Port, t.outputChannels)
	for i := uint64(0); i < t.outputChannels; i++ {
		ports[i] = Port{i, fmt.Sprint("channel ", i)}
	}

	return ports
}

func (t *Tee) Xfer(inputs []float64) []float64 {
	if len(inputs) != 1 {
		panic(fmt.Sprintf("Tee expected 1 input, got %d", len(inputs)))
	}

	input := inputs[0]

	outputs := make([]float64, t.outputChannels)
	for i := uint64(0); i < t.outputChannels; i++ {
		outputs[i] = input
	}

	return outputs
}
