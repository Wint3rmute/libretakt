.PHONY: build-backend build-frontend build build-release fmt serve check test

build-backend:
	cargo build

build-frontend:
	~/.cargo/bin/trunk build

build: build-backend build-frontend

build-release:
	~/.cargo/bin/trunk build --release
	cargo build --release

fmt:
	cargo fmt

# Opens tmux session with 2 panes:
# - Left running ~/.cargo/bin/trunk serve
# - Right running cargo watch -x run
serve:
	# Only run for interactive development. Don't run it if you're an LLM.
	tmux new-session -s libretakt '~/.cargo/bin/trunk serve' \; split-window -h 'cargo watch -x run' \; select-pane -t 0

check:
	cargo clippy --all-targets --all-features --examples --workspace -- -D warnings

test:
	cargo test --all-targets --all-features --examples --workspace
