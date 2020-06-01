use std::fs::File;
use std::io::{BufReader, BufRead};
use crate::query::gen_subdomain;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Dict {
    dicts: Vec<String>
}

impl Dict {
    pub fn new(dict_file: &str) -> Self {
        let f = File::open(dict_file).unwrap();
        Self {
            dicts: read_file(f)
        }
    }

    pub fn is_exist(&self, item: &String) -> bool {
        self.dicts.contains(item)
    }

    pub fn get_dict(self) -> Vec<String> {
        self.dicts
    }

    pub fn len(&self) -> usize {
        self.dicts.len()
    }
}

fn read_file(f: File) -> Vec<String> {
    let mut dict = Vec::new();
    let reader = BufReader::new(f);
    for line in reader.lines() {
        match line {
            Ok(line) => {
                let line = line.trim().to_lowercase();
                if gen_subdomain(line.as_str(), ".1.com").is_some() {
                    dict.push(line)
                }
            }
            Err(e) => {
                warn!("[dict] read_file msg: {:?}", e.kind())
            }
        }
    }

    dict.sort();
    dict.dedup();
    dict

}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_is_exist() {
        let mut d = Dict::new("depth.txt");
        assert_eq!(d.is_exist(&"www".to_string()), false);
        assert_eq!(d.is_exist(&"api".to_string()), true);
        assert_eq!(d.is_exist(&"search".to_string()), false);
    }
}
