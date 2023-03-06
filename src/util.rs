/// util.rs: auxillary/helper functions for the emulator
/// Copyright (C) 2023 Justin Noah <justinnoah+rusty_chips@gmail.com>

/// This program is free software: you can redistribute it and/or modify
/// it under the terms of the GNU Affero General Public License as published
/// by the Free Software Foundation, either version 3 of the License, or
/// (at your option) any later version.

/// This program is distributed in the hope that it will be useful,
/// but WITHOUT ANY WARRANTY; without even the implied warranty of
/// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
/// GNU Affero General Public License for more details.

/// You should have received a copy of the GNU Affero General Public License
/// along with this program.  If not, see <https://www.gnu.org/licenses/>.
use regex::Regex;

pub fn test_roms() -> Vec<Vec<u8>> {
    let mut roms = Vec::new();
    #[allow(non_snake_case)]
    let ESC = Vec::from([
        0x00, 0xE0, 0xA0, 0x96, 0x60, 0x00, 0x61, 0x00, 0xD0, 0x15, 0xA0, 0xA0, 0x60, 0x05, 0xD0,
        0x15, 0xA0, 0x8C, 0x60, 0x0A, 0xD0, 0x15,
    ]);
    roms.push(ESC);
    roms
}

fn input_to_hertz(input: &str) -> u128 {
    let re_num = Regex::new(r"\d+(\.\d+)?").unwrap();
    let num_range: (usize, usize) = {
        let matches = re_num.find(input).unwrap();
        (matches.start(), matches.end())
    };
    let number: f64 = input[num_range.0..num_range.1].trim().parse().unwrap();
    let freq = input[num_range.1..].to_string().to_lowercase();
    let multiplier = match freq.as_str() {
        "ghz" => 1000 * 1000,
        "mhz" => 1000,
        "hz" => 1,
        _ => panic!("Chip8 Frequency must end with GHz, MHz, or Hz"),
    };
    let frequency_in_hertz = number * (multiplier as f64);
    frequency_in_hertz.floor() as u128
}

fn hertz_to_seconds(hertz: u128) -> f64 {
    1f64 / (hertz as f64)
}

pub fn hz_to_secs(input: &str) -> f64 {
    hertz_to_seconds(input_to_hertz(input))
}
