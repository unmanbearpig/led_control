package main

import (
	"fmt"
	"net"
	"encoding/binary"
)

type UDPOut struct {
	addr string
	conn *net.Conn
	ports []Port
}

func MakeUDPOut(addr string) (UDPOut, error) {
	conn, err := net.Dial("udp", addr)
	if err != nil {
		return UDPOut{"", nil, []Port{}}, fmt.Errorf("MakeUdpOut Dial: %s", err)
	}

	numChans := uint64(4)
	ports := make([]Port, 0, numChans)
	for i := uint64(0); i < numChans; i++ {
		ports = append(ports, Port{i, fmt.Sprint("channel ", i) })
	}

	udpOut := UDPOut{
		addr,
		&conn,
		ports,
	}

	return udpOut, nil
}

func (u *UDPOut) Name() string {
	return fmt.Sprintf("UDPOut %s %d channels", u.addr, len(u.ports))
}

func (u *UDPOut) Inputs() []Port {
	return u.ports
}

func (u *UDPOut) Outputs() []Port {
	return u.ports
}

const UDPMsgMagic uint16 = 0x1324
const UDPMsgTypeWrite = 2
const UDPMsgFlagFloat = 1

type UDPFloat4Msg struct {
	Magic uint16
	Type uint16
	Flags uint16
	Amonut float32
	FloatValues [4]float32
}

func (u *UDPOut) Xfer(inputs []float64) []float64 {

	// TODO: make bytes
	if len(inputs) != 4 {
		panic(
			fmt.Sprint("UDPOut Xfer expected 4 input values, got ",
				len(inputs)))
	}

	msg := UDPFloat4Msg{
		UDPMsgMagic,
		UDPMsgTypeWrite,
		UDPMsgFlagFloat,
		1.0,
		[4]float32{
			float32(inputs[0]),
			float32(inputs[1]),
			float32(inputs[2]),
			float32(inputs[3])},
	}

	err := binary.Write(*u.conn, binary.LittleEndian, msg)
	if err != nil {
		panic(fmt.Errorf("UDPOut Xfer write: %s", err))
	}

	return inputs
}
