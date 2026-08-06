# CondensedContext

## Context Freshness
Context ID: FRESH-001
Last Verified Commit: 79501ed5fa8b99b28059ac199c2e0ba0f7a4c84d
Current HEAD: 79501ed5fa8b99b28059ac199c2e0ba0f7a4c84d
Generated: 2026-08-05
Status:
- Partial; the validated Rust foundation is uncommitted in the working tree.
Files requiring verification:
- Rust foundation files until committed.

## Purpose
Durable, compact memory for agent work in this repository. Keep this file short, current, and useful for future agents.

## Current Focus
Context ID: ACTIVE-001
Confidence: High; validated foundation and stated goal.
- Select the next flight-control-oriented peripheral after hardware-validating TC5 pulse capture.
- Keep consuming package-filtered SAMD51 capability data as additional peripheral instances are implemented.

## Handoff
Context ID: HANDOFF-001
Confidence: High; builds validated locally.
- Last known state: PWM, corrected ADC/USB startup, DAC1/A2, and TC5 pulse capture are hardware-validated.
- Next useful step: Select the next flight-control-oriented peripheral, likely interrupts, DMA, CAN, or advanced ADC acquisition.
- Validation: A1-to-A3 loopback reports 20000 us/37500 ticks period and alternating 999/1999 us pulse widths; the first report after a command transition may contain the prior completed cycle.

## Stable Facts
Context ID: FACTS-001
Confidence: High.
- Project name is AnyHAL.
- Initial hardware target is Microchip ATSAMD51P19A.
- The tested Feather SAMD51 board uses the ATSAMD51P19A 120-ball TFBGA package (`TFBGA120`, Microchip pinout `SAMD51PC`).
- The project is intended as a playground for portable embedded HAL development.
- The SAMD51P19A foundation uses the `thumbv7em-none-eabihf` Rust target and a 512 KiB flash/192 KiB RAM linker map.
- Board LED/D13 is PB00; all user-supplied Feather aliases are encoded in `boards/feather_samd51p19/mod.rs`.
- Hardware-validated SPI header aliases are MOSI=PB12/SERCOM4 PAD0, SCK=PB13/PAD1, and MISO=PB14/PAD2.
- SAMD51 native USB uses PA24/PA25 and CDC-ACM; its initializer configures a 120 MHz core plus dedicated 48 MHz USB clock before other peripherals.
- SAMD51 SPI uses SERCOM4 at a configurable clock/mode/order; board mapping is MOSI PB12/pad0, SCK PB13/pad1, MISO PB14/pad2.
- SAMD51 PWM proof uses TC4 match-PWM output WO1 on A1/PB09 (peripheral function E); PB00/onboard LED has no timer waveform route.
- SAMD51 ADC proof uses calibrated 12-bit ADC0/AIN2 on A0/PB08 (peripheral function B), VDDANA reference, and GCLK2 at 48 MHz with a /32 ADC prescaler.
- Corrected ADC0/A0 sampling and USB enumeration are hardware-validated as of 2026-08-05.
- SAMD51 DAC proof uses 12-bit channel 1/VOUT1 on A2/PA05 (peripheral function B), VDDANA reference, and GCLK2 at 48 MHz.
- DAC1/A2 triangle output is hardware-validated as of 2026-08-05.
- SAMD51 pulse capture uses PA06/EIC EXTINT6 -> EVSYS channel 0 -> TC5 PPW; A1/PB09 TC4 supplies the 50 Hz loopback signal.
- SAMD51 pulse capture is hardware-validated at the USB-configured 120 MHz core clock: TC5 /64 is 1.875 MHz, 20 ms is 37500 ticks, and 1/2 ms are approximately 1874/3749 ticks.
- TC PPW requires both `CTRLA.CAPTEN0` and `CAPTEN1`; the first hardware test exposed their omission before capture validation.
- TC capture flags must be consumed independently; leaving one MCx flag set prevents hardware from updating that channel and raises capture overflow.
- EIC-to-TC PPW routing must use EIC high-level sensing; EIC both-edge sensing produces narrow event pulses and yields a zero captured high time.
- SEGGER J-Link V9.24a is installed at `C:\Program Files\SEGGER\JLink_V924a\JLink.exe`.
- NUCLEO-H563ZI uses STM32H563ZI (Cortex-M33, 2 MiB flash, 640 KiB total SRAM); initial linker uses 256 KiB SRAM1 and LD1 is active-high PB0.
- ESC MCU target is exactly STM32G431VBT6: Cortex-M4F, LQFP100, 128 KiB flash, 32 KiB SRAM.

