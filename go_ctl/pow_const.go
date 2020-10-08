package main

import (
	"fmt"
	"math"
)

type PowConst struct {
	Adjustments []float64
}


func MakePowConst(adjustments []float64) PowConst {
	return PowConst{adjustments}
}

func (c *PowConst) Name() string {
	return fmt.Sprintf("PowConst %v", c.Adjustments)
}

func (c *PowConst) Inputs() []Port {
	inputs := make([]Port, len(c.Adjustments))

	for i, adj := range c.Adjustments {
		inputs[i] = Port{ uint64(i), fmt.Sprintf("input x^%v", adj) }
	}

	return inputs
}

func (c *PowConst) Outputs() []Port {
	outputs := make([]Port, len(c.Adjustments))

	for i, adj := range c.Adjustments {
		outputs[i] = Port{ uint64(i), fmt.Sprintf("output x^%v", adj) }
	}

	return outputs
}

func (c *PowConst) Xfer(inputs []float64) []float64 {
	if len(inputs) != len(c.Adjustments) {
		panic(fmt.Sprintf("PowConst expected %d inputs, got %d", len(c.Adjustments), len(inputs)))
	}

	outputs := make([]float64, len(inputs))
	for i, f := range inputs {
		outputs[i] = math.Pow(f, c.Adjustments[i])
	}

	return outputs
}
