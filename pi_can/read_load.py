"""\
This script reads load cell data from the CAN bus and prints it to the console.
It is intended to be used with the RS485-CAN HAT on a Raspberry Pi.
https://www.waveshare.com/rs485-can-hat.htm
"""

import os
import can

os.system("sudo ip link set can0 type can bitrate 100000")
os.system("sudo ifconfig can0 up")

can0 = can.interface.Bus(channel="can0", bustype="socketcan")  # socketcan_native


def u8_array_to_u16(data_array):
    return (data_array[0] << 8) | data_array[1]


def u8_array_to_u16(data_array):
    return (data_array[0] << 8) | data_array[1]


def unpack_bridge_config(bridge_config):
    a2d_offset = bridge_config & 0x000F
    preamp_gain = (bridge_config >> 4) & 0x0007
    gain_polarity = (bridge_config >> 7) & 0x0001 == 1
    long_int = (bridge_config >> 8) & 0x0001 == 1
    bsink = (bridge_config >> 9) & 0x0001 == 1
    preamp_mux = (bridge_config >> 10) & 0x0003
    disable_nulling = (bridge_config >> 12) & 0x0001 == 1
    idt_reserved = (bridge_config >> 13) & 0x0007

    return (
        a2d_offset,
        preamp_gain,
        gain_polarity,
        long_int,
        bsink,
        preamp_mux,
        disable_nulling,
        idt_reserved,
    )


full_scale_load = 50  # kg
preamp_gain = 192  # amplifier gain
adc_bits = 14
load_cell_sensitivity = 1.3e-3  # 1.3 mV/V

load_per_bit = full_scale_load / (2**adc_bits * preamp_gain * load_cell_sensitivity)
zero_offset_bits = -(2 ** (adc_bits - 1)) + 82

# filter
load = 0
update = 0.2

while True:
    # receive message
    msg = can0.recv(10.0)
    if msg is None:
        print("Timeout occurred, no message.")
    else:
        if len(msg.data) == 3:
            # config
            print("-------")
            print(
                hex(msg.data[0])
            )  # when successful, will be 0x5a (requires power cycle)
            config = unpack_bridge_config(u8_array_to_u16(msg.data[1:]))
            print(config)
            print("######")
        elif len(msg.data) == 2:
            # only load cell data
            print(u8_array_to_u16(msg.data))
        elif len(msg.data) == 5:
            # load cell data with status (normal 0, command 1)
            raw_load_bits = u8_array_to_u16(msg.data[1:3])
            new_load = (raw_load_bits + zero_offset_bits) * load_per_bit
            load = load * (1 - update) + new_load * update
            print(f"status: {msg.data[0]}, data: {load:+.3f}")
        else:
            print(msg.data)


# os.system("sudo ifconfig can0 down")
