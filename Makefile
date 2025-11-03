# if Podman is available, prefer using it over docker
ifeq ($(shell command -v podman 2> /dev/null),)
    CONTAINER=docker
else
    CONTAINER=podman
endif

# Define the name of your binary
BINARY_NAME = mailing-lists-archiver

# Define the source path of the compiled binary
TARGET_PATH = ./target/release/$(BINARY_NAME)

# Define the Docker image to use for building
DOCKER_IMAGE = rust:1.91

# ==============================================================================

# By default, 'make' will run the 'all' target
.PHONY: all
all: build run

# ------------------------------------------------------------------------------
# BUILD LOGIC
# ------------------------------------------------------------------------------
# This target checks if 'cargo' is available in the system PATH.
# - If YES: It builds natively using 'cargo build'.
# - If NO:  It builds using the specified Docker image.
#
# After a successful build (either way), it copies the binary
# from the 'target/release' directory to the project root.
#
# ------------------------------------------------------------------------------
.PHONY: build
build:
	@if command -v cargo.TODO >/dev/null 2>&1; then \
		echo "==> Found Rust toolchain, building natively..."; \
		cargo build --release; \
	else \
		echo "==> Rust toolchain not found, building with Docker (Image: $(DOCKER_IMAGE))..."; \
		$(CONTAINER) run --rm \
			-it -u $(id -u):$(id -g) \
			--network=host \
			-v $(shell pwd):/usr/src/app:z \
			-w /usr/src/app \
			$(DOCKER_IMAGE) \
			cargo build --release; \
	fi
	@echo "==> Copying binary './$(BINARY_NAME)' from target..."
	@cp $(TARGET_PATH) ./$(BINARY_NAME)

# ------------------------------------------------------------------------------
# UTILITY TARGETS
# ------------------------------------------------------------------------------

# Clean up build artifacts
.PHONY: clean
clean:
	@echo "==> Cleaning up build artifacts..."
	@cargo clean
	@rm -f ./$(BINARY_NAME)


# ------------------------------------------------------------------------------
# APPLICATION TARGETS
# ------------------------------------------------------------------------------


.PHONY: run
run:
	@if [ ! -f ./$(BINARY_NAME) ]; then \
		echo "==> Binary './$(BINARY_NAME)' not found. Building first..."; \
		$(MAKE) build; \
	fi
	@echo "==> Running application..."
	@./$(BINARY_NAME)

# Target to run the parser
# Checks if 'output' dir exists and is not empty, then runs docker-compose.
.PHONY: parse
parse:
	@if [ ! -d "output" ] || [ -n "$(ls "output")" ]; then \
		echo "==> Error: 'output' directory is missing or empty."; \
		echo "==> Please run the archiver first to generate files."; \
		exit 1; \
	else \
		echo "==> Found files in 'output'. Changing to 'parser' directory..."; \
		cd parser && $(CONTAINER)-compose up; \
	fi
