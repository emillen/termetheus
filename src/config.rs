pub struct Config {
    base_url: String,
    query: String,
}

impl Config {
    pub fn new(args: &[String]) -> Result<Config, &'static str> {
        if args.len() != 3 {
            return Err("wrong args");
        }
        let base_url = args[1].clone();
        let query = args[2].clone();
        Ok(Config { base_url, query })
    }

    pub fn base_url(&self) -> &String {
        &self.base_url
    }

    pub fn query(&self) -> &String {
        &self.query
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_new() {
        let args = vec![
            String::from("termetheus"),
            String::from("http://localhost:9090"),
            String::from("up"),
        ];
        let config = Config::new(&args).unwrap();
        assert_eq!(config.base_url(), "http://localhost:9090");
        assert_eq!(config.query(), "up");
    }

    #[test]
    fn config_new_not_enough_args() {
        let args = vec![String::from("termetheus")];
        let config = Config::new(&args);
        assert!(config.is_err());
    }

    #[test]
    fn config_new_too_many_args() {
        let args = vec![
            String::from("termetheus"),
            String::from("http://localhost:9090"),
            String::from("up"),
            String::from("one_too_many"),
        ];
        let config = Config::new(&args);
        assert!(config.is_err());
    }
}
