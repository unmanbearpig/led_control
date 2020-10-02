package main

import (
	"fmt"
	"os"
	"time"
)

func main() {
	// freq := MakeConstant(0.1)
	// amp := MakeConstant(3.0)
	// amp2 := MakeConstant(1.0)
	// sine := MakeSine(true)
	// sine2 := MakeSine(true)
	// logger := MakeLogger("Logger1", []Port{
	// 	Port{ 0, "const value" },
	// 	Port{ 1, "sine" },
	// })
	// sine1outConst := MakeConstant(0.3)
	// sine1outMod := MakeProduct(2)
	// udpTee := MakeTee(4)
	udp, err := MakeUDPOut("192.168.0.102:8932")
	if err != nil {
		panic(fmt.Sprint("make udp out:", err))
	}
	// link := MakeLink(&constant, 0, &logger, 0)

	// udpIn, err := MakeUDPIn("0.0.0.0:9999")
	// if err != nil {
	// 	panic(fmt.Sprint("make udp in: %s", err))
	// }

	udpArtIn, err := MakeUDPIn("0.0.0.0:9998")
	if err != nil {
		panic(fmt.Sprint("make udp in: %s", err))
	}

	// logUdpIn := MakeLogger("UDPIn", []Port{
	// 	Port{0, "udp-in chan1"},
	// })

	// mulArt := MakeProduct(2)

	avg10 := MakeAverage(50)
	delay11 := MakeDelay(100)
	avg11 := MakeAverage(300)
	max11 := MakeMax(2)
	delay12 := MakeDelay(300)
	avg12 := MakeAverage(600)
	max12 := MakeMax(2)

	avg20 := MakeAverage(50)
	delay21 := MakeDelay(100)
	avg21 := MakeAverage(300)
	max21 := MakeMax(2)
	delay22 := MakeDelay(300)
	avg22 := MakeAverage(600)
	max22 := MakeMax(2)

	avg30 := MakeAverage(50)
	delay31 := MakeDelay(100)
	avg31 := MakeAverage(300)
	max31 := MakeMax(2)
	delay32 := MakeDelay(300)
	avg32 := MakeAverage(600)
	max32 := MakeMax(2)


	artOut1 := MakeAdd(3)
	artOut2 := MakeAdd(3)
	artOut3 := MakeAdd(3)

	links := make([]Link, 0)
	links = append(links, MakeLink(&udpArtIn, 0, &delay11, 0))
	links = append(links, MakeLink(&delay11, 0, &avg11, 0))
	links = append(links, MakeLink(&delay11, 0, &max11, 0))
	links = append(links, MakeLink(&avg11, 0, &max11, 1))
	links = append(links, MakeLink(&udpArtIn, 0, &delay12, 0))
	links = append(links, MakeLink(&delay12, 0, &avg12, 0))
	links = append(links, MakeLink(&delay12, 0, &max12, 0))
	links = append(links, MakeLink(&avg12, 0, &max12, 1))
	links = append(links, MakeLink(&udpArtIn, 0, &avg10, 0))
	links = append(links, MakeLink(&avg10, 0, &artOut1, 0))
	links = append(links, MakeLink(&max11, 0, &artOut3, 0))
	links = append(links, MakeLink(&max12, 0, &artOut2, 0))

	links = append(links, MakeLink(&udpArtIn, 1, &delay21, 0))
	links = append(links, MakeLink(&delay21, 0, &avg21, 0))
	links = append(links, MakeLink(&delay21, 0, &max21, 0))
	links = append(links, MakeLink(&avg21, 0, &max21, 1))
	links = append(links, MakeLink(&udpArtIn, 1, &delay22, 0))
	links = append(links, MakeLink(&delay22, 0, &avg22, 0))
	links = append(links, MakeLink(&delay22, 0, &max22, 0))
	links = append(links, MakeLink(&avg22, 0, &max22, 1))
	links = append(links, MakeLink(&udpArtIn, 1, &avg20, 0))
	links = append(links, MakeLink(&avg20, 0, &artOut3, 1))
	links = append(links, MakeLink(&max21, 0, &artOut2, 1))
	links = append(links, MakeLink(&max22, 0, &artOut1, 1))

	links = append(links, MakeLink(&udpArtIn, 2, &delay31, 0))
	links = append(links, MakeLink(&delay31, 0, &avg31, 0))
	links = append(links, MakeLink(&delay31, 0, &max31, 0))
	links = append(links, MakeLink(&avg31, 0, &max31, 1))
	links = append(links, MakeLink(&udpArtIn, 2, &delay32, 0))
	links = append(links, MakeLink(&delay32, 0, &avg32, 0))
	links = append(links, MakeLink(&delay32, 0, &max32, 0))
	links = append(links, MakeLink(&avg32, 0, &max32, 1))
	links = append(links, MakeLink(&udpArtIn, 2, &avg30, 0))
	links = append(links, MakeLink(&avg30, 0, &artOut2, 2))
	links = append(links, MakeLink(&max31, 0, &artOut3, 2))
	links = append(links, MakeLink(&max32, 0, &artOut1, 2))


	outAdj := MakePowConst(3, 2.2)
	links = append(links, MakeLink(&artOut1, 0, &outAdj, 0))
	links = append(links, MakeLink(&artOut2, 0, &outAdj, 1))
	links = append(links, MakeLink(&artOut3, 0, &outAdj, 2))

	links = append(links, MakeLink(&outAdj, 0, &udp, 0))
	links = append(links, MakeLink(&outAdj, 1, &udp, 1))
	links = append(links, MakeLink(&outAdj, 2, &udp, 2))
	links = append(links, MakeLink(&outAdj, 2, &udp, 3))



	// links = append(links, MakeLink(&freq, 0, &sine, 0))
	// links = append(links, MakeLink(&amp, 0, &sine, 1))
	// // links = append(links, MakeLink(&sine, 0, &sine2, 0))
	// links = append(links, MakeLink(&udpIn, 0, &sine2, 0))
	// links = append(links, MakeLink(&sine, 0, &sine1outMod, 0))
	// links = append(links, MakeLink(&sine1outConst, 0, &sine1outMod, 1))
	// links = append(links, MakeLink(&amp2, 0, &sine2, 1))
	// links = append(links, MakeLink(&sine2, 0, &logger, 1))
	// // links = append(links, MakeLink(&logger, 1, &udp, 1))
	// // links = append(links, MakeLink(&logger, 1, &udp, 2))
	// links = append(links, MakeLink(&udpIn, 0, &logUdpIn, 0))
	// links = append(links, MakeLink(&logger, 1, &udp, 0))
	// links = append(links, MakeLink(&udpArtIn, 0, &mulArt, 0))
	// links = append(links, MakeLink(&sine1outMod, 0, &mulArt, 1))
	// links = append(links, MakeLink(&mulArt, 0, &udp, 1))
	// links = append(links, MakeLink(&udpIn, 0, &udp, 2))
	// links = append(links, MakeLink(&logger, 1, &udp, 3))

	// links = append(links, MakeLink(&logger, 1, &udpTee, 0))
	// links = append(links, MakeLink(&udpTee, 0, &udp, 0))
	// links = append(links, MakeLink(&udpTee, 1, &udp, 1))
	// links = append(links, MakeLink(&udpTee, 2, &udp, 2))
	// links = append(links, MakeLink(&udpTee, 3, &udp, 3))

	plinks := make([]*Link, 0)
	for i := range links {
		plinks = append(plinks, &links[i])
	}

	r, err := MakeRouter("Main", plinks)
	r.PrintDevs()
	// r.PrintLinks()

	// r.Debug = true
	if err != nil {
		panic(fmt.Sprint("could not make router", err))
	}

	ticker := time.NewTicker(1000000)

	for {
		r.Xfer([]float64{})
		os.Stdout.Sync()
		<-ticker.C
	}

	os.Exit(0)
}
