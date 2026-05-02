# Pamplemouche-OS build system
#
# Targets:
#   make build     – cross-compile the kernel (default)
#   make run       – build and launch in QEMU
#   make test      – run unit tests for the compat crate (host)
#   make clean     – remove build artefacts

CARGO        ?= cargo
QEMU         ?= qemu-system-x86_64
KERNEL_BIN    = target/x86_64-pamplemouche/debug/pamplemouche-kernel
BOOTIMAGE     = target/x86_64-pamplemouche/debug/bootimage-pamplemouche-kernel.bin

.PHONY: all build run test clean

all: build

## ── Build ──────────────────────────────────────────────────────────────────

build:
	@echo "==> Building kernel (x86_64-pamplemouche)"
	cd kernel && $(CARGO) build -Z build-std=core,compiler_builtins,alloc \
	    -Z build-std-features=compiler-builtins-mem
	@echo "==> Building compat library (host)"
	$(CARGO) build -p pamplemouche-compat

## ── Run in QEMU ─────────────────────────────────────────────────────────────

run: build
	@echo "==> Creating bootable image"
	cd kernel && $(CARGO) bootimage
	@echo "==> Launching QEMU"
	$(QEMU) \
	    -drive format=raw,file=$(BOOTIMAGE) \
	    -serial stdio \
	    -display none \
	    -m 256M \
	    -no-reboot

## ── Tests ────────────────────────────────────────────────────────────────────

test:
	@echo "==> Running compat unit tests (host)"
	$(CARGO) test -p pamplemouche-compat

## ── Clean ────────────────────────────────────────────────────────────────────

clean:
	$(CARGO) clean
