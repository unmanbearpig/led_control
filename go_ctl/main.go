package main

import (
	"fmt"
	"os"
	"time"
)

func MakeSlowPropagation() (Router) {
	avg10 := MakeAverage(100)
	delay11 := MakeDelay(100)
	avg11 := MakeAverage(300)
	max11 := MakeMax(2)
	delay12 := MakeDelay(300)
	avg12 := MakeAverage(600)
	max12 := MakeMax(2)

	links := make([]Link, 0)
	links = append(links, MakeLink(nil, 0, &delay11, 0))
	links = append(links, MakeLink(&delay11, 0, &avg11, 0))
	links = append(links, MakeLink(&delay11, 0, &max11, 0))
	links = append(links, MakeLink(&avg11, 0, &max11, 1))
	links = append(links, MakeLink(nil, 0, &delay12, 0))
	links = append(links, MakeLink(&delay12, 0, &avg12, 0))
	links = append(links, MakeLink(&delay12, 0, &max12, 0))
	links = append(links, MakeLink(&avg12, 0, &max12, 1))
	links = append(links, MakeLink(nil, 0, &avg10, 0))
	links = append(links, MakeLink(&avg10, 0, nil, 0))
	links = append(links, MakeLink(&max11, 0, nil, 1))
	links = append(links, MakeLink(&max12, 0, nil, 2))

	plinks := make([]*Link, 0)
	for i := range links {
		plinks = append(plinks, &links[i])
	}

	r, err := MakeRouter("Slow Propagation", plinks)
	if err != nil {
		panic(fmt.Sprint("could not make slow propagation: ", err))
	}

	return r
}

func main() {
	udp, err := MakeUDPOut("192.168.0.102:8932")
	if err != nil {
		panic(fmt.Sprint("make udp out:", err))
	}

	udpArtIn, err := MakeUDPIn("0.0.0.0:9998")
	if err != nil {
		panic(fmt.Sprintf("make udp in: %s", err))
	}

	artOut1 := MakeAdd("artOut1", 3)
	artOut2 := MakeAdd("artOut2", 3)
	artOut3 := MakeAdd("artOut3", 3)

	links := make([]Link, 0)

	slowProp1 := MakeSlowPropagation()
	links = append(links, MakeLink(&udpArtIn, 0, &slowProp1, 0))
	links = append(links, MakeLink(&slowProp1, 0, &artOut1, 0))
	links = append(links, MakeLink(&slowProp1, 1, &artOut3, 0))
	links = append(links, MakeLink(&slowProp1, 2, &artOut2, 0))

	slowProp2 := MakeSlowPropagation()
	links = append(links, MakeLink(&udpArtIn, 1, &slowProp2, 0))
	links = append(links, MakeLink(&slowProp2, 0, &artOut3, 1))
	links = append(links, MakeLink(&slowProp2, 1, &artOut2, 1))
	links = append(links, MakeLink(&slowProp2, 2, &artOut1, 1))

	slowProp3 := MakeSlowPropagation()
	links = append(links, MakeLink(&udpArtIn, 2, &slowProp3, 0))
	links = append(links, MakeLink(&slowProp3, 0, &artOut2, 2))
	links = append(links, MakeLink(&slowProp3, 1, &artOut3, 2))
	links = append(links, MakeLink(&slowProp3, 2, &artOut1, 2))

	udpGamepadIn, err := MakeUDPIn("0.0.0.0:9999")
	if err != nil {
		panic(fmt.Sprintf("make udp in: %s", err))
	}

	blur := MakeBlur("output blur", 0.36, 3)

	// something wrong with the first strip
	outAdj := MakePowConst([]float64{ 2.05, 2.2, 2.2 })
	links = append(links, MakeLink(&blur, 0, &outAdj, 0))
	links = append(links, MakeLink(&blur, 2, &outAdj, 1))
	links = append(links, MakeLink(&blur, 1, &outAdj, 2))

	out1BeforeAdj := MakeAdd("out1BeforeAdj", 2)
	links = append(links, MakeLink(&udpGamepadIn, 3, &out1BeforeAdj, 0))
	links = append(links, MakeLink(&artOut1, 0, &out1BeforeAdj, 1))
	links = append(links, MakeLink(&out1BeforeAdj, 0, &blur, 0))

	out2BeforeAdj := MakeAdd("out2BeforeAdj", 2)
	links = append(links, MakeLink(&udpGamepadIn, 2, &out2BeforeAdj, 0))
	links = append(links, MakeLink(&artOut2, 0, &out2BeforeAdj, 1))
	links = append(links, MakeLink(&out2BeforeAdj, 0, &blur, 2))

	out3BeforeAdj := MakeAdd("out3BeforeAdj", 2)
	links = append(links, MakeLink(&udpGamepadIn, 1, &out3BeforeAdj, 0))
	links = append(links, MakeLink(&artOut3, 0, &out3BeforeAdj, 1))
	links = append(links, MakeLink(&out3BeforeAdj, 0, &blur, 1))

	udpLogger := MakeLogger(
		"udp logger",
		false,
		[]Port{
			Port{0, "udp0"},
			Port{1, "udp1"},
			Port{2, "udp2"},
			Port{3, "udp3"},
		})

	links = append(links, MakeLink(&outAdj, 0, &udpLogger, 0))
	links = append(links, MakeLink(&outAdj, 1, &udpLogger, 1))
	links = append(links, MakeLink(&outAdj, 2, &udpLogger, 2))
	links = append(links, MakeLink(&outAdj, 2, &udpLogger, 3))

	links = append(links, MakeLink(&udpLogger, 0, &udp, 0))
	links = append(links, MakeLink(&udpLogger, 1, &udp, 1))
	links = append(links, MakeLink(&udpLogger, 2, &udp, 2))
	links = append(links, MakeLink(&udpLogger, 3, &udp, 3))

	// links = append(links, MakeLink(&outAdj, 0, &udp, 0))
	// links = append(links, MakeLink(&outAdj, 1, &udp, 1))
	// links = append(links, MakeLink(&outAdj, 2, &udp, 2))
	// links = append(links, MakeLink(&outAdj, 2, &udp, 3))

	plinks := make([]*Link, 0)
	for i := range links {
		plinks = append(plinks, &links[i])
	}

	r, err := MakeRouter("Main", plinks)
	// r.PrintDevs()
	r.PrintLinks()

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
