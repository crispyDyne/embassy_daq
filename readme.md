## Embassy
- [Embassy getting started](https://embassy.dev/book/dev/getting_started.html)
    - See the "getting started" section here to set up rust, and install probe-rs.


## Required Hardware:
- Raspberry Pi
    - CAN Hat ([amazon](https://www.amazon.com/RS485-CAN-HAT-Long-Distance-Communication/dp/B07VMB1ZKH), [direct](https://www.waveshare.com/rs485-can-hat.htm))
- Mikroe click
    - [Uni-Clicker](https://www.mikroe.com/uni-clicker)
    - [STM32F413RHTx MCU card](https://www.mikroe.com/mcu-card-29-for-stm32-stm32f413rh)
- Click Modules
    - [9DOF 3 Click](https://www.mikroe.com/9dof-3-click)
    - [Load Cell 4 Click](https://www.mikroe.com/load-cell-4-click)
- ST-Link/V2
    - [Amazon Knock-Off](https://www.amazon.com/s?k=st+link+v2) (this is what I use)
    - [Official hardware from STM](https://www.st.com/en/development-tools/st-link-v2.html)


## Raspbery Pi Setup
See the [raspberry-pi.md](./raspberry-pi.md) file for instructions on setting up the raspberry pi.

## Run can demo
The reset button on the board must be pressed before flashing.
``` console
cargo flash --bin can --release --chip STM32f413RHTx
```

Which has been aliased to:
``` console
cargo fl_can
```

The python script `echo_can.py` should be run on the connected raspberry pi. 

## Run accel demo
The reset button on the board must be pressed before flashing.
``` console
cargo flash --bin accel --release --chip STM32f413RHTx
```

Which has been aliased to:
``` console
cargo fl_can
```

The python script `read_accel.py` should be run on the connected raspberry pi.

## Places where the chip name needs to be updated:
Cargo.toml
``` toml 
[dependencies]
embassy-stm32 = { features = [ "stm32f413rh",]  }
```

.cargo/config.toml
``` toml
[target.'cfg(all(target_arch = "arm", target_os = "none"))']
runner = "probe-run --chip STM32F413RHTx" # this does not seem to do anything
```