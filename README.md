# tuya-web

tuya-web is a web service to toggle Tuya smart outlets on or off through a REST API.

It uses https://github.com/fruitiex/rust-async-tuyapi under the hood.

## Compile

Run `cargo build`

## Running

Run `cargo run -- --help` for a list of options.

An example for two different devices would be:

```sh
cargo run -- \
    -d aaaaaa01234 -k 'SomeSecret' -d 10.0.5.50 \
    -d bbbbbb56789 -k 'SecondCode' -d 10.0.5.51
```

You can then turn on the first output by sending: `curl -X PUT "http://localhost:3000/outlet/0/true"`.

While turning off the second output with: `curl -X PUT "http://localhost:3000/outlet/1/false"`

Toggling an outlet is done with: `curl -X POST "http://localhost:3000/outlet/0"`

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
