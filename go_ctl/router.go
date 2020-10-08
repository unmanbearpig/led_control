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
	inputLinks []*Link
	outputLinks []*Link
	inputPorts []Port
	outputPorts []Port
	inputs []float64
	outputs []float64
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
	return fmt.Sprintf("Router \"%s\"", r.name)
}

func MakeRouter(name string, links []*Link) (r Router, err error) {
	r.Debug = false
	r.name = name
	devs := make([]Dev, 0)
	sourceDevs := make([]Dev, 0)
	devsWithInputs := make([]Dev, 0)

	// fmt.Println("Making router")

	for _, l := range links {
		if l.DevFrom == l.DevTo {
			panic(fmt.Sprintf("link has the same in and out dev: %v", l.String()))
		}

		if l.DevFrom == nil {
			r.inputLinks = append(r.inputLinks, l)
		} else {
			if !DevExists(devs, l.DevFrom) {
				devs = append(devs, l.DevFrom)
			}
		}

		if l.DevTo == nil {
			r.outputLinks = append(r.outputLinks, l)
		} else {
			if !DevExists(devs, l.DevTo) {
				devs = append(devs, l.DevTo)
			}

			if !DevExists(devsWithInputs, l.DevTo) {
				devsWithInputs = append(devsWithInputs, l.DevTo)
			}
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

	r.makeInputs()
	r.makeOutputs()

	r.ValidateRouting()

	return r, nil
}

func (r *Router) Inputs() []Port {
	return r.inputPorts
}

func (r *Router) Outputs() []Port {
	return r.outputPorts
}

func (r *Router) makeInputs() {
	ports := make([]Port, 0)

	maxInput := uint64(0)

	for _, l := range r.inputLinks {
		alreadyExists := false
		if l.PortFrom > maxInput {
			maxInput = l.PortFrom
		}
		for _, p := range ports {
			if p.Index == l.PortFrom {
				alreadyExists = true
				break
			}
		}

		if alreadyExists {
			continue
		}

		ports = append(ports,
			Port{l.PortFrom, fmt.Sprint("router input ", l.PortFrom)})
	}

	r.inputs = make([]float64, maxInput+1)
	r.inputPorts = ports
}

func (r *Router) makeOutputs() {
	ports := make([]Port, 0)
	maxOutput := uint64(0)

	for _, l := range r.outputLinks {
		if l.PortTo > maxOutput {
			maxOutput = l.PortTo
		}

		alreadyExists := false
		for _, p := range ports {
			if p.Index == l.PortTo {
				alreadyExists = true
				break
			}
		}

		if alreadyExists {
			continue
		}

		ports = append(ports,
			Port{l.PortTo, fmt.Sprint("router output ", l.PortTo)})
	}

	r.outputs = make([]float64, maxOutput+1)
	r.outputPorts = ports
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
	if dev == nil {
		return r.inputLinks
	}

	links = make([]*Link, 0, len(dev.Inputs()))
	for _, l := range r.Links {
		if l.DevTo == dev {
			links = append(links, l)
		}
	}

	return links
}

func (r *Router) OutputLinks(dev Dev) (links []*Link) {
	if dev == nil {
		return r.outputLinks
	}

	links = make([]*Link, 0, len(dev.Outputs()))
	for _, l := range (*r).Links {
		if l.DevFrom == dev {
			links = append(links, l)
		}
	}

	return links
}

func (r *Router) InputValues(dev Dev) []float64 {
	var inputs []float64
	var inputLinks []*Link

	if dev == nil {
		inputs = r.inputs
		inputLinks = r.inputLinks
	} else {
		inputs = make([]float64, len(dev.Inputs()))
		inputLinks = r.InputLinks(dev)
	}

	for _, l := range inputLinks {
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
		fmt.Printf("%v\n", link.Inspect())
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
	// if r.Debug {
	// 	fmt.Println("processDev", dev.Name())
	// }
	var outputs []float64

	if dev == nil {
		return
	}

	if r.HasAllInputs(dev) {
		outputs = dev.Xfer(r.InputValues(dev))
	} else {
		panic("not all inputs ready")
		// TODO: get inputs and Xfer
	}

	for _, l := range r.OutputLinks(dev) {
		// if r.Debug {
		// 	fmt.Println("output ", i, ": ", l.Inspect())
		// }
		l.HasValue = true
		l.Value = outputs[l.PortFrom] // outputs[i] worked

		if r.HasAllInputs(l.DevTo) {
			// if r.Debug {
			// 	fmt.Println(l.DevTo.Name(), "has all inputs: ", r.InputValues(l.DevTo))
			// }
			r.processDev(l.DevTo)
		}
	}
}

func (r *Router) Xfer(inputs []float64) []float64 {
	if r.Debug {
		fmt.Println("router xfer\n---------------")
	}
	r.Reset()

	r.inputs = inputs

	for _, l := range r.inputLinks {
		l.HasValue = true
		l.Value = inputs[l.PortFrom]
	}

	for _, l := range r.inputLinks {
		r.processDev(l.DevTo)
	}

	for _, srcDev := range r.sourceDevs {
		r.processDev(srcDev)
	}

	for _, l := range r.outputLinks {
		if !l.HasValue {
			r.PrintLinks()
			panic(fmt.Sprintf("output link %v has no value", (*l).String()))
		}

		r.outputs[l.PortTo] = l.Value
	}

	if r.Debug {
		r.PrintLinks()
	}

	return r.outputs
}
