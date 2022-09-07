rkz 1.1.0
Rémi Thebault <remi.thebault@gmail.com>

Computes compression factor of several gases and mixtures in conditions of
pressure and temperature using the either of the following equations of state:
  - Van der Waals
  - Redlich-Kwong (default)
  - Soave-Redlich-Kwong
  - Peng-Robinson

                                                           PV
The compression factor for a mole of gas is defined as Z = --.
                                                           RT

A range can be provided instead of scalar values for pressure or temperature. In
such case, the result is written in CSV format with one Z value per combination
of pressure and temperature (1 row per pressure condition, 1 column per
temperature condition).
Range are provided in the form of min:max[:step] (e.g. '20:800' or '20:800:10').
If step is omitted, it is assumed to be equal to one.

Mixture for option --gas|-g can be specified in the form of
molar_fraction%gas_id+[molar_fraction%gas_id]. Mixture molar fractions can only
be specified as percentage or be omitted. Gases without molar fraction evenly
take the rest of the mixture. Examples:
  - '80%N2+20%O2' => 80% Nitrogen and 20% Oxygen
  - '80%N2+O2' => 80% Nitrogen and 20% Oxygen
  - '80%N2+O2+CO2' => 80% Nitrogen, 10% Oxygen and 10% Carbon dioxide
  - '78%N2+21%O2+Ar' => air composition (more or less)
  - 'N2+O2' => 50% Nitrogen and 50% Oxygen

DISCLAIMER: rkz is provided "as is" without any warranty. See the --license
option for details.

USAGE:
    rkz [FLAGS] [OPTIONS]

FLAGS:
    -h, --help        Prints help information
        --license     Prints the license text and exits
        --list-gas    Prints a list of referenced gases
    -V, --version     Prints version information

OPTIONS:
    -e, --eos <equation>
            Specify the equation of state (case insensitive). Choices are VdW
            for Van-der-Waals, RK for Redlich-Kwong, SRK for Soave-Redlich-Kwong
            and PR for Peng-Robinson. [default: RK]
    -g, --gas <gas>
            Specify the gas by id or by mixture spec (see above)

    -p, --pressure <pressure>
            Specify the pressure in bar. By default absolute unless --relative
            is used. A range can be specified in the form of start:stop[:step].
    -r, --relative <relative>
            Specify that the pressure is relative to the pressure indicated in
            this parameter (in hPa). "stdatm" can be used for 1013.25.
    -t, --temperature <temperature>
            Specify the temperature in °C. A range can be specified in the form
            of start:stop[:step].

EXAMPLES:
    rkz --list-gas
        Print a list of all gases referenced in RKZ
    rkz -g N2 -p 200 -t 20
        Z-factor of Nitrogen at 200bar and 20°C
    rkz -g 78%N2+21%O2+Ar -p 200 -t 50 -e PR
        Z-factor of air at 200bar and 50°C with Peng-Robinson equation of state
    rkz -g H2 -p 0:1000:10 -t -40:80 -r stdatm
        Z-factor CSV table of Hydrogen from 0 to 1000barG and -40 to +80°C