## Decisions
Context ID: DECISIONS-001
Confidence: High.
- Command-line build instructions must remain current in README.md.
- CondensedContext.md is the durable project-memory file.
- Rust and Cargo are authoritative; the root manifest defines the portable `anyhal` package and workspace, while nested board/platform manifests define embedded builds.
- Keep board wiring separate from MCU support and isolate unsafe code at startup/register boundaries.
- UF2 board applications start at `0x4000`, preserving the resident 16 KiB bootloader, and use SAMD51 UF2 family ID `0x55114460`.
- Portable GPIO/delay traits are safe; unsafe hardware ownership is explicit through `claim` constructors.
- Portable GPIO input supports floating, pull-up, and pull-down configuration; the first hardware test uses A0/PB08 pulled up and active low.
- Portable mux configuration carries AF number, pull, output type, and speed; STM32 supports AF0-15 while SAMD maps 0-13 to A-N and rejects native open-drain.
- H563 bring-up uses reset HSI64; maximum-frequency PLL configuration waits for hardware validation.
- Main clock setup temporarily selects OSCULP32K, re-enables open-loop DFLL48M with the Microchip erratum workaround, then selects undivided DFLL48M on GCLK0 at nominal 48 MHz.
- The direct SAMD dependency aliases the current package-wide `atsamd51p` PAC so AnyHAL and `atsamd-hal` share one peripheral definition and singleton.
- Package naming convention is `anyhal-<chip>` for platforms and `anyhal-board-<board>` for boards; board packages depend on their platform package by path.
- Portable implementation stays under `src`; each platform/board owns its Rust modules, linker script, and target-specific examples.
- Make target discovery is declarative: `resources/boards` and `resources/*_platform` define manifest/triple/example/artifact metadata consumed by `resources/Makefile`.
- Keep README product-level and board-neutral; detailed hardware bring-up procedures belong with board sources and durable project context.
- Portable PWM duty uses the normalized full `u16` range so applications do not depend on a timer's native counter resolution.
- Portable one-shot ADC exposes resolution/reference metadata and reference-aware millivolt conversion without assuming every backend is 12-bit or 3.3 V.
- Portable DAC exposes resolution/reference metadata, raw code writes, and clamped millivolt conversion.
- Portable pulse capture is non-blocking and reports period/high time in native ticks plus tick-frequency conversion metadata.
- SAMD51 platform drivers resolve caller-supplied pins through centralized chip route metadata; inline board-specific pin equality checks are prohibited.
- MCU platform builds must select exactly one supplier package; SAMD51P19A currently supports `package-tfbga120` and `package-tqfp128`, with bonded-pin capabilities generated from Microchip SAMD51_DFP 3.9.276 ATDF data.

## Known Constraints
Context ID: CONSTRAINTS-001
Confidence: High.
- Preserve pre-existing uncommitted user work.
- Embedded builds select the owning nested `Cargo.toml` with `--manifest-path`; platform and board crates are always `no_std`.
- H563/Nucleo use `thumbv8m.main-none-eabihf`; SAMD51 and STM32G431VBT6 use `thumbv7em-none-eabihf`.
- No STM32CubeProgrammer, OpenOCD, or probe-rs CLI was detected; STM32 flashing is not yet scripted.

