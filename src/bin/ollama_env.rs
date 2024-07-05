use libcnb::data::exec_d_program_output_key;
use libcnb::exec_d::write_exec_d_program_output;
use std::collections::HashMap;

fn main() {
    let ollama_host = std::env::var("PORT")
        .map(|port| format!("0.0.0.0:{port}"))
        .unwrap_or(String::from("0.0.0.0"));

    write_exec_d_program_output(HashMap::from([(
        exec_d_program_output_key!("OLLAMA_HOST"),
        ollama_host,
    )]))
}
