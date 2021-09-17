use clap::{App, Arg};
use std::iter;
use std::process;

mod gas;
mod rk;

use gas::GASES;
use rk::RkGas;

fn main() {
    let matches = App::new("Redlich-Kwong Z-factor calculator")
        .version("0.1.0")
        .author("Rémi Thebault <remi.thebault@gmail.com>")
        .about(concat!(
            "Computes compression (Z) factor of several gases in conditions of pressure and temperature. ",
            "If a range is provided for pressure or temperature, the result is written in CSV format for ",
            "each element of both range (1 row per pressure condition, 1 column per temperature condition)"
        ))
        .arg(Arg::with_name("gas")
            .short("g")
            .long("gas")
            .help("Specify the gas by id. (--list-gas to show referenced gases)")
            .takes_value(true))
        .arg(Arg::with_name("temperature")
            .short("T")
            .long("temperature")
            .allow_hyphen_values(true)
            .help("Specify the temperature in °C. A range can be specified in the form of start:stop[:step].")
            .takes_value(true))
        .arg(Arg::with_name("pressure")
            .short("P")
            .long("pressure")
            .help("Specify the abs. pressure in bar. A range can be specified in the form of start:stop[:step].")
            .takes_value(true))
        .arg(Arg::with_name("list-gas")
            .long("list-gas")
            .help("Print a list of referenced gases"))
        .get_matches();

    let mut done_something = false;

    if matches.is_present("list-gas") {
        println!("Gases referenced by RKZ:");
        println!("    ID      Name");
        for g in GASES.iter() {
            let space = 8 - g.id.chars().count();
            assert!(space > 0);
            let space = iter::repeat(" ").take(space).collect::<String>();
            println!("    {}{}{}", g.id, space, g.name);
        }
        done_something = true;
    }

    let gas = matches.value_of("gas");
    let temperature = matches.value_of("temperature");
    let pressure = matches.value_of("pressure");

    match (gas, temperature, pressure) {
        (None, None, None) => {}
        (Some(gas), Some(temperature), Some(pressure)) => {
            match process_args(gas, temperature, pressure) {
                Err(err) => {
                    eprintln!("{}", err);
                    process::exit(1);
                }
                _ => {
                    done_something = true;
                }
            }
        }
        _ => {
            eprintln!("Unsufficient parameters. Please specify gas, temperature and pressure");
            process::exit(1);
        }
    }

    if !done_something {
        eprintln!("No parameter supplied.");
        process::exit(1);
    }
}

fn process_args(gas: &str, temperature: &str, pressure: &str) -> Result<(), String> {
    let gas = GASES
        .iter()
        .find(|g| g.id == gas)
        .ok_or("The requested gas is not referenced")?;
    let temperature = Range::parse(temperature)?;
    let pressure = Range::parse(pressure)?;

    match (temperature.is_scalar(), pressure.is_scalar()) {
        (true, true) => {
            let p_pa = pressure.start * 100000f64;
            let t_k = temperature.start + 273.15;
            println!("{}", gas.z(p_pa, t_k));
        }
        (_, _) => {
            // writing CSV
            // header
            print!("P \\ T");
            for t in temperature.iter() {
                print!("\t{}", t);
            }
            // rows
            for p in pressure.iter() {
                print!("\n{}", p);
                let p = p * 100000f64;
                for t in temperature.iter().map(|t| t + 273.15f64) {
                    print!("\t{}", gas.z(p, t));
                }
            }
        }
    }
    Ok(())
}

struct Range {
    start: f64,
    stop: f64,
    step: f64,
}

impl Range {
    fn parse(input: &str) -> Result<Range, String> {
        let v = {
            let mut v: Vec<f64> = Vec::new();
            for s in input.split(':') {
                v.push(parse_num(s)?);
            }
            v
        };

        match v.len() {
            1 => {
                let val = v[0];
                Ok(Range {
                    start: val,
                    stop: val,
                    step: 1f64,
                })
            }
            2 => {
                let start = v[0];
                let stop = v[1];
                if stop <= start {
                    Err("Range stop must be higher than start".into())
                } else {
                    Ok(Range {
                        start,
                        stop,
                        step: 1f64,
                    })
                }
            }
            3 => {
                let start = v[0];
                let stop = v[1];
                let step = v[2];
                if stop <= start {
                    Err("Range stop must be higher than start".into())
                } else if step <= 0f64 {
                    Err("Range step must be positive".into())
                } else {
                    Ok(Range { start, stop, step })
                }
            }
            _ => Err(format!("Can't parse \"{}\" as a range", input)),
        }
    }

    fn is_scalar(&self) -> bool {
        self.start + self.step > self.stop
    }

    fn iter(&self) -> ScalarIt {
        ScalarIt {
            cur: self.start,
            stop: self.stop,
            step: self.step,
        }
    }
}

struct ScalarIt {
    cur: f64,
    stop: f64,
    step: f64,
}

impl Iterator for ScalarIt {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur <= self.stop {
            let res = self.cur;
            self.cur += self.step;
            Some(res)
        } else {
            None
        }
    }
}

fn parse_num(input: &str) -> Result<f64, String> {
    input
        .parse::<f64>()
        .map_err(|_| format!("Can't parse {} as a number", input))
}
