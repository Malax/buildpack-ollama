use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub(crate) fn base_model_from_modelfile(path: &Path) -> Result<Option<String>, std::io::Error> {
    File::open(path).map(BufReader::new).map(|reader| {
        reader.lines().find_map(|line| {
            line.ok().and_then(|line| {
                line.split_once(' ').and_then(|(instruction, arguments)| {
                    instruction
                        .eq_ignore_ascii_case("FROM")
                        .then_some(String::from(arguments))
                })
            })
        })
    })
}
