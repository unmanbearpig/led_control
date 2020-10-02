package main

import (
	"fmt"
	"math"
)

type PowConst struct {
	Value float64
	numChannels uint64
}


func MakePowConst(numChannels uint64, value float64) PowConst {
	return PowConst{value, numChannels}
}

func (c *PowConst) Name() string {
	return fmt.Sprintf("PowConst x^%f (%d channels)", c.Value, c.numChannels)
}

func (c *PowConst) Inputs() []Port {
	inputs := make([]Port, 0, c.numChannels)

	for i := uint64(0); i < c.numChannels; i++ {
		inputs = append(inputs, Port{uint64(i), fmt.Sprintf("input %d", i)})
	}

	return inputs
}

func (c *PowConst) Outputs() []Port {
	outputs := make([]Port, 0, c.numChannels)

	for i := uint64(0); i < c.numChannels; i++ {
		outputs = append(outputs, Port{uint64(i), fmt.Sprintf("output %d", i)})
	}

	return outputs
}

func (c *PowConst) Xfer(inputs []float64) []float64 {
	if uint64(len(inputs)) != c.numChannels {
		panic(fmt.Sprintf("PowConst expected %d inputs, got %d", c.numChannels, len(inputs)))
	}

	outputs := make([]float64, len(inputs))
	for i, f := range inputs {
		outputs[i] = math.Pow(f, c.Value)
	}

	return outputs
}
