# worker-process-service-challenge

This is a prototype implementation of a service that allows running arbitrary processes and a client
that can interface with that service remotely over gRPC.

## Coding Style

This project roughly follows the somewhat old [Rust Style Guidelines](https://doc.rust-lang.org/1.0.0/style/README.html) by default
and the formatting defaults of [rustfmt](https://github.com/rust-lang/rustfmt). rustfmt defaults take place over style guidelines
should they conflict due to it being newer but is not provided in a written book form.

The project also enforces automatic linting with [clippy](https://github.com/rust-lang/rust-clippy) and will fail on any warning.

CI is configured to fail if the code is not properly formatted or any lints trigger.

## Usernames and Identifiers

In a production grade product this should probably rely on something other
than usernames for unique user identification. You would probably have some sort of service
that associates usernames with UUIDs and that can be used to refer to users internally instead.
For simplicity this prototype just sticks with usernames.

## Errors

In a production grade product, errors would ideally be handled by handwriting enums and serializing that to error
codes over gRPC. Due to time and code size I've opted to simply use human-readable text errors instead.
