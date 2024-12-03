# tuya-web

tuya-web is a web service to toggle Tuya smart outlets on or off through a REST API.

It uses https://github.com/fruitiex/rust-async-tuyapi under the hood.

## Compile

Run `cargo build`

## Running

Run `cargo run -- --help` for a list of options.

An example for two different devices would be:

```sh
RUST_LOG=warn cargo run -- \
    -n livingroom-1 -d aaaaaa01234 -k 'SomeSecret' -a 10.0.5.50 -p 3.4 \
    -n livingroom-2 -d bbbbbb56789 -k 'SecondCode' -a 10.0.5.51 -p 3.3
```

You can then turn on the first output by sending: `curl -X PUT "http://localhost:3000/outlet/0/true"`.

While turning off the second output with: `curl -X PUT "http://localhost:3000/outlet/1/false"`

Toggling an outlet is done with: `curl -X POST "http://localhost:3000/outlet/0"`

Get metrics: `curl http://localhost:3000/metrics`
```
# TYPE tuya_smartplug_scrapes_total counter
tuya_smartplug_scrapes_total 1

# TYPE tuya_smartplug_voltage gauge
tuya_smartplug_voltage{device="livingroom-1"} 227.84

# TYPE tuya_smartplug_power gauge
tuya_smartplug_power{device="livingroom-1"} 1.13

# TYPE tuya_smartplug_count_devices gauge
tuya_smartplug_count_devices 1

# TYPE tuya_smartplug_last_scrape_error gauge
tuya_smartplug_last_scrape_error 0

# TYPE tuya_smartplug_frequency gauge
tuya_smartplug_frequency{device="livingroom-1"} 49.97

# TYPE tuya_smartplug_current gauge
tuya_smartplug_current{device="livingroom-1"} 0.146
```

## License

tuya-web is licensed under GNU AGPL v3 or later, see the `LICENSE` file for the full license.

```
tuya-web - a web service for tuya outlets
Copyright (c) 2024, Linus Karlsson

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU Affero General Public License as
published by the Free Software Foundation, either version 3 of the
License, or (at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
```

## Links
* [dulfer/localtuya-device-datapoints](https://github.com/dulfer/localtuya-device-datapoints)
* [rkosegi/tuya-smartplug-exporter](https://github.com/rkosegi/tuya-smartplug-exporter)
