## Run
The reset button on the board must be pressed before flashing.
``` console
cargo flash --bin can --release --chip STM32f413RHTx
```

Which has been aliased to:
``` console
cargo fl
```

### Places where the chip name needs to be updated:
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