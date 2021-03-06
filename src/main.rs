extern crate clap;

use clap::{App, AppSettings, Arg, SubCommand};
use std::cell::Cell;
use std::fmt::Display;
use std::str::FromStr;

mod birthday_attack;
mod overflow_attack;
mod tree_attack;

struct WordValidator {
    len: Cell<Option<usize>>,
}

impl WordValidator {
    fn new() -> Box<dyn Fn(String) -> Result<(), String>> {
        let validator = WordValidator {
            len: Cell::new(None),
        };
        Box::new(move |w| validator.validate(w))
    }

    fn validate(&self, s: String) -> Result<(), String> {
        match self.len.get() {
            Some(l) => {
                if s.len() != l {
                    Err("words of the alphabet must have the same length".to_string())
                } else {
                    Ok(())
                }
            }
            None => {
                self.len.set(Some(s.len()));
                Ok(())
            }
        }
    }
}

fn is_valid<T>(s: String) -> Result<(), String>
where
    T: FromStr,
    T::Err: Display,
{
    match s.parse::<T>() {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("{}", e)),
    }
}

fn main() {
    let matches = App::new("antihash")
        .version("1.0.0")
        .author("Alessandro Bortolin <bortolin.alessandro@outlook.it>")
        .about("Find antihash testcases")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .setting(AppSettings::DisableHelpSubcommand)
        .setting(AppSettings::VersionlessSubcommands)
        .arg(
            Arg::with_name("reverse")
                .short("r")
                .long("reverse")
                .help("Print reversed strings")
                .display_order(100),
        )
        .arg(
            Arg::with_name("uppercase")
                .short("u")
                .long("uppercase")
                .help("Print uppercase strings")
                .display_order(100),
        )
        .subcommand(
            SubCommand::with_name("overflow")
                .about("Overflow attack")
                .arg(
                    Arg::with_name("length")
                        .short("l")
                        .long("length")
                        .help("Minimum length of strings")
                        .takes_value(true)
                        .value_name("LENGTH")
                        .default_value("1024")
                        .validator(is_valid::<usize>),
                ),
        )
        .subcommand(
            SubCommand::with_name("birthday")
                .about("Birthday attack")
                .arg(
                    Arg::with_name("coefficients")
                        .help("Base and module of hash")
                        .required(true)
                        .takes_value(true)
                        .number_of_values(2)
                        .multiple(true)
                        .value_names(&["BASE", "MODULE"])
                        .validator(is_valid::<u32>),
                )
                .arg(
                    Arg::with_name("alphabet")
                        .help("String used as alphabet to build collision")
                        .takes_value(true)
                        .last(true)
                        .multiple(true)
                        .value_name("WORDS")
                        .validator(WordValidator::new()),
                ),
        )
        .subcommand(
            SubCommand::with_name("tree")
                .about("Tree attack")
                .arg(
                    Arg::with_name("coefficients")
                        .help("Base and module of hash")
                        .required(true)
                        .takes_value(true)
                        .number_of_values(2)
                        .multiple(true)
                        .validator(is_valid::<u64>)
                        .value_names(&["BASE", "MODULE"]),
                )
                .arg(
                    Arg::with_name("alphabet")
                        .help("String used as alphabet to build collision")
                        .takes_value(true)
                        .last(true)
                        .multiple(true)
                        .value_name("WORDS")
                        .validator(WordValidator::new()),
                )
                .arg(
                    Arg::with_name("cluster")
                        .short("c")
                        .long("cluster")
                        .help("Size of clusters")
                        .takes_value(true)
                        .value_name("SIZE")
                        .default_value("100000")
                        .validator(is_valid::<usize>),
                ),
        )
        .get_matches();

    let coll = match matches.subcommand() {
        ("overflow", Some(submatches)) => {
            let len = submatches.value_of("length").unwrap().parse::<usize>().unwrap().next_power_of_two();
            overflow_attack::find_collision(len)
        }
        ("birthday", Some(submatches)) => {
            let mut bases = Vec::new();
            let mut modules = Vec::new();
            for (index, coefficient) in submatches.values_of("coefficients").unwrap().enumerate() {
                if index % 2 == 0 {
                    bases.push(coefficient.parse().unwrap());
                } else {
                    modules.push(coefficient.parse().unwrap());
                }
            }
            let alphabet = match submatches.values_of("alphabet") {
                Some(a) => a.map(|s| s.to_string()).collect(),
                None => (0..26)
                    .map(|i| std::char::from_u32(i + 97).unwrap().to_string())
                    .collect(),
            };
            birthday_attack::find_collision(bases, modules, alphabet)
        }
        ("tree", Some(submatches)) => {
            let mut bases = Vec::new();
            let mut modules = Vec::new();
            for (index, coefficient) in submatches.values_of("coefficients").unwrap().enumerate() {
                if index % 2 == 0 {
                    bases.push(coefficient.parse().unwrap());
                } else {
                    modules.push(coefficient.parse().unwrap());
                }
            }
            let cluster_size = submatches.value_of("cluster").unwrap().parse().unwrap();
            let alphabet = match submatches.values_of("alphabet") {
                Some(a) => a.map(|s| s.to_string()).collect(),
                None => (0..26)
                    .map(|i| std::char::from_u32(i + 97).unwrap().to_string())
                    .collect(),
            };
            tree_attack::find_collision(bases, modules, cluster_size, alphabet)
        }
        _ => None,
    };

    if let Some((mut fi, mut se)) = coll {
        if matches.is_present("reverse") {
            fi = fi.chars().rev().collect::<String>();
            se = se.chars().rev().collect::<String>();
        }
        if matches.is_present("uppercase") {
            fi.make_ascii_uppercase();
            se.make_ascii_uppercase();
        }

        println!("{}\n{}", fi, se);
    } else {
        println!("Collision not found");
    }
}

