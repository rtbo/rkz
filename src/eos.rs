//! Equation of State
use super::gas::{Gas, GasComp, GasMixture, PureGas};
use roots::{self, Roots};
#[cfg(test)]
use float_cmp::assert_approx_eq;

const R: f64 = 8.31446262;

/// Equation of state
#[derive(Copy, Clone, Debug)]
pub enum Eos {
    VanDerWaals,
    RedlichKwong,
    SoaveRedlichKwong,
    PengRobinson,
}

pub trait EosGas {
    fn a(&self, eos: Eos, t: f64) -> f64;
    fn b(&self, eos: Eos) -> f64;

    fn z(&self, eos: Eos, p: f64, t: f64) -> f64 {
        let (a3, a2, a1, a0) = match eos {
            Eos::VanDerWaals => {
                let a = self.a(eos, t) * p / (R * R * t * t);
                let b = self.b(eos) * p / (R * t);

                let a3 = 1f64;
                let a2 = -b - 1f64;
                let a1 = a;
                let a0 = -a * b;

                (a3, a2, a1, a0)
            }
            Eos::RedlichKwong => {
                let a = self.a(eos, t) * p / (R * R * t.powf(2.5));
                let b = self.b(eos) * p / (R * t);

                let a3 = 1f64;
                let a2 = -1f64;
                let a1 = a - b * b - b;
                let a0 = -a * b;

                (a3, a2, a1, a0)
            }
            Eos::SoaveRedlichKwong => {
                let a = self.a(eos, t) * p / (R * R * t * t);
                let b = self.b(eos) * p / (R * t);

                let a3 = 1f64;
                let a2 = -1f64;
                let a1 = a - b * b - b;
                let a0 = -a * b;

                (a3, a2, a1, a0)
            }
            Eos::PengRobinson => {
                let a = self.a(eos, t) * p / (R * R * t * t);
                let b = self.b(eos) * p / (R * t);

                let a3 = 1f64;
                let a2 = b - 1f64;
                let a1 = -3f64 * b * b - 2f64 * b + a;
                let a0 = b * b * b + b * b - a * b;

                (a3, a2, a1, a0)
            }
        };

        let roots = roots::find_roots_cubic(a3, a2, a1, a0);
        match roots {
            Roots::No(_) => panic!("could not find Z-factor root"),
            Roots::One([root]) => root,
            Roots::Two(roots) => roots[0].max(roots[1]),
            Roots::Three(roots) => roots[0].max(roots[1]).max(roots[2]),
            _ => unreachable!(),
        }
    }
}

impl EosGas for PureGas {
    fn a(&self, eos: Eos, t: f64) -> f64 {
        match eos {
            Eos::VanDerWaals => 27f64 * R * R * self.tc * self.tc / (64f64 * self.pc),
            Eos::RedlichKwong => 0.42748023 * R * R * self.tc.powf(2.5) / self.pc,
            Eos::SoaveRedlichKwong => {
                let m = 0.48 + 1.574 * self.w - 0.176 * self.w * self.w;
                let alpha = 1f64 + m * (1f64 - (t / self.tc).sqrt());
                let alpha = alpha * alpha;
                alpha * 0.42748023 * R * R * self.tc * self.tc / self.pc
            }
            Eos::PengRobinson => {
                let m = if self.w <= 0.491 {
                    0.37464 + 1.56226 * self.w - 0.26992 * self.w * self.w
                } else {
                    0.379642 + 1.487503 * self.w
                        - 0.164423 * self.w * self.w
                        - 0.016666 * self.w * self.w * self.w
                };
                let alpha = 1f64 + m * (1f64 - (t / self.tc).sqrt());
                let alpha = alpha * alpha;
                alpha * 0.45724 * R * R * self.tc * self.tc / self.pc
            }
        }
    }
    fn b(&self, eos: Eos) -> f64 {
        match eos {
            Eos::VanDerWaals => R * self.tc / (8f64 * self.pc),
            Eos::RedlichKwong | Eos::SoaveRedlichKwong => 0.08664035 * R * self.tc / self.pc,
            Eos::PengRobinson => 0.0778 * R * self.tc / self.pc,
        }
    }
}

impl EosGas for GasMixture {
    fn a(&self, eos: Eos, t: f64) -> f64 {
        let mut res = 0f64;
        for i in self.comps.iter() {
            let ai = i.pure_gas().a(eos, t);
            for j in self.comps.iter() {
                let aj = j.pure_gas().a(eos, t);
                res += i.molar_fraction() * j.molar_fraction() * (ai * aj).sqrt();
            }
        }
        res
    }

    fn b(&self, eos: Eos) -> f64 {
        let mut res = 0f64;
        for i in self.comps.iter() {
            res += i.molar_fraction() * i.pure_gas().b(eos);
        }
        res
    }
}

impl EosGas for Gas {
    fn a(&self, eos: Eos, t: f64) -> f64 {
        match self {
            Gas::Pure(g) => g.a(eos, t),
            Gas::Mixture(g) => g.a(eos, t),
        }
    }
    fn b(&self, eos: Eos) -> f64 {
        match self {
            Gas::Pure(g) => g.b(eos),
            Gas::Mixture(g) => g.b(eos),
        }
    }
}

#[test]
fn test_eos() {

    let h2 = Gas::from_string("H2").unwrap();
    let p700b = 101325f64 + 70_000_000f64;
    let t15c = 273.15 + 15f64;
    assert_approx_eq!(f64, h2.z(Eos::VanDerWaals, p700b, t15c), 1.6818452, epsilon = 0.00001);
    assert_approx_eq!(f64, h2.z(Eos::RedlichKwong, p700b, t15c), 1.506842, epsilon = 0.00001);
    assert_approx_eq!(f64, h2.z(Eos::SoaveRedlichKwong, p700b, t15c), 1.48638434, epsilon = 0.00001);
    assert_approx_eq!(f64, h2.z(Eos::PengRobinson, p700b, t15c), 1.396375, epsilon = 0.00001);
}
