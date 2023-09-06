"""\
This script reads the accelerometer data from the CAN bus and prints it to the console.
It is intended to be used with the RS485-CAN HAT on a Raspberry Pi.
https://www.waveshare.com/rs485-can-hat.htm
"""

import os
import can


os.system("sudo ip link set can0 type can bitrate 100000")
os.system("sudo ifconfig can0 up")

can0 = can.interface.Bus(channel="can0", bustype="socketcan")  # socketcan_native


def translate_u8_to_xyz(rx_buf):
    def get_value(buf, idx1, idx2):
        tmp = (buf[idx1] << 8) | buf[idx2]
        sign_res = (tmp & 0xFFFF) if tmp & 0x8000 == 0 else tmp - 0x10000
        return sign_res >> 4

    x = get_value(rx_buf, 1, 0)
    y = get_value(rx_buf, 3, 2)
    z = get_value(rx_buf, 5, 4)

    return x, y, z


while True:
    # receive message
    msg = can0.recv(10.0)
    if msg is None:
        print("Timeout occurred, no message.")
    else:
        (x, y, z) = translate_u8_to_xyz(msg.data)
        print(f"x: {x}, y: {y}, z: {z}")


# os.system("sudo ifconfig can0 down")
