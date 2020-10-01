package main

import (
	"fmt"
	"os"
	"time"
)

func main() {
	constant1 := MakeConstant(0.4)
	constant2 := MakeConstant(0.3)
	add := MakeAdd(2)
	freq := MakeConstant(0.1)
	amp := MakeConstant(3.0)
	amp2 := MakeConstant(1.0)
	sine := MakeSine(true)
	sine2 := MakeSine(true)
	logger := MakeLogger("Logger1", []Port{
		Port{ 0, "const value" },
		Port{ 1, "sine" },
	})
	sine1outConst := MakeConstant(0.3)
	sine1outMod := MakeProduct(2)
	// udpTee := MakeTee(4)
	udp, err := MakeUDPOut("192.168.0.102:8932")
	if err != nil {
		panic(fmt.Sprint("make udp:", err))
	}
	// link := MakeLink(&constant, 0, &logger, 0)

	links := make([]Link, 0)
	links = append(links, MakeLink(&constant1, 0, &add, 0))
	links = append(links, MakeLink(&constant2, 0, &add, 1))
	links = append(links, MakeLink(&add, 0, &logger, 0))
	links = append(links, MakeLink(&freq, 0, &sine, 0))
	links = append(links, MakeLink(&amp, 0, &sine, 1))
	links = append(links, MakeLink(&sine, 0, &sine2, 0))
	links = append(links, MakeLink(&sine, 0, &sine1outMod, 0))
	links = append(links, MakeLink(&sine1outConst, 0, &sine1outMod, 1))
	links = append(links, MakeLink(&sine1outMod, 0, &udp, 1))
	links = append(links, MakeLink(&amp2, 0, &sine2, 1))
	links = append(links, MakeLink(&sine2, 0, &logger, 1))
	links = append(links, MakeLink(&logger, 1, &udp, 0))
	// links = append(links, MakeLink(&logger, 1, &udp, 1))
	links = append(links, MakeLink(&logger, 1, &udp, 2))
	links = append(links, MakeLink(&logger, 1, &udp, 3))


	// links = append(links, MakeLink(&logger, 1, &udpTee, 0))
	// links = append(links, MakeLink(&udpTee, 0, &udp, 0))
	// links = append(links, MakeLink(&udpTee, 1, &udp, 1))
	// links = append(links, MakeLink(&udpTee, 2, &udp, 2))
	// links = append(links, MakeLink(&udpTee, 3, &udp, 3))

	plinks := make([]*Link, 0)
	for i := range links {
		plinks = append(plinks, &links[i])
	}

	r, err := MakeRouter(plinks)
	// r.Debug = true
	if err != nil {
		panic(fmt.Sprint("could not make router", err))
	}

	for {
		r.Xfer([]float64{})
		os.Stdout.Sync()
		time.Sleep(10000000)
	}

	os.Exit(0)
}
