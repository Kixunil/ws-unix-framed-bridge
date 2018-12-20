extern crate configure_me_codegen;

fn main() {
    configure_me_codegen::build_script_with_man_written_to("config_spec.toml", "target/ws-unix-framed-bridge.1").expect("Failed to generate configuration loading code.");
}
