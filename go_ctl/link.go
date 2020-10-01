package main

import (
	"math"
	"fmt"
)

type Link struct {
	DevFrom Dev
	PortFrom int64
	DevTo Dev
	PortTo int64
	HasValue bool
	Value float64
}

func (l *Link) String() string {
	return fmt.Sprintf("%s[%d] -> %s[%d]", l.DevFrom.Name(), l.PortFrom, l.DevTo.Name(), l.PortTo)
}

func (l *Link) Inspect() string {
	return fmt.Sprintf("%s[%d] -> %s[%d]: %v %v", l.DevFrom.Name(), l.PortFrom, l.DevTo.Name(), l.PortTo, l.HasValue, l.Value)
}

func MakeLink(devFrom Dev, portFrom int64, devTo Dev, portTo int64) Link {
	return Link{devFrom, portFrom, devTo, portTo, false, math.NaN()}
}