## File Map
Context ID: FILEMAP-001
Confidence: High; verified against the working tree.
- `README.md`: board-neutral project overview, supported features, portable API example, build guide, add-board workflow, and roadmap.
- `Cargo.toml`: portable root package, workspace membership, host tools/tests, and shared release profile.
- `src/lib.rs`: portable API root and feature-to-source routing.
- `src/hal/`: device-independent contracts, pin identities, errors, and results.
- `src/hal/i2c.rs`: portable blocking seven-bit I2C contract.
- `src/hal/serial.rs`: portable polling, non-blocking serial-console contract.
- `src/hal/spi.rs`: portable blocking SPI configuration and bus contract.
- `src/hal/pwm.rs`: normalized single-channel PWM output contract.
- `src/hal/adc.rs`: blocking one-shot ADC contract and millivolt conversion.
- `src/hal/dac.rs`: reference-aware single-channel analog-output contract.
- `src/hal/capture.rs`: non-blocking period/high-pulse-width capture contract.
- `src/testing/`: host fakes and executable contract tests.
- `src/examples/`: device-independent examples.
- `platforms/*/Cargo.toml`, `boards/*/Cargo.toml`: target-local dependencies and examples; adjacent `build.rs` files install each target's linker script.
- `platforms/samd51p19a/`: chip clock, delay, GPIO, runtime, raw memory layout, and chip smoke example.
- `platforms/samd51p19a/routes.rs`: route descriptor types plus build-generated, package-filtered capability lookups used by platform drivers.
- `platforms/samd51p19a/build.rs`: enforces one package feature and generates Rust route lookups from the checked-in capability catalog.
- `resources/devices/microchip/atsamd51p19a-routes.csv`: TFBGA120/TQFP128 bonded-pin and peripheral mux data derived from Microchip SAMD51_DFP 3.9.276.
- `tools/extract-microchip-atdf-routes.ps1`: reproducible ATDF-to-route-catalog extractor.
- `platforms/samd51p19a/i2c.rs`: timeout-bounded SERCOM1 I2C controller on PA16/PA17.
- `platforms/samd51p19a/usb_serial.rs`: native USB CDC-ACM console using PA24/PA25.
- `platforms/samd51p19a/spi.rs`: timeout-bounded SERCOM4 controller on PB12/PB13/PB14.
- `platforms/samd51p19a/pwm.rs`: configurable TC4 match-PWM backend validating a supplied TC4 WO1-capable pin.
- `platforms/samd51p19a/adc.rs`: calibrated ADC0 one-shot backend validating a supplied ADC0 pin.
- `platforms/samd51p19a/dac.rs`: synchronized DAC1 backend validating a supplied DAC VOUT1 pin.
- `platforms/samd51p19a/capture.rs`: EIC/EVSYS/TC5 PPW backend dynamically routing a supplied EIC-capable pin.
- `platforms/stm32h563/`: M33 runtime, reset-clock delay, GPIO/AF, memory, and smoke example.
- `platforms/stm32g431vbt6/`: exact M4F ESC MCU runtime, GPIO/AF, 128K/32K memory, and smoke example.
- `rust-toolchain.toml`: stable toolchain, Cortex-M target, rustfmt, and Clippy.
- `boards/feather_samd51p19/`: pin aliases, bootloader-preserving memory layout, and board blink example.
- `boards/nucleo_h563zi/`: LD1/LD2/LD3, user button, VCP aliases, linker profile, and LD1 blink.
- `boards/feather_samd51p19/examples/gpio_input.rs`: A0-to-GND input proof driving PB00 LED.
- `boards/feather_samd51p19/examples/pwm_sweep.rs`: USB-reported 20 kHz A1/PB09 duty sweep.
- `boards/feather_samd51p19/examples/adc_monitor.rs`: USB-reported averaged A0/PB08 analog monitor.
- `boards/feather_samd51p19/examples/dac_wave.rs`: USB-reported stepped triangle on A2/PA05.
- `boards/feather_samd51p19/examples/pulse_capture.rs`: A1-to-A3 50 Hz servo-pulse loopback with USB measurements.
- `tools/anyhal-uf2.rs`: dependency-free ELF32-to-aligned-SAMD51-UF2 converter.
- `tools/jlink-flash.jlink`: non-chip-erase SEGGER Commander flashing recipe.
- `CMakeLists.txt` and `Makefile`: migration/delegation shims; Cargo remains authoritative.
- `resources/Makefile`: generic target validation, build, clean, UF2, listing, and compatibility recipes.
- `resources/boards`, `resources/*_platform`: discovered build-target catalogs.

