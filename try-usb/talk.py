# requires python3-libusb1 or something like that
# import usb1

# context = usb1.USBContext()
# handle = context.openByVendorIDAndProductID(0xCAFE, 0xCAFE)
# device = handle.getDevice() # do I need it?

# print("serial: " + handle.getSerialNumber())

# # interface = handle.claimInterface(2)

# read_endp = 0x05
# data_len = 5
# data = handle.interruptRead(read_endp, data_len)

# print(data)


import usb.core
import usb.util

dev = usb.core.find(idVendor=0xcafe, idProduct=0xcafe)
if dev is None:
    print("could not open device")
    exit(1)

cfg = dev.get_active_configuration()
intf = cfg[(2,0)]

eps = intf.endpoints()
if len(eps) != 2:
    print("found expected 2 endpoints, got " + str(len(eps)))
    exit(1)

for ep in eps:
    print(str(ep))

in_endp = None
for ep in eps:
    if ep.bEndpointAddress == 0x86:
        in_endp = ep
        break

if in_endp == None:
    print("could not find IN endpoint")
    exit(1)

print("reading:")
print(in_endp.read(50))
