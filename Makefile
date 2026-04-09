# ── Paths ─────────────────────────────────────────────────────────────

BENCH_DIR  := $(abspath $(dir $(lastword $(MAKEFILE_LIST))))
AGFS_ROOT  := $(abspath $(BENCH_DIR)/..)
YOLO_BENCH := $(AGFS_ROOT)/local/target/release/yolo-bench
PAPER_DIR  ?= $(AGFS_ROOT)/paper

# ── Paper artifact installation ───────────────────────────────────────

.PHONY: install-paper

install-paper:
	cargo build --release
	$(YOLO_BENCH) install-paper --paper-dir $(PAPER_DIR)
