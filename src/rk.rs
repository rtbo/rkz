use super::gas::{Gas, GasComp, GasMixture, PureGas};
use roots::{self, Roots};

const R: f64 = 8.31446262;

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
        let a1 = a - b * b - b;
        let a0 = -a * b;
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

impl RkGas for GasMixture {
    fn a_const(&self) -> f64 {
        let mut res = 0f64;
        for i in self.comps.iter() {
            let ai = i.pure_gas().a_const();
            for j in self.comps.iter() {
                let aj = j.pure_gas().a_const();
                res += i.molar_fraction() * j.molar_fraction() * (ai * aj).sqrt();
            }
        }
        res
    }

    fn b_const(&self) -> f64 {
        let mut res = 0f64;
        for i in self.comps.iter() {
            res += i.molar_fraction() * i.pure_gas().b_const();
        }
        res
    }
}

impl RkGas for Gas {
    fn a_const(&self) -> f64 {
        match self {
            Gas::Pure(g) => g.a_const(),
            Gas::Mixture(g) => g.a_const(),
        }
    }
    fn b_const(&self) -> f64 {
        match self {
            Gas::Pure(g) => g.b_const(),
            Gas::Mixture(g) => g.b_const(),
        }
    }
}
