pub struct PureGas {
    /// Identifier of the gas
    pub id: &'static str,
    /// Name of the gas
    pub name: &'static str,
    /// Critical temperature in K
    pub tc: f64,
    /// Critical pressure in Pa
    pub pc: f64,
}

pub static GASES: &[PureGas] = &[
    PureGas {
        id: "H2",
        name: "Hydrogen",
        tc: 33f64,
        pc: 1290000f64,
    },
    PureGas {
        id: "N2",
        name: "Nitrogen",
        tc: 126.2f64,
        pc: 3390000f64,
    },
    PureGas {
        id: "O2",
        name: "Oxygen",
        tc: 154.6f64,
        pc: 5040000f64,
    },
    PureGas {
        id: "CO2",
        name: "Carbon dioxide",
        tc: 31.3f64 + 273.15f64,
        pc: 72.9f64 * 101325f64,
    },
];
