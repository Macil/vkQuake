#[cfg(not(test))]
pub fn init() {
    use crate::{adapters::console::quake_print, quake_println};
    use tracing::metadata::LevelFilter;
    use tracing_subscriber::{prelude::*, EnvFilter, Registry};

    struct QuakeWriter;

    impl std::io::Write for QuakeWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            let message = std::str::from_utf8(buf).unwrap_or("QuakeWriter failure: Invalid UTF-8");
            quake_print(message);
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    // Log tracing messages to stdout with default formatting and
    // to the Quake console with more compact formatting.
    // Respect the [RUST_LOG environment
    // variable](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/struct.EnvFilter.html#example-syntax)
    // if present, otherwise log INFO level and higher.

    let stdout_log = tracing_subscriber::fmt::layer();
    let subscriber = Registry::default().with(stdout_log);

    let quake_console_layer = tracing_subscriber::fmt::layer()
        .without_time()
        .with_ansi(false)
        .with_writer(|| QuakeWriter);

    let subscriber = subscriber.with(quake_console_layer).with(
        EnvFilter::builder()
            .with_default_directive(LevelFilter::INFO.into())
            .from_env_lossy(),
    );
    tracing::subscriber::set_global_default(subscriber).expect("Unable to set global subscriber");

    // Log to the Quake console in addition to the default panic handling.
    // Use the default panic handler to print the panic message to the console instead
    // of going through tracing so we get all the default behavior and formatting in stdout.
    // Force abort on panic because Tokio will ignore them by default.
    let default_panic = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        quake_println!("{}", panic_info);
        default_panic(panic_info);
        std::process::abort();
    }));
}

#[cfg(test)]
pub fn init() {
    tracing_subscriber::fmt::init();
}
