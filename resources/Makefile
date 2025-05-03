# Generic AnyHAL command-line build orchestration.

ANYHAL_RESOURCE_DIR := $(patsubst %/,%,$(dir $(lastword $(MAKEFILE_LIST))))
ANYHAL_CATALOGS := $(wildcard $(ANYHAL_RESOURCE_DIR)/boards $(ANYHAL_RESOURCE_DIR)/*_platform)
include $(ANYHAL_CATALOGS)

ANYHAL_TARGETS := host $(ANYHAL_BOARDS) $(ANYHAL_PLATFORMS)
ANYHAL_TARGET ?= host
ANYHAL_EXAMPLE ?=

anyhal.host.manifest := Cargo.toml
anyhal.host.example := host_smoke

ANYHAL_MANIFEST = $(anyhal.$(ANYHAL_TARGET).manifest)
ANYHAL_TRIPLE = $(anyhal.$(ANYHAL_TARGET).triple)
ANYHAL_SELECTED_EXAMPLE = $(if $(ANYHAL_EXAMPLE),$(ANYHAL_EXAMPLE),$(anyhal.$(ANYHAL_TARGET).example))
ANYHAL_BUILD_ARGS = --manifest-path $(ANYHAL_MANIFEST) $(if $(ANYHAL_TRIPLE),--release --target $(ANYHAL_TRIPLE),)
ANYHAL_EXAMPLE_ARG = $(if $(ANYHAL_SELECTED_EXAMPLE),--example $(ANYHAL_SELECTED_EXAMPLE),)

UF2_TARGET ?= $(if $(filter-out host,$(ANYHAL_TARGET)),$(ANYHAL_TARGET),feather-samd51p19)
BOARD_EXAMPLE ?= $(anyhal.$(UF2_TARGET).example)
UF2_EXAMPLE = $(if $(ANYHAL_EXAMPLE),$(ANYHAL_EXAMPLE),$(BOARD_EXAMPLE))

.PHONY: all list build test clean fresh-build uf2 fresh-uf2 check-target check-uf2
.PHONY: platform board nucleo-h563zi stm32g431vbt6

all:
	make build ANYHAL_TARGET=host

list:
	@echo Boards: $(ANYHAL_BOARDS)
	@echo Platforms: $(ANYHAL_PLATFORMS)
	@echo Host: host

check-target:
	$(if $(filter $(ANYHAL_TARGET),$(ANYHAL_TARGETS)),,$(error Unknown ANYHAL_TARGET '$(ANYHAL_TARGET)'. Run 'make list'))
	$(if $(ANYHAL_MANIFEST),,$(error Target '$(ANYHAL_TARGET)' has no Cargo manifest))
	$(if $(ANYHAL_SELECTED_EXAMPLE),,$(error Target '$(ANYHAL_TARGET)' needs ANYHAL_EXAMPLE=<Cargo example>))

build: check-target
	cargo build $(ANYHAL_BUILD_ARGS) $(ANYHAL_EXAMPLE_ARG)

fresh-build:
	cargo clean
	make build ANYHAL_TARGET=$(ANYHAL_TARGET) ANYHAL_EXAMPLE=$(ANYHAL_SELECTED_EXAMPLE)

test:
	cargo test

clean:
	cargo clean

check-uf2:
	$(if $(filter $(UF2_TARGET),$(ANYHAL_TARGETS)),,$(error Unknown UF2_TARGET '$(UF2_TARGET)'. Run 'make list'))
	$(if $(filter uf2,$(anyhal.$(UF2_TARGET).artifact)),,$(error Target '$(UF2_TARGET)' does not define UF2 output))

uf2: check-uf2
	make build ANYHAL_TARGET=$(UF2_TARGET) ANYHAL_EXAMPLE=$(UF2_EXAMPLE)
	cargo run --manifest-path Cargo.toml --bin anyhal-uf2 -- target/$(anyhal.$(UF2_TARGET).triple)/release/examples/$(UF2_EXAMPLE) target/$(anyhal.$(UF2_TARGET).triple)/release/examples/$(UF2_EXAMPLE).uf2

fresh-uf2:
	cargo clean
	make uf2 UF2_TARGET=$(UF2_TARGET) ANYHAL_EXAMPLE=$(UF2_EXAMPLE)

# Compatibility aliases; new targets belong in catalogs, not as new recipes.
platform:
	make build ANYHAL_TARGET=samd51p19a

board:
	make build ANYHAL_TARGET=feather-samd51p19 ANYHAL_EXAMPLE=$(BOARD_EXAMPLE)

nucleo-h563zi:
	make build ANYHAL_TARGET=nucleo-h563zi

stm32g431vbt6:
	make build ANYHAL_TARGET=stm32g431vbt6
