package main

import (
	"fmt"
)

type Delay struct {
	buf []float64
	current uint
}

func MakeDelay(numSamples uint64) Delay {
	return Delay{
		make([]float64, numSamples, numSamples),
		0,
	}
}

func (d *Delay) Name() string {
	return fmt.Sprintf("Delay (%d samples)", len(d.buf))
}

func (d *Delay) Inputs() []Port {
	return []Port{ Port{ 0, "input" } }
}

func (d *Delay) Outputs() []Port {
	return []Port{ Port{ 0, "output" } }
}

func (d *Delay) push(value float64) {
	d.buf[d.current] = value
	d.current = (d.current + 1) % uint(len(d.buf))
}

func (d *Delay) get() float64 {
	return d.buf[d.current]
}

func (d *Delay) Xfer(inputs []float64) []float64 {
	if len(inputs) != 1 {
		panic(fmt.Sprintf("Delay expected 1 input, got %d", len(inputs)))
	}
	input := inputs[0]
	d.push(input)

	return []float64{d.get()}
}
