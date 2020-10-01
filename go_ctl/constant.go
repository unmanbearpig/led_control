package main

import (
)

type Constant struct {
	Value float64
}


func MakeConstant(value float64) Constant {
	return Constant{value}
}

func (c *Constant) Name() string {
	return "Constant"
}

func (c *Constant) Inputs() []Port {
	return []Port{};
}

func (c *Constant) Outputs() []Port {
	return []Port{
		Port{0, "constant value"},
	};
}

func (c *Constant) Xfer(inputs []float64) []float64 {
	return []float64{c.Value}
}
