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

## Certificates and Keys

For the purpose of demonstration, I've included all certificates and keys for the client, server and their CAs as hardcoded files in the `data` folder.

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
and then each subcommand has it's own set of required parameters. The CLI itself
has fairly extensive documentation on how it all works but some examples will be provided here.

The CLI comes with a root command and a number of subcommands. When in doubt, simply run the command with `--help`, also works for all subcommands.

### Spawning a job

```
./client -d localhost -e https://localhost:7005 -u acrimon spawn -p /usr/bin/echo -a hi -e TESTENV=ENVVALUE -w /sys
```

### Stopping a job

```
./client -d localhost -e https://localhost:7005 -u acrimon stop -u <uuid>
```
