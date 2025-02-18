run:
	cp config/nearby.toml $HOME/.config/nearby/config.toml
	dbox exec fedora-41 'cargo build'
	cargo run

install:
	dbox exec fedora-41 'cargo install --path . --force'
	cp systemd/nearby.service $HOME/.config/systemd/user/nearby.service
	systemctl --user daemon-reload
	systemctl --user restart nearby
