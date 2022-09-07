use crate::gases::GASES;
use crate::util;
#[cfg(test)]
use float_cmp::assert_approx_eq;

pub fn find_gas(id: &str) -> Option<&PureGas> {
    GASES.iter().find(|g| g.id == id)
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PureGas {
    /// Identifier of the gas
    pub id: &'static str,
    /// Name of the gas
    pub name: &'static str,
    /// Critical temperature in K
    pub tc: f64,
    /// Critical pressure in Pa
    pub pc: f64,
    /// Acentric factor
    pub w: f64,
}

#[derive(Clone, Debug)]
pub struct GasMixture {
    /// Components of the gas
    pub comps: Vec<(f64, PureGas)>,
}

pub trait GasComp {
    fn molar_fraction(&self) -> f64;
    fn pure_gas(&self) -> &PureGas;
}

impl GasComp for (f64, PureGas) {
    fn molar_fraction(&self) -> f64 {
        self.0
    }
    fn pure_gas(&self) -> &PureGas {
        &self.1
    }
}

#[derive(Clone, Debug)]
pub enum Gas {
    Pure(PureGas),
    Mixture(GasMixture),
}

impl Gas {
    pub fn from_string(input: &str) -> Result<Gas, String> {
        let comps = {
            let mut v: Vec<&str> = Vec::new();
            for s in input.split('+') {
                v.push(s);
            }
            v
        };

        if comps.is_empty() {
            unreachable!();
        }

        if comps.len() == 1 {
            let gas = find_gas(comps[0]).ok_or("The requested gas is not referenced")?;
            Ok(Gas::Pure(*gas))
        } else {
            const NO_FRAC: f64 = -1f64;

            let mut gas_comps = Vec::new();

            for comp in comps.into_iter() {
                let frac_gas: Vec<&str> = comp.split('%').collect();
                if frac_gas.is_empty() {
                    unreachable!()
                }
                if frac_gas.len() > 2 {
                    return Err(format!(
                        "\"{}\" from \"{}\" is invalid gas spec",
                        comp, input
                    ));
                }
                let gas = find_gas(frac_gas[frac_gas.len() - 1])
                    .ok_or("The requested gas is not referenced")?;
                if frac_gas.len() == 1 {
                    gas_comps.push((NO_FRAC, *gas));
                } else {
                    let frac = util::parse_num(frac_gas[0])?;
                    if frac <= 0f64 {
                        return Err("molar fraction cannot be negative".into());
                    }
                    gas_comps.push((frac / 100f64, *gas));
                }
            }

            let (total_frac, num_no_frac) = {
                let mut total = 0f64;
                let mut num = 0;
                for c in gas_comps.iter() {
                    if c.0 == NO_FRAC {
                        num += 1;
                    } else {
                        total += c.0;
                    }
                }
                (total, num)
            };

            if total_frac > 1f64 || (total_frac - 1f64).abs() < f64::EPSILON && num_no_frac > 0 {
                return Err("total molar fraction is too high".into());
            } else if total_frac < 1f64 && num_no_frac == 0 {
                return Err("total molar fraction is too low".into());
            } else {
                let missing = (1f64 - total_frac) / num_no_frac as f64;
                for c in gas_comps.iter_mut() {
                    if c.0 == NO_FRAC {
                        c.0 = missing;
                    }
                }
            }

            Ok(Gas::Mixture(GasMixture { comps: gas_comps }))
        }
    }
}

#[cfg(test)]
impl Gas {
    fn is_pure(&self) -> bool {
        match self {
            Gas::Pure(_) => true,
            _ => false,
        }
    }

    fn pure(&self) -> PureGas {
        match self {
            Gas::Pure(g) => *g,
            _ => panic!("{:?} is not pure", self),
        }
    }

    fn is_mixture(&self) -> bool {
        match self {
            Gas::Mixture(_) => true,
            _ => false,
        }
    }

    fn mixture(&self) -> GasMixture {
        match self {
            Gas::Mixture(g) => g.clone(),
            _ => panic!("{:?} is not a mixture", self),
        }
    }
}

#[test]
fn test_gas_parse() {
    let gas = Gas::from_string("N2");
    assert!(gas.is_ok());
    let gas = gas.unwrap();
    assert!(gas.is_pure());
    assert_eq!(gas.pure(), *find_gas("N2").unwrap());

    let gas = Gas::from_string("80%N2+20%O2");
    assert!(gas.is_ok());
    let gas = gas.unwrap();
    assert!(gas.is_mixture());
    let gas = gas.mixture();
    assert_eq!(gas.comps.len(), 2);
    assert_approx_eq!(f64, gas.comps[0].molar_fraction(), 0.8);
    assert_eq!(gas.comps[0].pure_gas(), find_gas("N2").unwrap());
    assert_approx_eq!(f64, gas.comps[1].molar_fraction(), 0.2);
    assert_eq!(gas.comps[1].pure_gas(), find_gas("O2").unwrap());

    let gas = Gas::from_string("80%N2+O2");
    assert!(gas.is_ok());
    let gas = gas.unwrap();
    assert!(gas.is_mixture());
    let gas = gas.mixture();
    assert_eq!(gas.comps.len(), 2);
    assert_approx_eq!(f64, gas.comps[0].molar_fraction(), 0.8);
    assert_eq!(gas.comps[0].pure_gas(), find_gas("N2").unwrap());
    assert_approx_eq!(f64, gas.comps[1].molar_fraction(), 0.2);
    assert_eq!(gas.comps[1].pure_gas(), find_gas("O2").unwrap());

    let gas = Gas::from_string("N2+O2");
    assert!(gas.is_ok());
    let gas = gas.unwrap();
    assert!(gas.is_mixture());
    let gas = gas.mixture();
    assert_eq!(gas.comps.len(), 2);
    assert_approx_eq!(f64, gas.comps[0].molar_fraction(), 0.5);
    assert_eq!(gas.comps[0].pure_gas(), find_gas("N2").unwrap());
    assert_approx_eq!(f64, gas.comps[1].molar_fraction(), 0.5);
    assert_eq!(gas.comps[1].pure_gas(), find_gas("O2").unwrap());

    let gas = Gas::from_string("80%N2+30%O2");
    assert!(gas.is_err());
    assert_eq!(gas.err().unwrap(), "total molar fraction is too high");

    let gas = Gas::from_string("80%N2+10%O2");
    assert!(gas.is_err());
    assert_eq!(gas.err().unwrap(), "total molar fraction is too low");

    let gas = Gas::from_string("100%N2+O2");
    assert!(gas.is_err());
    assert_eq!(gas.err().unwrap(), "total molar fraction is too high");
}
