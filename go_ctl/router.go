package main

import (
	"fmt"
)

type Router struct {
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

func MakeRouter(links []*Link) (r Router, err error) {
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

	// fmt.Println("Made router")

	return r, nil
}

func (r *Router) Reset() {
	for _, l := range r.Links {
		l.HasValue = false
	}
}

func (r *Router) InputLinks(dev Dev) (links []*Link) {
	if IsSource(dev) {
		return []*Link{}
	}

	links = make([]*Link, 0, len(dev.Inputs()))
	for _, l := range (*r).Links {
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

	for i, l := range r.InputLinks(dev) {
		if l.HasValue {
			inputs[i] = l.Value
		} else {
			panic("missing value when trying to get input values")
			// inputs[i] = math.NaN()
		}
	}

	return inputs
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
				fmt.Println(l.DevTo.Name(), "has all inputs")
			}
			r.processDev(l.DevTo)
		}
	}
}

func (r *Router) Xfer(_inputs []float64) []float64 {
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
