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

