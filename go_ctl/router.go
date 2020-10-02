package main

import (
	"fmt"
	"strings"
	"sort"
)

type Router struct {
	name string
	Debug bool
	Devs []Dev
	Links []*Link
	sourceDevs []Dev // should only include device from Devs
}

func DevExists(devs []Dev, dev Dev) bool {
	for _, d := range devs {
		if d == dev {
			return true
		}
	}
	return false
}

func (r *Router) Name() string {
	return fmt.Sprint("Router", r.Name)
}

func MakeRouter(name string, links []*Link) (r Router, err error) {
	r.Debug = false
	devs := make([]Dev, 0)
	sourceDevs := make([]Dev, 0)
	devsWithInputs := make([]Dev, 0)

	// fmt.Println("Making router")

	for _, l := range links {
		// fmt.Println("link ", l.String())

		// todo error if fromDev = toDev
		if !DevExists(devs, l.DevFrom) {
			devs = append(devs, l.DevFrom)
		}

		if !DevExists(devs, l.DevTo) {
			devs = append(devs, l.DevTo)
		}

		if !DevExists(devsWithInputs, l.DevTo) {
			devsWithInputs = append(devsWithInputs, l.DevTo)
		}
	}

	for _, d := range devs {
		if !DevExists(devsWithInputs, d) {
			sourceDevs = append(sourceDevs, d)
		}
	}

	r.Devs = devs
	r.Links = links
	r.sourceDevs = sourceDevs

	r.ValidateRouting()

	// fmt.Println("Made router")

	return r, nil
}

func (r *Router) ValidateRouting() {
	errors := make([]string, 0)

	for _, dev := range r.Devs {
		expectedInPorts := make([]uint64, 0)
		for i := range dev.Inputs() {
			expectedInPorts = append(expectedInPorts, uint64(i))
		}

		actualInPorts := make([]uint64, 0)
		for _, l := range r.InputLinks(dev) {
			actualInPorts = append(actualInPorts, l.PortTo)
		}

		if len(expectedInPorts) != len(actualInPorts) {
			errors = append(errors, (fmt.Sprintf(
				"Routing error: Device %v should have %v inputs but %v are routed",
				dev.Name(),
				len(expectedInPorts),
				len(actualInPorts),
			)))
			continue
		}

		sort.Slice(actualInPorts, func(i, j int) bool {
			return actualInPorts[i] < actualInPorts[j]
		})

		for i := range expectedInPorts {
			if expectedInPorts[i] != actualInPorts[i] {
				errors = append(errors, fmt.Sprintf("Routing error: Device %v has input %v instead of %v",
					dev.Name(),
					actualInPorts[i],
					expectedInPorts[i],
				))
			}
		}
	}

	if len(errors) != 0 {
		panic(strings.Join(errors, "\n"))
	}
}

func (r *Router) Reset() {
	for _, l := range r.Links {
		l.HasValue = false
	}
}

func (r *Router) InputLinks(dev Dev) (links []*Link) {
	links = make([]*Link, 0, len(dev.Inputs()))
	for _, l := range r.Links {
		if l.DevTo == dev {
			links = append(links, l)
		}
	}

	return links
}

func (r *Router) OutputLinks(dev Dev) (links []*Link) {
	links = make([]*Link, 0, len(dev.Outputs()))
	for _, l := range (*r).Links {
		if l.DevFrom == dev {
			links = append(links, l)
		}
	}

	return links
}

func (r *Router) InputValues(dev Dev) []float64 {
	inputs := make([]float64, len(dev.Inputs()))

	for _, l := range r.InputLinks(dev) {
		if l.HasValue {
			inputs[l.PortTo] = l.Value
		} else {
			panic("missing value when trying to get input values")
			// inputs[i] = math.NaN()
		}
	}

	return inputs
}

func (r *Router) PrintDevs() {
	fmt.Println("Devices:")
	for _, dev := range r.Devs {
		fmt.Printf("%v: %d inputs, %d outputs\n",
			dev.Name(),
			len(r.InputLinks(dev)),
			len(r.OutputLinks(dev)))
	}
	fmt.Println("")
}

func (r *Router) PrintLinks() {
	fmt.Println("Links:")
	for _, link := range r.Links {
		fmt.Printf("%v\n", link.String())
	}
	fmt.Println("")
}

func (r *Router) HasAllInputs(dev Dev) bool {
	for _, l := range r.InputLinks(dev) {
		if !l.HasValue {
			return false
		}
	}
	return true
}

func (r *Router) processDev(dev Dev) {
	if r.Debug {
		fmt.Println("processDev", dev.Name())
	}
	var outputs []float64

	if r.HasAllInputs(dev) {
		outputs = dev.Xfer(r.InputValues(dev))
	} else {
		panic("not all inputs ready")
		// TODO: get inputs and Xfer
	}

	for i, l := range r.OutputLinks(dev) {
		if r.Debug {
			fmt.Println("output ", i, ": ", l.Inspect())
		}
		l.HasValue = true
		l.Value = outputs[l.PortFrom] // outputs[i] worked

		if r.HasAllInputs(l.DevTo) {
			if r.Debug {
				fmt.Println(l.DevTo.Name(), "has all inputs: ", r.InputValues(l.DevTo))
			}
			r.processDev(l.DevTo)
		}
	}
}

func (r *Router) Xfer(inputs []float64) []float64 {
	if r.Debug {
		fmt.Println("router xfer\n---------------")
	}
	r.Reset()

	for _, srcDev := range r.sourceDevs {
		if r.Debug {
			fmt.Println("Router Xfer dev", srcDev.Name())
		}
		r.processDev(srcDev)
	}

	// for _, l := range r.Links {
	// 	fmt.Println(l.Inspect())
	// }

	return []float64{}
}
