package main

import (
	"encoding/binary"
)

const UDPMsgMagic uint16 = 0x1324
const UDPMsgTypeWrite = 2
const UDPMsgFlagFloat = 1
var UDPMsgEndianness = binary.LittleEndian

type UDPFloat4Msg struct {
	Magic uint16
	Type uint16
	Flags uint16
	Amonut float32
	FloatValues [4]float32
}
