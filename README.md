SailStats Logger
================

A logger for navigational data on boats. Currently supporting Yacht Devices NMEA 2000 Wi-Fi Gateway YDWG-02 in UDP propagating mode.

Build for Raspberry PI
----------------------
1. Run `git clone https://github.com/setnorth/sailstats-logger sailstats-logger`
2. Run `cd sailstats-logger && cargo build --target arm-unknown-linux-musleabi --release`
3. Copy to raspberry and install the service from misc/ directory.
TODO: More thorough documentation how to build and install.

License
-------
SailStats Logger 0.1.0a
Copyright (C) 2021  Thorsten Ernst Schilling <thorsten.schilling@gmail.com>

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU Affero General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
