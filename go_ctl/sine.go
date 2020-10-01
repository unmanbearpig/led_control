package main

import (
	"fmt"
	"time"
	"math"
)

type Sine struct {
	inputPorts []Port
	outputPorts []Port
	positive bool
	phi float64
	lastT time.Time
}

func MakeSine(positive bool) Sine {
	return Sine{
		[]Port{ Port{0, "frequency"}, Port{1, "amplitude"} },
		[]Port{ Port{0, "value"} },
		positive,
		0,
		time.Now(),
	}
}

func (s *Sine) Name() string {
	return "Sine"
}

func (s *Sine) Inputs() []Port {
	return s.inputPorts
}

func (s *Sine) Outputs() []Port {
	return s.outputPorts
}

func (s *Sine) Xfer(inputs []float64) []float64 {
	if len(inputs) != 2 {
		panic(fmt.Sprint("sine expected 2 inputs, got ", len(inputs)))
	}

	freq := inputs[0]
	amp := inputs[1]

	t := time.Now()
	dt_sec := t.Sub(s.lastT).Seconds()
	s.phi += dt_sec * freq * math.Pi * 2.0

	result := math.Sin(s.phi) * amp

	if s.positive {
		result = (result + amp) / 2.0
	}

	s.lastT = t

	return []float64{result}
}
