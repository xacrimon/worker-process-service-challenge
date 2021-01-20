# worker-process-service-challenge

This is a prototype implementation of a service that allows running arbitrary processes and a client
that can interface with that service remotely over gRPC.

The design document can be found [here](https://docs.google.com/document/d/1y4gG4gp-Tg695jdfXQGGm_pQd4VMT1-ZNO06QolkzmQ/edit?usp=sharing).

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

## Certificates and Keys

For the purpose of demonstration, I've included all certificates and keys for the client, server and their CAs as hardcoded files in the `data` folder.

## Tests

A handful of tests are available and are all implemented in the client package. To run them,
simply enter the root repository directory and execute `cargo test`. The test code itself is located
in the `client/src/tests` directory. Current tests cover TLS, authorization and basic usage.

## Usage

First, an instance of the server itself needs to be running. This is as simple as
compiling the server binary and running it without any arguments. If everything works
as intended you should see something similar to this in your console. Please note that all child processes
are run with the user and permissions as the user this service is started under.

```
serving gRPC endpoint at 0.0.0.0:7005
```

You're then ready to connect to it with the client.
The client has a few base parameters that will need to be met for all subcommands
and then each subcommand has it's own set of required parameters. The CLI itself has some decent documentation
accessible via the `--help` parameter which may follow the main command or any subcommand.

Some examples are provided below.

### Spawning a job

```
./client --endpoint https://localhost:7005 --username acrimon spawn --program-path /usr/bin/echo --args hi,man --envs TESTENV=ENVVALUE --working-directory /sys
```

### Stopping a job

```
./client --endpoint https://localhost:7005 --username acrimon stop --uuid <uuid>
```

### Fetch the status for a job

```
./client --endpoint https://localhost:7005 --username acrimon status --uuid <uuid>
```

### Stream all past and future output events from a job

```
./client --endpoint https://localhost:7005 --username acrimon stream-log --stream-type <stream-type> --past-events --uuid <uuid>
```

Valid values for `stream-type` are
- `raw`
- `stdout`
- `stderr`
