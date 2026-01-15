# bcachefs-auto-snapshot

Automatic snapshot tool for bcachefs subvolumes.

## Usage

Configure subvolumes in `/etc/bcachefs-auto-snapshot.ron`:

```ron
[
    (path: "/path/to/subvolume", frequencies: [(Hourly, 24), (Daily, 7)]),
]
```

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