## Validation Memory
Context ID: VALIDATION-001
Confidence: High; full clean matrix passed on 2026-08-04.
- Host: `cargo test` and `cargo run --example host_smoke`.
- Lint: `cargo clippy --all-targets -- -D warnings`.
- Embedded: build with `cargo build --manifest-path platforms/<chip>/Cargo.toml --release --target <triple> --examples`.
- Board: build with `cargo build --manifest-path boards/<board>/Cargo.toml --release --target <triple> --examples`; then run `anyhal-uf2` on UF2-capable output.
- GPIO input: build/convert `feather_gpio_input`; ground A0/PB08 to light the LED.
- Nucleo: manifest `boards/nucleo_h563zi/Cargo.toml`, target `thumbv8m.main-none-eabihf`, example `nucleo_h563zi_blink`.
- ESC MCU: manifest `platforms/stm32g431vbt6/Cargo.toml`, target `thumbv7em-none-eabihf`, example `stm32g431vbt6_smoke`.
- USB console: build `feather_usb_console`, convert it to UF2, then open the enumerated CDC COM port to validate report/heartbeat/echo.
- Generic Make: `make list`; `make build ANYHAL_TARGET=<catalog-id> [ANYHAL_EXAMPLE=<example>]`; UF2-capable boards use `make uf2 UF2_TARGET=<id>`.
- PWM: build/convert `feather_pwm_sweep`; serial reports TC4 setup while A1/PB09 sweeps from 0-100% at 20 kHz.
- ADC: build/convert `feather_adc_monitor`; serial reports averaged raw counts and millivolts for A0/PB08 every 250 ms.
- DAC: build/convert `feather_dac_wave`; serial reports setup while A2/PA05 generates a stepped 0-3.3 V triangle.
- Capture: build/convert `feather_pulse_capture`; jumper A1 to A3 and expect period near 20000 us and high time alternating near 1000/2000 us.
- Package routes: clean TQFP128 raw-platform smoke build and clean TFBGA120 Feather all-example build both pass; embedded Clippy passes with warnings denied.

## Coverage
Context ID: COVERAGE-001
Confidence: High.
- Files indexed: All meaningful Rust, build, platform, example, and documentation files.
- Coverage notes: Generated `target/` contents are intentionally excluded.

