# multiping

Send ping to multiple hosts simultaneously for LAN monitoring.

https://user-images.githubusercontent.com/679719/234576907-0a2d3b8a-6690-4485-a420-cd7d8a3c6d24.mov

## Install

```console
$ cargo install --git https://github.com/0x6b/multiping
```

## Uninstall

```console
$ cargo uninstall multiping
```

## Usage

```console
$ multiping --help
Send ping to multiple hosts simultaneously for LAN monitoring.

Usage: multiping [OPTIONS] [TARGETS]...

Arguments:
  [TARGETS]...  Space seperated ping targets [default: "192.168.0.10
                turingpi.local 192.168.0.31 192.168.0.32 192.168.0.33
                192.168.0.34"]

Options:
  -i, --interval <INTERVAL>  Specify ping interval in seconds [default: 1]
  -t, --timeout <TIMEOUT>    Specify ping timeout in seconds [default: 1]
  -h, --help                 Print help
  -V, --version              Print version
```

## LICENSE

MIT. See [LICENSE](LICENSE) for details.