#[test]
fn overflow_attack() {
    let (s1, s2) = overflow_attack::find_collision(1024).expect("collision not found");
    let (mut h1, mut h2) = (0u64, 0u64);
    let base = 9973;
    for c1 in s1.chars() {
        h1 = h1.wrapping_mul(base).wrapping_add(c1 as u64);
    }
    for c2 in s2.chars() {
        h2 = h2.wrapping_mul(base).wrapping_add(c2 as u64);
    }
    assert!(h1 == h2, "hashes are different");
}

#[test]
fn birthday_attack() {
    let base = 9973;
    let module = 1000000007;
    let alphabet = (0..26)
        .map(|i| std::char::from_u32(i + 97).unwrap().to_string())
        .collect();
    let (s1, s2) = birthday_attack::find_collision(vec![base], vec![module], alphabet).expect("collision not found");
    let (mut h1, mut h2) = (0u64, 0u64);
    for c1 in s1.chars() {
        h1 = (h1 * base + c1 as u64) % module;
    }
    for c2 in s2.chars() {
        h2 = (h2 * base + c2 as u64) % module;
    }
    assert!(h1 == h2, "hashes are different");
}

#[test]
fn birthday_attack_multiple() {
    let bases = vec![9973, 11173];
    let modules = vec![1000000007, 1000000009];
    let alphabet = (0..26)
        .map(|i| std::char::from_u32(i + 97).unwrap().to_string())
        .collect();
    let (s1, s2) = birthday_attack::find_collision(bases.clone(), modules.clone(), alphabet).expect("collision not found");
    for (&b, &m) in bases.iter().zip(modules.iter()) {
        let (mut h1, mut h2) = (0u64, 0u64);
        for c1 in s1.chars() {
            h1 = (h1 * b + c1 as u64) % m;
        }
        for c2 in s2.chars() {
            h2 = (h2 * b + c2 as u64) % m;
        }
        assert!(h1 == h2, "hashes are different");
    }
}

#[test]
fn birthday_attack_alphabet() {
    let base = 9973;
    let module = 1000000007;
    let alphabet = vec!["xcphdx".to_string(), "fsngso".to_string()];
    let (s1, s2) = birthday_attack::find_collision(vec![base], vec![module], alphabet).expect("collision not found");
    let (mut h1, mut h2) = (0u64, 0u64);
    for c1 in s1.chars() {
        h1 = (h1 * base + c1 as u64) % module;
    }
    for c2 in s2.chars() {
        h2 = (h2 * base + c2 as u64) % module;
    }
    assert!(h1 == h2, "hashes are different");
}

#[test]
fn tree_attack() {
    let base = 9973;
    let module = 1000000007;
    let alphabet = (0..26)
        .map(|i| std::char::from_u32(i + 97).unwrap().to_string())
        .collect();
    let (s1, s2) = tree_attack::find_collision(vec![base], vec![module], 100000, alphabet).expect("collision not found");
    let (mut h1, mut h2) = (0u64, 0u64);
    for c1 in s1.chars() {
        h1 = (h1 * base + c1 as u64) % module;
    }
    for c2 in s2.chars() {
        h2 = (h2 * base + c2 as u64) % module;
    }
    assert!(h1 == h2, "hashes are different");
}

#[test]
fn tree_attack_multiple() {
    let bases = vec![9973, 11173];
    let modules = vec![1000000007, 1000000009];
    let alphabet = (0..26)
        .map(|i| std::char::from_u32(i + 97).unwrap().to_string())
        .collect();
    let (s1, s2) = tree_attack::find_collision(bases.clone(), modules.clone(), 100000, alphabet).expect("collision not found");
    for (&b, &m) in bases.iter().zip(modules.iter()) {
        let (mut h1, mut h2) = (0u64, 0u64);
        for c1 in s1.chars() {
            h1 = (h1 * b + c1 as u64) % m;
        }
        for c2 in s2.chars() {
            h2 = (h2 * b + c2 as u64) % m;
        }
        assert!(h1 == h2, "hashes are different");
    }
}

#[test]
fn tree_attack_alphabet() {
    let base = 9973;
    let module = 1000000007;
    let alphabet = vec!["xcphdx".to_string(), "fsngso".to_string()];
    let (s1, s2) = tree_attack::find_collision(vec![base], vec![module], 100000, alphabet).expect("collision not found");
    let (mut h1, mut h2) = (0u64, 0u64);
    for c1 in s1.chars() {
        h1 = (h1 * base + c1 as u64) % module;
    }
    for c2 in s2.chars() {
        h2 = (h2 * base + c2 as u64) % module;
    }
    assert!(h1 == h2, "hashes are different");
}
