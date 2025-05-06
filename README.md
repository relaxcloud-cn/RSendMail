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
```bash
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
- `--from`: Sender email address (for SMTP envelope, doesn't modify message content by default)
- `--to`: Recipient email address (for SMTP envelope, doesn't modify message content by default)
- `--dir`: Email files directory
- `--extension`: Email file extension (default: eml)
- `--processes`: Number of processes, "auto" for CPU core count or specify a number (default: auto)
- `--batch-size`: Number of emails to send in a single SMTP session (default: 1)
- `--smtp-timeout`: SMTP session timeout in seconds (default: 30)
- `--log-level`: Log level (error/warn/info/debug/trace) (default: info)
- `--keep-headers`: Keep original email headers (default: false, takes precedence over modify-headers)
- `--modify-headers`: Use --from and --to to modify From and To headers in message content (default: false)
- `--anonymize-emails`: Anonymize all email addresses (default: false)
- `--anonymize-domain`: Domain to use for anonymized emails (default: example.com)
- `--loop`: Loop indefinitely until interrupted (default: false)
- `--repeat`: Number of times to repeat sending (default: 1)
- `--loop-interval`: Interval between sending loops in seconds (default: 1)
- `--retry-interval`: Interval before retrying after failure in seconds (default: 5)

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

![CodeRabbit Pull Request Reviews](https://img.shields.io/coderabbit/prs/github/relaxcloud-cn/RSendMail?utm_source=oss&utm_medium=github&utm_campaign=relaxcloud-cn%2FRSendMail&labelColor=171717&color=FF570A&link=https%3A%2F%2Fcoderabbit.ai&label=CodeRabbit+Reviews)