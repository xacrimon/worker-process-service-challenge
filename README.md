# worker-process-service-challenge

This is a prototype implementation of a service that allows running arbitrary processes and a client
that can interface with that service remotely over gRPC.

## Coding Style

This project follows the somewhat old [Rust Style Guidelines](https://doc.rust-lang.org/1.0.0/style/README.html) by default
and the formatting defaults of [rustfmt](https://github.com/rust-lang/rustfmt). rustfmt defaults take place over style guidelines
should they conflict due to it being newer but is not provided in a written book form.

The project also enforces automatic linting with [clippy](https://github.com/rust-lang/rust-clippy) and will fail on any warning.

CI is configured to fail if the code is not properly formatted or any lints trigger.
