package main

import (
	"fmt"
)

type Average struct {
	buf []float64
	current uint
}

func MakeAverage(numSamples uint64) Average {
	return Average{
		make([]float64, numSamples, numSamples),
		0,
	}
}

func (d *Average) Name() string {
	return fmt.Sprintf("Average (%d samples)", len(d.buf))
}

func (d *Average) Inputs() []Port {
	return []Port{ Port{ 0, "input" } }
}

func (d *Average) Outputs() []Port {
	return []Port{ Port{ 0, "output" } }
}

func (d *Average) push(value float64) {
	d.buf[d.current] = value
	d.current = (d.current + 1) % uint(len(d.buf))
}

func (d *Average) get() float64 {
	return d.buf[d.current]
}

func (d *Average) average() (result float64) {
	result = 0
	for _, v := range d.buf {
		result += v
	}

	result = result / float64(len(d.buf))
	return result
}

func (d *Average) Xfer(inputs []float64) []float64 {
	if len(inputs) != 1 {
		panic(fmt.Sprintf("Average expected 1 input, got %d", len(inputs)))
	}
	input := inputs[0]
	d.push(input)

	return []float64{d.average()}
}
