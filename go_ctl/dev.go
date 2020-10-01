package main

import (
)

type Port struct {
	Index uint64
	Description string
}

type Dev interface {
	Inputs() []Port
	Outputs() []Port
	Name() string
	Xfer(inputs []float64) []float64
}

func IsSource(dev Dev) bool {
	if len(dev.Inputs()) == 0 {
		return true
	}
	return false
}
