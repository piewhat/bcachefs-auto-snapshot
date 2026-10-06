# bcachefs-auto-snapshot

Automatic snapshot tool for bcachefs subvolumes.

## Usage

Configure subvolumes in `/etc/bcachefs-auto-snapshot.ron`:

```ron
[
    (path: "/path/to/subvolume", frequencies: [(Hourly, 24), (Daily, 7)]),
]
```
### Frequencies
- Frequently (15 minutes)
- Hourly
- Daily
- Monthly
- Yearly

## Install
```sh
cargo build --release
sudo cp ./target/release/bcachefs-auto-snapshot /usr/local/bin/bcachefs-auto-snapshot
```

**Important:** If not using the included systemd timer, you must run at precise intervals (`:00`, `:15`, `:30`, `:45`) for proper snapshot timing.

```sh
sudo cp bcachefs-auto-snapshot.{service,timer} /etc/systemd/system/
sudo systemctl enable --now bcachefs-auto-snapshot.timer
```

## Build a debian package

Debian can install all build prerequisites declared by the package and produce a
`.deb` directly from the checkout:

```sh
sudo apt update
sudo apt build-dep .
dpkg-buildpackage --build=binary --no-sign
sudo apt install ../bcachefs-auto-snapshot_0.1.0-1_*.deb
```

The package installs `bcachefs-tools`, the example configuration, and the
systemd service and timer. The timer is deliberately not enabled on install;
edit `/etc/bcachefs-auto-snapshot.ron`, then enable it:

```sh
sudo systemctl enable --now bcachefs-auto-snapshot.timer
```
