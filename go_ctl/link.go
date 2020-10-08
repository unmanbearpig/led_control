package main

import (
	"math"
	"fmt"
)

type Link struct {
	DevFrom Dev
	PortFrom uint64
	DevTo Dev
	PortTo uint64
	HasValue bool
	Value float64
}

func (l *Link) String() string {
	var devFromName string
	var devToName string

	if l.DevFrom == nil {
		devFromName = "router-input-dev"
	} else {
		devFromName = l.DevFrom.Name()
	}

	if l.DevTo == nil {
		devToName = "router-output-dev"
	} else {
		devToName = l.DevTo.Name()
	}

	return fmt.Sprintf("%s[%d] -> %s[%d]", devFromName, l.PortFrom, devToName, l.PortTo)
}

func (l *Link) Inspect() string {
	return fmt.Sprintf("%s: %v %v", l.String(), l.HasValue, l.Value)
}

func MakeLink(devFrom Dev, portFrom uint64, devTo Dev, portTo uint64) Link {
	return Link{devFrom, portFrom, devTo, portTo, false, math.NaN()}
}
