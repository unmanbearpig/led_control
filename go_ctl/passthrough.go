package main

import (
	"fmt"
)

type Passthrough struct {
	name string
	numChannels uint64
	ports []Port
}

func MakePassthrough(name string, numChannels uint64) Passthrough {
	ports := make([]Port, numChannels)

	for i := range ports {
		ports[i] = Port{uint64(i), fmt.Sprint("channel ", i)}
	}

	return Passthrough{
		name,
		numChannels,
		ports,
	}
}

func (p *Passthrough) Name() string {
	return p.name
}

func (p *Passthrough) Inputs() []Port {
	return p.ports
}

func (p *Passthrough) Outputs() []Port {
	return p.ports
}

func (p *Passthrough) Xfer(inputs []float64) []float64 {
	return inputs
}
