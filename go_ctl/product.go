package main

import (
	"fmt"
)

type Product struct {
	inputChannels uint64
	inputPorts []Port
}

func MakeProduct(numChannels uint64) Product {
	ports := make([]Port, numChannels)
	for i := uint64(0); i < numChannels; i++ {
		ports[i] = Port{i, fmt.Sprint("add input ", i)}
	}

	return Product{numChannels, ports}
}

func (a *Product) Name() string {
	return fmt.Sprint("Product (", a.inputChannels, " channels)")
}

func (a *Product) Inputs() []Port {
	return a.inputPorts
}

func (a *Product) Outputs() []Port {
	return []Port{ Port{0, "add result"} }
}

func (a *Product) Xfer(inputs []float64) []float64 {
	var result float64 = 1.0

	for _, f := range inputs {
		result *= f
	}

	return []float64{result}
}
