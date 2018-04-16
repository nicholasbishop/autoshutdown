# autoshutdown

Automatically shut down the machine after a period of inactivity.

The last-modified time of a file (default: `/run/last_heartbeat`) is
used to indicate when the machine was last active. The file is checked
at a fixed interval (default: one minute). There is a grace period
(default: five minutes) before the machine is powered off, by which
time a new heartbeat may have arrived.

The `check-interval` and `grace-duration` arguments accept numbers
with a unit: `h`, `m`, or `s`. For example, `5m` indicates five
minutes.

The default shutdown command is `poweroff`.
