Redlich-Kwong Z-factor calculator 0.1.0
Rémi Thebault <remi.thebault@gmail.com>
Computes compression (Z) factor of several gases in conditions of pressure and temperature. If a range is provided for
pressure or temperature, the result is written in CSV format for each element of both range (1 row per pressure
condition, 1 column per temperature condition)

USAGE:
    rkz [FLAGS] [OPTIONS]

FLAGS:
    -h, --help        Prints help information
        --list-gas    Print a list of referenced gases
    -V, --version     Prints version information

OPTIONS:
    -g, --gas <gas>                    Specify the gas by id. (--list-gas to show referenced gases)
    -P, --pressure <pressure>          Specify the abs. pressure in bar. A range can be specified in the form of
                                       start:stop[:step].
    -T, --temperature <temperature>    Specify the temperature in °C. A range can be specified in the form of
                                       start:stop[:step].
