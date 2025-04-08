# RSendMail
Used for batch sending emails for testing

English | [简体中文](README_zh.md)

## Features

- Batch process and send multiple emails
- Multi-threaded processing for improved performance
- Support for custom SMTP server configuration
- Detailed logging with configurable levels (error/warn/info/debug/trace)
- Comprehensive error tracking and statistics
- Docker support for easy deployment
- Support for batch sending in single SMTP session

## Building

### Local Build
```bash
cd rsendmail
cargo build --release
```

### Docker Build
```bash
docker build -t rsendmail .
```

## Usage

### Windows Usage
Download the Windows executable (`rsendmail-windows-x86_64.exe`) from the [Releases](https://github.com/kpassy/RSendMail/releases) page.
```
rsendmail-windows-x86_64.exe --smtp-server <smtp_server> --port <port> --from <from_addr> --to <to_addr> --dir <email_dir> --processes <processes> --batch-size <batch_size>
```

### Local Usage
```bash
rsendmail --smtp-server <smtp_server> --port <port> --from <from_addr> --to <to_addr> --dir <email_dir> --processes <processes> --batch-size <batch_size>
```

### Docker Usage
```bash
docker run --rm -v /path/to/emails:/data rsendmail --smtp-server <smtp_server> --port <port> --from <from_addr> --to <to_addr> --dir /data --processes <processes> --batch-size <batch_size>
```

### Parameters

- `--smtp-server`: SMTP server address
- `--port`: SMTP server port (default: 25)
- `--from`: Sender email address
- `--to`: Recipient email address
- `--dir`: Email files directory
- `--extension`: Email file extension (default: eml)
- `--processes`: Number of processes, "auto" for CPU core count or specify a number (default: auto)
- `--batch-size`: Number of emails to send in a single SMTP session (default: 1)
- `--smtp-timeout`: SMTP session timeout in seconds (default: 30)
- `--log-level`: Log level (error/warn/info/debug/trace) (default: info)
- `--keep-headers`: Keep original email headers (default: false)
- `--anonymize-emails`: Anonymize all email addresses (default: false)
- `--anonymize-domain`: Domain to use for anonymized emails (default: example.com)

## Example

```bash
# Local example
# Normal logging (info level)
rsendmail --smtp-server 192.168.1.100 --port 25 --from sender@example.com --to recipient@example.com --dir ./emails --processes 10 --batch-size 5

# Detailed debug logging
rsendmail --smtp-server 192.168.1.100 --port 25 --from sender@example.com --to recipient@example.com --dir ./emails --processes 10 --batch-size 5 --log-level debug

# Docker example
docker run --rm -v $(pwd)/emails:/data rsendmail --smtp-server 192.168.1.100 --port 25 --from sender@example.com --to recipient@example.com --dir /data --processes 10 --batch-size 5
```

## Docker Container

The Docker container is designed with security and efficiency in mind:

- Based on debian:bookworm-slim for minimal size
- Runs as non-root user
- Includes only necessary runtime dependencies
- Uses volume mounting for email files
- Stateless operation

### Container Structure

- `/usr/local/bin/rsendmail`: Application binary
- `/data`: Mount point for email files (volume)

## Performance

- Multi-threaded processing
- Efficient memory usage
- Fast email parsing and sending
- Detailed performance statistics output
- Support for batch sending in single SMTP session for improved efficiency

## Security

- Non-root user execution
- Minimal container footprint
- Isolated runtime environment
- No persistent storage
