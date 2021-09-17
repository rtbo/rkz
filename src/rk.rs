use roots::{self, Roots};
use super::gas::PureGas;

const R: f64 = 6.3145;

/// Redlich-Kwong A factor
const RK_A: f64 = 0.42748023;
/// Redlich-Kwong B factor
const RK_B: f64 = 0.08664035;

pub trait RkGas {
    fn a_const(&self) -> f64;
    fn b_const(&self) -> f64;

    fn a(&self, p: f64, t: f64) -> f64 {
        self.a_const() * p / (R.powi(2) * t.powf(2.5))
    }

    fn b(&self, p: f64, t: f64) -> f64 {
        self.b_const() * p / (R * t)
    }

    fn z(&self, p: f64, t: f64) -> f64 {
        let a = self.a(p, t);
        let b = self.b(p, t);

        let a3 = 1.0;
        let a2 = -1.0;
        let a1 = a - b*b - b;
        let a0 = -a*b;
        let roots = roots::find_roots_cubic(a3, a2, a1, a0);
        match roots {
            Roots::No(_) => panic!("could not find Z-factor root"),
            Roots::One([root]) => root,
            Roots::Two(roots) => roots[1],
            Roots::Three(roots) => roots[2],
            _ => unreachable!(),
        }
    }
}

impl RkGas for PureGas {
    fn a_const(&self) -> f64 {
        RK_A * R.powi(2) * self.tc.powf(2.5) / self.pc
    }
    fn b_const(&self) -> f64 {
        RK_B * R * self.tc / self.pc
    }
}
