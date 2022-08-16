pub struct Config {
    pub transactions_path: String,
}

impl Config {
    pub fn new(args: &[String]) -> Config {
        if args.len() < 2 {
            panic!("No transactions file provided, please specify a transaction file.")
        }

        let transactions_path = args[1].clone();

        Config { transactions_path }
    }
}
