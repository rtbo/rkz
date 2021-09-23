use clap::{App, Arg};
use std::process;

mod gas;
mod gases;
mod rk;
mod util;

use gas::Gas;
use gases::GASES;
use rk::RkGas;

fn main() {
    let matches = App::new("rkz")
        .version("v0.1.0")
        .bin_name("rkz")
        .author("Rémi Thebault <remi.thebault@gmail.com>")
        .set_term_width(80)
        .long_about(concat!(
            "\nComputes compression factor of several gases and mixtures in conditions of pressure and temperature ",
            "using the Redlich-Kwong equation of state.\n\n",
            "                                                  PV\n",
            "The compression factor of a gas is defined as Z = ---.\n",
            "                                                  nRT\n\n",
            "A range can be provided instead of scalar values for pressure or temperature. In such case, ",
            "the result is written in CSV format with one Z value per combination of pressure and temperature ",
            "(1 row per pressure condition, 1 column per temperature condition).\n",
            "Range are provided in the form of min:max[:step] (e.g. '20:800' or '20:800:10'). ",
            "If step is omitted, it is assumed to be equal to one.\n\n",
            "Mixture for option --gas|-g can be specified in the form of molar_fraction%gas_id+[molar_fraction%gas_id]. ",
            "Mixture molar fractions can only be specified as percentage or be omitted. ",
            "Gases without molar fraction evenly take the rest of the mixture. Examples:\n",
            "  - '80%N2+20%O2' => 80% Nitrogen and 20% Oxygen\n",
            "  - '80%N2+O2' => 80% Nitrogen and 20% Oxygen\n",
            "  - '80%N2+O2+CO2' => 80% Nitrogen, 10% Oxygen and 10% Carbon dioxide\n",
            "  - '78%N2+21%O2+Ar' => air composition (more or less)\n",
            "  - 'N2+O2' => 50% Nitrogen and 50% Oxygen\n",
        ))
        .after_help(concat!(
            "EXAMPLES:\n",
            "    rkz --list-gas\n",
            "            Print a list of all gases referenced in RKZ\n",
            "    rkz -g N2 -p 200 -t 20\n",
            "            Z-factor of Nitrogen at 200bar and 20°C\n",
            "    rkz -g 78%N2+21%O2+Ar -p 200 -t 50\n",
            "            Z-factor of air at 200bar and 50°C\n",
            "    rkz -g H2 -p 1:1000:10 -t -40:80\n",
            "            Z-factor CSV table of Hydrogen from 1 to 1000bar and -40 to +80°C\n",
        )
    )
        .arg(Arg::with_name("gas")
            .short("g")
            .long("gas")
            .help("Specify the gas by id or by mixture spec (see above)")
            .takes_value(true))
        .arg(Arg::with_name("temperature")
            .short("t")
            .long("temperature")
            .allow_hyphen_values(true)
            .help("Specify the temperature in °C. A range can be specified in the form of start:stop[:step].")
            .takes_value(true))
        .arg(Arg::with_name("pressure")
            .short("p")
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
        println!("    ID        Name");
        for g in GASES.iter() {
            let space = 10 - g.id.chars().count();
            assert!(space > 0);
            let space = " ".repeat(space);
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
    let gas = Gas::from_string(gas)?;
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
            println!();
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
                v.push(util::parse_num(s)?);
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
