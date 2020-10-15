package main

import (
	"fmt"
	"math"
)

type Blur struct {
	name string
	radius float64
	numChannels uint64
}

func MakeBlur(name string, radius float64, numChannels uint64) Blur {
	return Blur{ name, radius, numChannels }
}

func (b *Blur) Name() string {
	return b.name
}

func (b *Blur) Inputs() []Port {
	inputs := make([]Port, b.numChannels)
	for i := range inputs {
		inputs[i] = Port{ uint64(i), fmt.Sprintf("input channel %d", i) }
	}
	return inputs
}

func (b *Blur) Outputs() []Port {
	outputs := make([]Port, b.numChannels)
	for i := range outputs {
		outputs[i] = Port{ uint64(i), fmt.Sprintf("output channel %d", i) }
	}
	return outputs
}

func (b *Blur) Xfer(inputs []float64) []float64 {
	outputs := blur(b.radius, inputs)
	return outputs
}

func gaussian(h, center, std, x float64) float64 {
	return h * math.Exp(-(math.Pow((x - center), 2.0) / (2 * math.Pow(std, 2.0))))
}

// modifies slice in place
func normalize(vals []float64) {
	l := float64(len(vals) +1)

	for i := range vals {
		vals[i] = vals[i] / l
	}
}

func maxSlice(vals []float64) float64 {
	m := vals[0]
	for _, v := range vals {
		m = math.Max(m, v)
	}
	return m
}

// modifies slice in place
func normalizeToMax(targetMax float64, vals []float64) {
	currentMax := maxSlice(vals)
	if currentMax == 0.0 {
		return
	}

	adj := targetMax / currentMax
	for i := range vals {
		vals[i] *= adj
	}
}

// returns new slice
func blur(radius float64, ins []float64) []float64 {
	blurred := blurRaw(radius, ins)
	normalizeToMax(maxSlice(ins), blurred)
	return blurred
}

// returns new slice
func blurRaw(radius float64, ins []float64) []float64 {
	outs := make([]float64, len(ins))
	copy(outs, ins)

	for i1 := range ins {
		g1 := float64(i1) / float64(len(ins))
		for i2, v2 := range ins {
			g2 := float64(i2) / float64(len(ins))
			outs[i1] += gaussian(v2, g1, radius, g2)
		}
	}
	normalize(outs)
	return outs
}
