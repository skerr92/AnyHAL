# AnyHAL

AnyHAL is a Rust playground for developing portable hardware-abstraction-layer
contracts across embedded platforms. It separates application-facing APIs from
chip register implementations and physical board wiring, making it practical to
prototype a peripheral once and then implement it independently for each MCU
family.

The project supports fast host-side contract tests and `no_std` bare-metal
firmware built entirely from command-line tools.

## Project status

AnyHAL currently provides portable contracts for:

- Digital input, output, pulls, and alternate-function pin configuration
- Blocking delays
- Blocking I2C controller transactions
- Configurable blocking SPI bus transactions
- Polling, non-blocking serial consoles
- Host-side peripheral fakes and executable contract tests
- Bare-metal startup, linker layouts, and board-specific firmware examples
- UF2 artifact generation for boards whose catalog declares UF2 support

Current platform foundations cover Microchip SAMD51P19A, STM32H563, and the
STM32G431VBT6 variant. Platform completeness varies by peripheral; the Cargo
feature and resource catalogs are the source of truth for buildable targets.

## Architecture

```text
src/                         portable APIs, host fakes, and shared examples
platforms/<chip>/            self-contained chip crate, runtime, linker layout, HAL
boards/<board>/              self-contained board crate, wiring, and applications
resources/                   Make target catalogs and generic build orchestration
tools/                       artifact conversion and flashing helpers
```

Portable code belongs under `src/`. Code that accesses MCU registers belongs
under `platforms/<chip>/`. Physical pin assignments, bootloader reservations,
and board test applications belong under `boards/<board>/`.

Unsafe code is restricted to hardware ownership, startup, and register-access
boundaries. Portable contracts remain safe Rust.

## Prerequisites

- Stable Rust managed by `rustup`
- The Rust target required by the selected platform
- GNU Make for the optional catalog-driven Make interface
- A board-appropriate flashing tool or bootloader

Install the currently used Arm targets with:

```console
rustup target add thumbv7em-none-eabihf thumbv8m.main-none-eabihf
```

## Building

Cargo is authoritative. The root manifest describes only the portable AnyHAL
crate, host tests, and workspace membership. Every platform and board owns a
nested `Cargo.toml`; embedded builds select that manifest directly.

The root `Makefile` imports `resources/Makefile`, which discovers
`resources/boards` and every `resources/*_platform` catalog. This keeps the
top-level build interface stable as targets are added.

Useful commands:

```console
make list
make test
make build ANYHAL_TARGET=<catalog-id>
make build ANYHAL_TARGET=<catalog-id> ANYHAL_EXAMPLE=<cargo-example>
make fresh-build ANYHAL_TARGET=<catalog-id>
make uf2 UF2_TARGET=<catalog-id> ANYHAL_EXAMPLE=<cargo-example>
```

`fresh-build` and `fresh-uf2` run workspace-scoped `cargo clean` before
building. Only targets declaring `artifact := uf2` accept the `uf2` command.

The equivalent direct Cargo form is:

```console
cargo build --manifest-path <platform-or-board>/Cargo.toml --release --target <rust-target> --example <name>
```

## Portable API example

The same application logic can operate on host fakes or hardware-backed pins
and delays:

```rust
use anyhal::hal::{delay::DelayMs, gpio::OutputPin, Result};

fn pulse<P: OutputPin, D: DelayMs>(pin: &mut P, delay: &mut D) -> Result<()> {
    pin.set_high()?;
    delay.delay_ms(100)?;
    pin.set_low()?;
    Ok(())
}
```

Platform code supplies concrete implementations; portable code depends only on
the contracts.

## Adding a board

1. Ensure the MCU already has a self-contained platform crate under
   `platforms/<chip>/`. If it does not, add its `Cargo.toml`, runtime, memory
   layout, and required HAL implementations first.
2. Create `boards/<board>/Cargo.toml`. Depend on `anyhal` with default features
   disabled and on the matching platform crate by path.
3. Create `boards/<board>/mod.rs` containing typed aliases for the board's
   physical pins and onboard peripherals.
4. Add `boards/<board>/memory.x` when the board's usable memory differs from the
   raw chip, such as when preserving a resident bootloader.
5. Put board-owned applications under `boards/<board>/examples/` and register
   them in the board's manifest.
6. Add a board-local `build.rs` that copies its linker script to Cargo's output
   directory and passes `-Tmemory.x` to the linker.
7. Add the board manifest to the root workspace member list.
8. Add the board to `resources/boards` with its manifest, Rust target triple,
   default example, and optional artifact type:

   ```makefile
   ANYHAL_BOARDS += my-board
   anyhal.my-board.manifest := boards/my_board/Cargo.toml
   anyhal.my-board.triple := thumbv7em-none-eabihf
   anyhal.my-board.example := my_board_smoke
   anyhal.my-board.artifact := uf2
   ```

9. Run host tests, Clippy, the raw-platform build, the board's default build,
   and any artifact conversion before testing on hardware.

Adding another platform family follows the same catalog pattern in a new or
existing `resources/*_platform` file. No new top-level Make recipe is required.

## Design principles

- Keep contracts small, explicit, and useful across MCU families.
- Make hardware ownership visible at API boundaries.
- Keep board wiring separate from chip support.
- Use host fakes as executable specifications for portable behavior.
- Prefer timeout-bounded polling before introducing interrupts or async APIs.
- Preserve bootloaders and reserved memory through board-owned linker layouts.
- Keep command-line build instructions and target catalogs current.

## Roadmap

- PWM and general-purpose timer contracts
- Advanced motor-control timers for flight-control and ESC workloads
- ADC and DAC abstractions
- CAN and additional serial transports
- Interrupt-driven and asynchronous peripheral variants
- DMA-backed transfers
- Unified flashing/debug-probe workflows
- Continuous integration across host tests and every cataloged target
- Migration toward reusable crates if the single-package playground outgrows
  its current structure

## License

AnyHAL is licensed under the MIT License. See `LICENSE`.
