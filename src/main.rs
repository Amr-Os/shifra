use shifra::{InterpreterBuilder, InterpreterBuilderExt};

pub fn main() -> std::process::ExitCode {
    let mut config = InterpreterBuilder::new();
    #[cfg(feature = "stdlib")]
    {
        config = config.init_stdlib();
    }
    shifra::run(config)
}
