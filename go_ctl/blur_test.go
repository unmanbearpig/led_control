package main

import (
	"testing"
)

func TestBlur(t *testing.T) {
	b := MakeBlur("test", 0.1, 4)
	input := []float64{ 0.0, 1.0, 0.0, 0.0 }
	output := b.Xfer(input)

	if len(input) != len(output) {
		t.Errorf("len(input) != len(output) : %d != %d",
			len(input), len(output))
	}

	for _, i := range []int{0, 2, 3} {
		if output[i] <= input[i] {
			t.Errorf("output[%d] is less than input: out=%v in=%v",
				i, output[i], input[i])
		}
	}

	if output[1] < 0.99999999 {
		t.Errorf("input value of 1.0 shouldn't change much, but it is %v",
			output[1])
	}
}
