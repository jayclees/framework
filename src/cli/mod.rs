use regex::Regex;
use std::collections::HashMap;
use std::fmt::Formatter;
use std::rc::Rc;

pub struct Registry {
    inner: Rc<HashMap<&'static str, OptionDefinition>>,
}

impl Registry {
    pub fn default() -> Registry {
        let mut args = HashMap::new();
        args.insert(
            "--host",
            OptionDefinition::new(
                "--host",
                None,
                "The host the server will run on. Default: 127.0.0.1",
                Some("127.0.0.1"),
                false,
            ),
        );
        args.insert(
            "--port",
            OptionDefinition::new(
                "--host",
                Some("-p"),
                "The port the server will listen to. Default: 3000",
                Some("3000"),
                false,
            ),
        );
        args.insert(
            "--help",
            OptionDefinition::new("--help", Some("-h"), "List run options.", None, true),
        );

        Registry {
            inner: Rc::new(args),
        }
    }

    pub fn parse(&self, args: Vec<String>) -> Result<Parsed, InvalidOption> {
        let mut parsed: HashMap<String, Value> = HashMap::new();

        for arg in args {
            let regex = Regex::new("^--?[a-zA-Z][a-zA-Z0-9]+(?:=.{1,}+)?$").unwrap();
            if !regex.is_match(arg.as_str()) {
                return Err(InvalidOption::new(arg.clone(), format!(r#"Invalid option "{arg}"."#)));
            }

            let split = arg.split("=").collect::<Vec<&str>>();
            if split.len() == 2 {
                let mut iter = split.iter();
                let option = iter.next().unwrap();
                let value = iter.next().unwrap();

                match self.inner.get(option) {
                    Some(option_type) => {
                        if option_type.is_flag {
                            return Err(InvalidOption::new(
                                arg.to_string(),
                                format!("Option {option} is a flag and cannot accept a value."),
                            ));
                        }

                        parsed.insert(
                            option.replace("--", "").to_owned(),
                            Value::String(value.to_string()),
                        );
                    }
                    None => {
                        return Err(InvalidOption::new(
                            arg.to_string(),
                            format!(r#"Invalid option "{option}""#),
                        ));
                    }
                }
            } else if split.len() == 1 {
                let mut iter = split.iter();
                let option = iter.next().unwrap();

                match self.inner.get(option) {
                    Some(option_type) => {
                        if !option_type.is_flag {
                            return Err(InvalidOption::new(
                                arg.to_string(),
                                format!("Option {} requires a value", option),
                            ));
                        }
                        parsed.insert(option.replace("--", "").to_owned(), Value::Flag(true));
                    }
                    None => {
                        // Option passed by user is not in the registered list.
                        return Err(InvalidOption::new(
                            arg.to_string(),
                            format!(r#"Invalid option "{option}""#),
                        ));
                    }
                }

                // todo handle shorthand flags -p3000 == --port=3000
            }
        }

        Ok(Parsed::new(parsed))
    }

    pub fn print_help(&self) {
        println!("Options:");
        for (handle, arg) in self.inner.iter() {
            println!("    {} {}", handle, arg.description);
        }
        println!("Exiting...");
    }

    pub fn eprint_help(&self, err_msg: String) {
        eprintln!("{}", err_msg.as_str());
        eprintln!("Options:");
        for (handle, arg) in self.inner.iter() {
            eprintln!("    {} {}", handle, arg.description);
        }
    }
}

#[derive(Debug)]
pub struct Parsed {
    inner: HashMap<String, Value>,
}

impl Parsed {
    pub fn new(parsed: HashMap<String, Value>) -> Parsed {
        Parsed { inner: parsed }
    }

    pub fn help_requested(&self) -> bool {
        self.inner.contains_key("help")
    }

    pub fn host(&self) -> String {
        if let Value::String(host) = self.inner.get("host").unwrap() {
            host.clone()
        } else {
            "127.0.0.1".to_string()
        }
    }

    pub fn port(&self) -> String {
        if let Value::String(port) = self.inner.get("port").unwrap() {
            port.clone()
        } else {
            "3000".to_string()
        }
    }
}

impl std::fmt::Display for Parsed {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut generated = vec![];
        for (k, v) in &self.inner {
            generated.push(match v {
                Value::String(v) => {
                    format!("{}={}", k, v)
                }
                Value::Flag(_) => {
                    format!("{k}")
                }
            });
        }
        write!(f, "{}", generated.join(", "))
    }
}

#[derive(Debug)]
pub struct OptionDefinition {
    signature: &'static str,
    signature_short: Option<&'static str>,
    description: &'static str,
    default: Option<&'static str>, // todo if option is required and does not have default, validate it
    is_flag: bool,
}

impl OptionDefinition {
    pub fn new(
        signature: &'static str,
        signature_short: Option<&'static str>,
        description: &'static str,
        default: Option<&'static str>,
        is_flag: bool,
    ) -> OptionDefinition {
        OptionDefinition {
            description,
            signature,
            signature_short,
            default,
            is_flag,
        }
    }
}

#[derive(Debug)]
pub enum Value {
    String(String),
    Flag(bool),
}

#[derive(Debug)]
pub struct InvalidOption {
    signature: String,
    msg: String,
}

impl InvalidOption {
    pub fn new(signature: String, msg: String) -> InvalidOption {
        InvalidOption { signature, msg }
    }

    pub fn msg(&self) -> &String {
        &self.msg
    }

    pub fn signature(&self) -> &String {
        &self.signature
    }
}

impl std::fmt::Display for InvalidOption {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, r#""{}": {}"#, self.signature, self.msg)
    }
}

impl std::error::Error for InvalidOption {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_malformed_option() {
        // missing leading dashes
        let args = vec!["malformed".to_string()];
        match Registry::default().parse(args) {
            Ok(parsed) => {
                assert!(false, "Should have failed but didn't.\nResult: {}", parsed)
            }
            Err(error) => {
                assert_eq!(r#"Invalid option "malformed"."#, error.msg());
            }
        };
    }

    #[test]
    fn validate_non_existent() {
        let args = vec!["--invalidoption=value".to_string()];
        match Registry::default().parse(args) {
            Ok(parsed) => {
                assert!(false, "Should have failed but didn't.\nResult: {}", parsed)
            }
            Err(error) => {
                assert_eq!(r#"Invalid option "--invalidoption""#, error.msg());
            }
        };
    }

    #[test]
    fn validate_required_value() {
        let args = vec!["--host".to_string()];
        match Registry::default().parse(args) {
            Ok(parsed) => {
                assert!(false, "Should have failed but didn't.\nResult: {}", parsed)
            }
            Err(error) => {
                assert_eq!("Option --host requires a value", error.msg());
            }
        };
    }

    #[test]
    fn validate_flag_requires_no_value() {
        let args = vec!["--help=invalidvalue".to_string()];
        match Registry::default().parse(args) {
            Ok(parsed) => {
                assert!(false, "Should have failed but didn't.\nResult: {}", parsed)
            }
            Err(error) => {
                assert_eq!("Option --help is a flag and cannot accept a value.", error.msg());
            }
        };
    }
}
