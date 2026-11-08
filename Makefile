CARGO ?= cargo
OPENOCD ?= openocd
ELF = target/thumbv7em-none-eabihf/release/magnetometer-compass

.PHONY: all build run flash clean

all: build

build:
	$(CARGO) build --release

run flash: build
	$(OPENOCD) -f stm32f3.cfg -c "program $(ELF) verify reset exit"

clean:
	$(CARGO) clean
