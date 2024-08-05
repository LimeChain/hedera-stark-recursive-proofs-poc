use clap::{Arg, ArgAction, Command};

pub fn initialize() -> Command {
    Command::new("Recursive Proof POC")
        .version("1.0")
        .about("Hedera STARK recursive proof")
        .arg(
            Arg::new("pubkey0")
                .long("pubkey0")
                .value_name("PUBKEY0")
                .help("The pubkey for epoch 0")
                .required_unless_present("demo"),
        )
        .arg(
            Arg::new("receipt")
                .long("receipt")
                .value_name("RECEIPT")
                .help("The path to the assumption receipt")
                .required(false),
        )
        .arg(
            Arg::new("demo")
                .long("demo")
                .help("Run the demo workflow")
                .action(ArgAction::SetTrue),
        )
}