## Changes
| Date | Tags | Change | Files | Commit | Remote |
| --- | --- | --- | --- | --- | --- |
| 2026-08-03 | [context] | Initialized durable repository context. | `CondensedContext.md` | Uncommitted | Not confirmed |
| 2026-08-03 | [rust] [build] [samd51] [docs] | Replaced the C direction with a validated host/no_std Rust foundation and documented workflow. | `Cargo.toml`, `src/lib.rs`, `build.rs`, `examples/`, `platforms/`, `README.md` | Uncommitted | Not confirmed |
| 2026-08-03 | [board] [gpio] [uf2] [jlink] | Added full board pin map, PB00 blink, bootloader-safe layout, aligned UF2 generation, and SEGGER workflow. | `src/board.rs`, `boards/`, `examples/feather_blink.rs`, `tools/`, `README.md` | Uncommitted | Not confirmed |
| 2026-08-03 | [gpio] [timer] [tests] | Replaced direct blink registers and nop delay with portable traits, host fakes, SAMD PORT output, and SysTick 500 ms timing. | `src/gpio.rs`, `src/delay.rs`, `src/host.rs`, `src/samd51p19a.rs`, `examples/feather_blink.rs` | Uncommitted | Not confirmed |
| 2026-08-03 | [clock] [samd51] | Added boot-path-independent 48 MHz DFLL/GCLK initialization using PAC register fields and wired SysTick to the returned clock token. | `Cargo.toml`, `src/samd51p19a.rs`, `examples/feather_blink.rs`, `README.md` | Uncommitted | Not confirmed |
| 2026-08-03 | [architecture] [build] [docs] | Layered portable/chip/board sources, moved owned examples and runtime, and renamed target features to explicit platform/board flags. | `src/`, `platforms/`, `boards/`, `Cargo.toml`, `Makefile`, `README.md` | Uncommitted | Not confirmed |
| 2026-08-03 | [gpio] [input] [tests] | Added portable input/pull contracts, host fake coverage, SAMD51 PORT input, and an A0-to-LED Feather test UF2. | `src/hal/gpio.rs`, `src/testing/mod.rs`, `platforms/samd51p19a/gpio.rs`, `boards/feather_samd51p19/examples/gpio_input.rs` | Uncommitted | Not confirmed |
| 2026-08-03 | [stm32] [platform] [board] [gpio] | Added STM32H563, NUCLEO-H563ZI, exact STM32G431VBT6, and cross-platform alternate-function GPIO with Feather mux proof. | `Cargo.toml`, `src/hal/gpio.rs`, `platforms/`, `boards/`, `README.md` | Uncommitted | Not confirmed |
| 2026-08-03 | [i2c] [samd51] [tests] [docs] | Added portable blocking I2C, host fake, SAMD51 SERCOM1 controller, and Feather scanner/UF2. | `src/hal/i2c.rs`, `src/testing/mod.rs`, `platforms/samd51p19a/i2c.rs`, `boards/feather_samd51p19/examples/i2c_scan.rs`, `README.md` | Uncommitted | Not confirmed |
| 2026-08-03 | [spi] [samd51] [board-test] | Added debounced PB12/PB13/PB14 LED-signature firmware to identify the physical SPI header wiring. | `boards/feather_samd51p19/examples/spi_pin_id.rs`, `Cargo.toml`, `README.md` | Uncommitted | Not confirmed |
| 2026-08-03 | [spi] [board] [pin-map] | Applied hardware-validated SERCOM4 roles: MOSI PB12/pad0, SCK PB13/pad1, MISO PB14/pad2. | `boards/feather_samd51p19/mod.rs`, `boards/feather_samd51p19/examples/gpio_mux.rs`, `README.md` | Uncommitted | Not confirmed |
| 2026-08-03 | [usb] [serial] [samd51] [tests] | Added portable serial console, host fake, native USB CDC-ACM backend, console/echo self-test UF2, and unified the SAMD PAC generation. | `Cargo.toml`, `src/hal/serial.rs`, `src/testing/mod.rs`, `platforms/samd51p19a/`, `boards/feather_samd51p19/examples/usb_console.rs`, `README.md` | Uncommitted | Not confirmed |
| 2026-08-03 | [spi] [samd51] [usb] [tests] | Added portable blocking SPI, host fake, SERCOM4 controller, USB-reported loopback UF2, and explicit clean-build target. | `src/hal/spi.rs`, `src/testing/mod.rs`, `platforms/samd51p19a/spi.rs`, `boards/feather_samd51p19/examples/spi_loopback.rs`, `Makefile`, `README.md` | Uncommitted | Not confirmed |
| 2026-08-04 | [build] [make] [architecture] [docs] | Replaced target-specific top-level recipes with wildcard-discovered board/platform catalogs and generic build/UF2 orchestration. | `Makefile`, `resources/Makefile`, `resources/boards`, `resources/samd_platform`, `resources/stm32_platform`, `README.md` | Uncommitted | Not confirmed |
| 2026-08-04 | [docs] [readme] | Recast README as a board-neutral AnyHAL overview with current capabilities, one portable API example, catalog-driven commands, an add-board checklist, and roadmap. | `README.md`, `CondensedContext.md` | Uncommitted | Not confirmed |
| 2026-08-04 | [cargo] [workspace] [architecture] | Split every board/platform into a nested Cargo package with local dependencies, examples, and linker setup; catalogs now build selected manifests directly. | `Cargo.toml`, `src/lib.rs`, `platforms/`, `boards/`, `resources/`, `README.md` | Uncommitted | Not confirmed |
| 2026-08-04 | [pwm] [samd51] [tests] | Added normalized portable PWM, host fake coverage, TC4/PB09 match-PWM, and a USB-reported duty-sweep UF2. | `src/hal/pwm.rs`, `src/testing/mod.rs`, `platforms/samd51p19a/pwm.rs`, `boards/feather_samd51p19/examples/pwm_sweep.rs`, `README.md` | Uncommitted | Not confirmed |
| 2026-08-05 | [adc] [samd51] [tests] | Added reference-aware one-shot ADC, host conversion coverage, calibrated ADC0/PB08 sampling, and a USB-reported analog-monitor UF2. | `src/hal/adc.rs`, `src/testing/mod.rs`, `platforms/samd51p19a/adc.rs`, `boards/feather_samd51p19/examples/adc_monitor.rs`, `README.md` | Uncommitted | Not confirmed |
| 2026-08-05 | [adc] [usb] [fix] | Corrected per-register ADC synchronization and made the monitor enumerate/poll USB before setup with a console-visible timeout path. | `platforms/samd51p19a/adc.rs`, `boards/feather_samd51p19/examples/adc_monitor.rs`, `CondensedContext.md` | Uncommitted | Not confirmed |
| 2026-08-05 | [dac] [samd51] [tests] | Added reference-aware DAC output, host conversion coverage, synchronized DAC1/PA05 output, and a USB-safe triangle-wave UF2. | `src/hal/dac.rs`, `src/testing/mod.rs`, `platforms/samd51p19a/dac.rs`, `boards/feather_samd51p19/examples/dac_wave.rs`, `README.md` | Uncommitted | Not confirmed |
| 2026-08-05 | [capture] [samd51] [evsys] [tests] | Added and hardware-validated portable pulse capture, host coverage, PA06/EIC-to-EVSYS-to-TC5 PPW capture, and a USB-reported 50 Hz loopback UF2. | `src/hal/capture.rs`, `src/testing/mod.rs`, `platforms/samd51p19a/capture.rs`, `boards/feather_samd51p19/examples/pulse_capture.rs`, `README.md` | Uncommitted | Not confirmed |
| 2026-08-05 | [capture] [samd51] [fix] | Enabled TC5 capture channels 0 and 1 required by PPW mode after hardware showed no latched measurements. | `platforms/samd51p19a/capture.rs`, `CondensedContext.md` | Uncommitted | Not confirmed |
| 2026-08-05 | [capture] [samd51] [fix] | Fixed stale CC1 measurements by consuming each MCx flag independently, clearing overflow state, and adding raw tick diagnostics. | `platforms/samd51p19a/capture.rs`, `boards/feather_samd51p19/examples/pulse_capture.rs`, `CondensedContext.md` | Uncommitted | Not confirmed |
| 2026-08-05 | [capture] [samd51] [fix] | Changed the EIC generator to high-level sensing so PPW receives the input level rather than zero-width both-edge event pulses. | `platforms/samd51p19a/capture.rs`, `CondensedContext.md` | Uncommitted | Not confirmed |
| 2026-08-05 | [architecture] [samd51] [pinmux] | Centralized current ADC, DAC, PWM, capture, I2C, and SPI pin capabilities into route descriptors; drivers now validate supplied pins by function/pad/channel rather than board-specific equality checks. | `platforms/samd51p19a/routes.rs`, SAMD51 peripheral backends, `README.md`, `CondensedContext.md` | Uncommitted | Not confirmed |
| 2026-08-05 | [package] [samd51] [pinmux] [build] | Declared the Feather's 120-TFBGA package, added official ATDF-derived TFBGA120/TQFP128 capability data, generated package-filtered routes at build time, and rejected missing or conflicting package selections. | `resources/devices/microchip/`, `tools/extract-microchip-atdf-routes.ps1`, `platforms/samd51p19a/`, board manifest, build catalogs, docs | Uncommitted | Not confirmed |

## Archived History
- None yet.

## Open Threads
Context ID: OPEN-001
Confidence: High.
- Decide how platform ownership should evolve if the single package eventually becomes a Cargo workspace with separate reusable crates.
- Confirm that this board's resident UF2 bootloader uses the standard SAMD51 16 KiB reservation if hardware behavior differs.
- Validate NUCLEO-H563ZI runtime/GPIO on arrival and choose/install its command-line ST-LINK flashing tool.
- Record the ESC board signal-to-pin map when the STM32G431VBT6 schematic is stable.
- Hardware-test SERCOM1 I2C with a known target and external pull-ups.
- Confirm USB CDC enumeration, DTR-gated report, polling heartbeat, and echo on hardware.
- Confirm the 1 MHz SERCOM4 electrical loopback report with PB12 jumpered to PB14.
- Execute `make list` and representative generic/dry-run targets after GNU Make is installed or from another development environment.
