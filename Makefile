# ============================================================
# YODPAY CORE AUTOMATED DIAGNOSTICS MATRIX
# ============================================================

.PHONY: test check format bench run-demo

test:
	cargo test --workspace

check:
	cargo check --workspace

format:
	cargo fmt --all -- --check
	cargo clippy --workspace -- -D warnings

bench:
	cargo bench

run-demo:
	cargo run --bin yodpay-demo
