#[cfg(test)]
mod parser_test {
    use crate::parser;
    use regex::Regex;
    use std::env;

    #[test]
    fn test_parser() {
        std::env::set_var("TEST", ".");
        std::env::var("TEST").unwrap();
        let (mut config, color_schemes) = parser::find_alacritty_configs().unwrap();

        let schemes = match color_schemes.get_available_schemes() {
            Ok(s) => s,
            Err(e) => panic!("{}", e)
        };

        let _current_scheme = match config.get_current_scheme_name() {
            Ok(s) => s,
            Err(e) => panic!("{}", e)
        };

        let new_scheme = match color_schemes.get_scheme("srcery") {
            Ok(s) => s,
            Err(e) => panic!("{}", e)
        };

        let new_scheme_re = Regex::new(r"\bcolors: \*srcery").unwrap();
        config.set_scheme(&new_scheme);

        assert!(new_scheme_re.is_match(&config.data));
    }
}
