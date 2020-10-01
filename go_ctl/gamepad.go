package main

import (
)

type Gamepad struct {
	hidrawPath string
}

func (g *Gamepad) Inputs() []Port {
	return []Port{};
}

func (g *Gamepad) Outputs() []Port {
	return []Port{
		Port{0, "channel 1"},
		Port{1, "channel 2"},
	};
}

func (g *Gamepad) Xfer(inputs []float64) []float64 {
	return []float64{0.4, 0.5}
}
