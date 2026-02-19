# Makefile for Soplang - The Somali Programming Language
# This file provides simple commands for common development tasks

# Python command (can be overridden by environment)
PYTHON := python3
PIP := $(PYTHON) -m pip

# Directories to clean/search (Python implementation and tests live in psrc/)
SRC_DIRS := psrc examples scripts

# Command flags
PYTEST_FLAGS := -v --cov=psrc
FLAKE8_FLAGS := --max-line-length=88 --extend-ignore=E203
BLACK_FLAGS := --line-length=88
ISORT_FLAGS := --profile black --filter-files
PYLINT_FLAGS := --disable=C0111,C0103,C0330,C0326,W0511,R0903,R0913,R0914,R0902,E1101,W0212

# Default target
.PHONY: help
help:
	@echo "Soplang Development Commands (Rust — primary):"
	@echo "  make build       - Build release binary (cargo build --release)"
	@echo "  make run         - Run a .sop file: make run FILE=examples/hello.sop"
	@echo "  make shell       - Run interactive REPL (Rust)"
	@echo "  make test-rust   - Run Rust unit and integration tests"
	@echo "  make bench       - Run criterion benchmarks"
	@echo ""
	@echo "Legacy / Python (psrc/):"
	@echo "  make install     - Install Python dependencies (dev and regular)"
	@echo "  make install-dev - Install development dependencies only"
	@echo "  make test        - Run Python tests (pytest)"
	@echo "  make format      - Format code with black and isort"
	@echo "  make lint        - Check code with flake8 and pylint"
	@echo "  make check       - Run all checks (format + lint + test)"
	@echo "  make precommit   - Run pre-commit hooks on all files"
	@echo "  make clean       - Remove Python cache and build artifacts"
	@echo "  make docker-build - Build Docker image"
	@echo "  make docker-run  - Run Soplang in Docker container"
	@echo "  make run-py FILE=<file> - Run a file with Python implementation"

# Installation targets
.PHONY: install install-dev
install:
	$(PIP) install -r psrc/requirements.txt
	$(PIP) install -r psrc/requirements-dev.txt
	pre-commit install

install-dev:
	$(PIP) install -r psrc/requirements-dev.txt
	pre-commit install

# Testing target (Python tests live in psrc/)
.PHONY: test
test:
	$(PYTHON) psrc/run_tests.py

# Code formatting targets
.PHONY: format format-check
format:
	$(PYTHON) -m black $(BLACK_FLAGS) $(SRC_DIRS)
	$(PYTHON) -m isort $(ISORT_FLAGS) $(SRC_DIRS)

format-check:
	$(PYTHON) -m black $(BLACK_FLAGS) --check $(SRC_DIRS)
	$(PYTHON) -m isort $(ISORT_FLAGS) --check-only $(SRC_DIRS)

# Linting targets
.PHONY: lint flake8 pylint
lint: flake8 pylint

flake8:
	$(PYTHON) -m flake8 $(FLAKE8_FLAGS) $(SRC_DIRS)

pylint:
	$(PYTHON) -m pylint $(PYLINT_FLAGS) $(SRC_DIRS)

# Pre-commit hooks
.PHONY: precommit
precommit:
	pre-commit run --all-files

# Combined check target
.PHONY: check
check: format-check lint test

# Clean up target (Python cache + Rust target/)
.PHONY: clean
clean:
	find . -type d -name __pycache__ -exec rm -rf {} +
	find . -type f -name "*.pyc" -delete
	find . -type f -name "*.pyo" -delete
	find . -type f -name "*.pyd" -delete
	find . -type f -name ".coverage" -delete
	find . -type d -name "*.egg-info" -exec rm -rf {} +
	find . -type d -name "*.egg" -exec rm -rf {} +
	find . -type d -name ".pytest_cache" -exec rm -rf {} +
	find . -type d -name ".coverage" -exec rm -rf {} +
	find . -type d -name "htmlcov" -exec rm -rf {} +
	find . -type d -name ".tox" -exec rm -rf {} +
	find . -type d -name "dist" -exec rm -rf {} +
	find . -type d -name "build" -exec rm -rf {} +
	cargo clean 2>/dev/null || true

# Docker targets
.PHONY: docker-build docker-run
docker-build:
	docker-compose build

docker-run:
	docker-compose up -d
	docker-compose exec soplang python -m psrc

# --- Rust targets (primary implementation) ---
.PHONY: build run shell test-rust bench
build:
	cargo build --release

run:
	@if [ -z "$(FILE)" ]; then \
		echo "Usage: make run FILE=<filename>"; \
		exit 1; \
	fi
	./target/release/soplang $(FILE)

shell:
	./target/release/soplang -i

test-rust:
	cargo test

bench:
	cargo bench

# --- Run targets for Python (legacy) ---
.PHONY: run-py shell-py
run-py:
	@if [ -z "$(FILE)" ]; then \
		echo "Usage: make run-py FILE=<filename>"; \
		exit 1; \
	fi
	$(PYTHON) main.py $(FILE)

shell-py:
	$(PYTHON) main.py
