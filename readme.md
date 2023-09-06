## Python CAN setup
https://www.waveshare.com/wiki/RS485_CAN_HAT

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
[dependencies]`
embassy-stm32 = { ... features = [... "stm32f413rh",...]  }
```

.cargo/config.toml
``` toml
[target.'cfg(all(target_arch = "arm", target_os = "none"))']
runner = "probe-run --chip STM32F413RHTx" # this does not seem to do anything
```