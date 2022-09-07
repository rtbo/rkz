use clap::{App, Arg};
use std::process;

mod eos;
mod gas;
mod gases;
mod util;

use eos::{Eos, EosGas};
use gas::Gas;
use gases::GASES;

fn main() {
    let matches = App::new("rkz")
        .version(env!("CARGO_PKG_VERSION"))
        .bin_name("rkz")
        .author("Rémi Thebault <remi.thebault@gmail.com>")
        .set_term_width(80)
        .long_about(concat!(
            "\nComputes compression factor of several gases and mixtures in conditions of pressure and temperature ",
            "using the either of the following equations of state:\n",
            "  - Van der Waals\n",
            "  - Redlich-Kwong (default)\n",
            "  - Soave-Redlich-Kwong\n",
            "  - Peng-Robinson\n",
            "\n",
            "                                                           PV\n",
            "The compression factor for a mole of gas is defined as Z = --.\n",
            "                                                           RT\n\n",
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
            "  - 'N2+O2' => 50% Nitrogen and 50% Oxygen\n\n",
            "DISCLAIMER: rkz is provided \"as is\" without any warranty. See the --license option for details.\n",
        ))
        .after_help(concat!(
            "EXAMPLES:\n",
            "    rkz --list-gas\n",
            "        Print a list of all gases referenced in RKZ\n",
            "    rkz -g N2 -p 200 -t 20\n",
            "        Z-factor of Nitrogen at 200bar and 20°C\n",
            "    rkz -g 78%N2+21%O2+Ar -p 200 -t 50 -e PR\n",
            "        Z-factor of air at 200bar and 50°C with Peng-Robinson equation of state\n",
            "    rkz -g H2 -p 0:1000:10 -t -40:80 -r stdatm\n",
            "        Z-factor CSV table of Hydrogen from 0 to 1000barG and -40 to +80°C\n",
        ))
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
            .help("Specify the pressure in bar. By default absolute unless --relative is used. A range can be specified in the form of start:stop[:step].")
            .takes_value(true))
        .arg(Arg::with_name("equation")
            .short("e")
            .long("eos")
            .help("Specify the equation of state (case insensitive). Choices are VdW for Van-der-Waals, RK for Redlich-Kwong, SRK for Soave-Redlich-Kwong and PR for Peng-Robinson.")
            .takes_value(true)
            .default_value("RK")
        )
        .arg(Arg::with_name("relative")
            .short("r")
            .long("relative")
            .help("Specify that the pressure is relative to the pressure indicated in this parameter (in hPa). \"stdatm\" can be used for 1013.25.")
            .takes_value(true))
        .arg(Arg::with_name("list-gas")
            .long("list-gas")
            .help("Prints a list of referenced gases"))
        .arg(Arg::with_name("license")
            .long("license")
            .help("Prints the license text and exits")
        )
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

    if matches.is_present("license") {
        let license = include_str!("../License.txt");
        print!("{}", license);
        done_something = true;
    }

    let gas = matches.value_of("gas");
    let temperature = matches.value_of("temperature");
    let pressure = matches.value_of("pressure");
    let relative = matches.value_of("relative");
    let eos = matches.value_of("equation");

    match (gas, temperature, pressure) {
        (None, None, None) => {}
        (Some(gas), Some(temperature), Some(pressure)) => {
            match process_args(gas, temperature, pressure, relative, eos) {
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

fn process_args(
    gas: &str,
    temperature: &str,
    pressure: &str,
    relative: Option<&str>,
    eos: Option<&str>,
) -> Result<(), String> {
    let gas = Gas::from_string(gas)?;
    let temperature = Range::parse(temperature)?;
    let mut pressure = Range::parse(pressure)?;
    let relative = relative.map(|r| {
        if r == "stdatm" {
            Ok(1.01325)
        } else {
            util::parse_num(r).map(|r| r / 1000.0)
        }
    });
    // convert from Option<Result<f64>> to Option<f64> (returning the Err if any).
    let relative = match relative {
        Some(Ok(rel)) => Some(rel),
        Some(Err(err)) => return Err(err),
        None => None,
    };

    let eos = match eos {
        Some(eos) => {
            let lw = eos.to_lowercase();
            if lw == "vdw" {
                Eos::VanDerWaals
            } else if lw == "rk" {
                Eos::RedlichKwong
            } else if lw == "srk" {
                Eos::SoaveRedlichKwong
            } else if lw == "pr" {
                Eos::PengRobinson
            } else {
                panic!("Unknown equation of state: {}", eos)
            }
        }
        None => Eos::RedlichKwong,
    };

    if let Some(relative) = relative {
        pressure.start += relative;
        pressure.stop += relative;
    }

    match (temperature.is_scalar(), pressure.is_scalar()) {
        (true, true) => {
            let p_pa = pressure.start * 100000f64;
            let t_k = temperature.start + 273.15;
            println!("{}", gas.z(eos, p_pa, t_k));
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
                let phead = if let Some(relative) = relative {
                    p - relative
                } else {
                    p
                };
                print!("\n{}", phead);
                let p = p * 100000f64;
                for t in temperature.iter().map(|t| t + 273.15f64) {
                    print!("\t{}", gas.z(eos, p, t));
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
