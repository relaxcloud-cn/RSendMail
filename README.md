# RSendMail
Used for batch sending emails for testing

## Features

- Batch process and send multiple emails
- Multi-threaded processing for improved performance
- Support for custom SMTP server configuration
- Detailed logging and statistics
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
- `--from`: From email address
- `--to`: To email address
- `--dir`: Directory containing email files
- `--extension`: Email file extension (default: eml)
- `--processes`: Number of parallel processes (use "auto" for automatic setting based on CPU cores, or specify a number)
- `--batch-size`: Number of emails to send in a single SMTP session (default: 1)

## Example

```bash
# Local example
rsendmail --smtp-server 192.168.1.100 --port 25 --from sender@example.com --to recipient@example.com --dir ./emails --processes 10 --batch-size 5

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
