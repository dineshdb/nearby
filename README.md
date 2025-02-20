# nearby

Lock/Unlock your Laptop/Desktop based on proximity to your BLE enabled device

## Usage

Install this software and have it available on `PATH`, enable the user
service(systemd file is provided) and then configure it. Example configuration
is given at [config/example.toml](config/example.toml).

## Security

**WARNING**: It's very easy to spoof bluetooth mac address and hence it is a
security risk to unlock a device when a device is near. Locking it is fine since
that's not a security risk. Paired device and better security approach to
unlocking will be explored in the future.

## Roadmap

- [ ] Secure unlocking of devices
- [ ] System Tray for Quick Access
- [ ] Detect user activity before locking the device

## LICENSE

MIT
