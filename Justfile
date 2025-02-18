run:
	cp config/nearby.toml $HOME/.config/nearby/config.toml
	dbox exec fedora-41 'cargo build'
	cargo run

build:
	cargo build

install:
	dbox exec fedora-41 'cargo install --path . --force'
	cp systemd/nearby.service $HOME/.config/systemd/user/nearby.service
	systemctl --user daemon-reload
	systemctl --user restart nearby

lint: fmt clippy check test

clippy:
	cargo clippy -- -D warnings

check:
	nice cargo check --workspace

test:
	nice ionice cargo test --workspace --all-targets --all-features

fmt:
	cargo fmt --all
