package main

import (
	"bytes"
	"fmt"
	"os"
	"net"
	"encoding/binary"
)

type UDPIn struct {
	addr *net.UDPAddr
	conn *net.UDPConn
	lastValues []float64 // TODO: I probably need to add a mutex
}

func MakeUDPIn(addrStr string) (UDPIn, error) {
	addr, err := net.ResolveUDPAddr("udp", addrStr)
	if err != nil {
		return UDPIn{nil, nil, []float64{0,0,0,0}},
			fmt.Errorf("MakeUDPIn invalid address: %s", err)
	}

	conn, err := net.ListenUDP("udp", addr)
	if err != nil {
		return UDPIn{nil, nil, []float64{0,0,0,0}},
			fmt.Errorf("MakeUDPIn Listen: %s", err)
	}

	initialValues := []float64{0.0, 0.0, 0.0, 0.0}

	u := UDPIn{
		addr,
		conn,
		initialValues,
	}

	go u.listen()

	return u, nil
}

func (u *UDPIn) listen() {
	buf := make([]byte, 1024)
	msg := UDPFloat4Msg{0, 0, 0, 0, [4]float32{0,0,0,0}}

	expectedMsgSize := 32 // binary.Size(msg)

	for {
		bytesRead, _, err := u.conn.ReadFrom(buf)

		if err != nil {
			panic(fmt.Errorf("UDPIn listen ReadFrom: %s", err))
		}

		if bytesRead != expectedMsgSize {
			fmt.Fprintf(os.Stderr, "UDPIn: invalid size packet: %d instead of %d\n",
				bytesRead, expectedMsgSize)
			continue
		}

		err = binary.Read(bytes.NewReader(buf), UDPMsgEndianness, &msg)
		if err != nil {
			fmt.Fprintf(os.Stderr, "UDPIn: could not parse packet: %s\n", err)
			continue
		}

		for i, v := range msg.FloatValues {
			u.lastValues[i] = float64(v)
		}
	}
}

func (u *UDPIn) Name() string {
	return fmt.Sprintf("UDP-In at [todo port]")
}

func (u *UDPIn) Inputs() []Port {
	return []Port{}
}

func (u *UDPIn) Outputs() []Port {
	return []Port{
		Port{0, "udp-in channel 1"},
		Port{1, "udp-in channel 2"},
		Port{2, "udp-in channel 3"},
		Port{3, "udp-in channel 4"}, }
}

func (u *UDPIn) Xfer(inputs []float64) []float64 {
	if len(inputs) != 0 {
		panic(fmt.Sprint("UDPIn has no inputs, but received ", len(inputs)))
	}

	vals := make([]float64, len(u.lastValues))
	for i, v := range u.lastValues {
		vals[i] = v
	}

	return vals
}
