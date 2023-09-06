"""\
This script echos a count value from the CAN bus and prints it to the console.
It is intended to be used with the RS485-CAN HAT on a Raspberry Pi.
https://www.waveshare.com/rs485-can-hat.htm
"""
import os
import can

os.system("sudo ip link set can0 type can bitrate 100000")
os.system("sudo ifconfig can0 up")

can0 = can.interface.Bus(channel="can0", bustype="socketcan")  # socketcan_native

count = 0
while True:
    # send message
    msg = can.Message(arbitration_id=0x123, data=[count], is_extended_id=False)
    can0.send(msg)

    # receive message
    msg = can0.recv(10.0)
    if msg is None:
        print("Timeout occurred, no message.")
    else:
        count = int(msg.data.hex(), 16)

    print(count)
