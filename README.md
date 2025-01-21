# RSendMail
Used for batch sending emails for testing

## Features

- Batch process and send multiple emails
- Multi-threaded processing for improved performance
- Support for custom SMTP server configuration
- Detailed logging and statistics
- Docker support for easy deployment

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
rsendmail -s <smtp_server> -P <port> -f <from_addr> -t <to_addr> -d <email_dir> -p <parallel>
```

### Docker Usage
```bash
docker run --rm -v /path/to/emails:/data rsendmail -s <smtp_server> -P <port> -f <from_addr> -t <to_addr> -d /data -p <parallel>
```

### Parameters

- `-s, --smtp`: SMTP server address
- `-P, --port`: SMTP server port
- `-f, --from`: From email address
- `-t, --to`: To email address
- `-d, --dir`: Directory containing email files (.eml)
- `-p, --parallel`: Number of parallel processes (default: 10)

## Example

```bash
# Local example
rsendmail -s 192.168.1.100 -P 25 -f sender@example.com -t recipient@example.com -d ./emails -p 10

# Docker example
docker run --rm -v $(pwd)/emails:/data rsendmail -s 192.168.1.100 -P 25 -f sender@example.com -t recipient@example.com -d /data -p 10
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

## Security

- Non-root user execution
- Minimal container footprint
- Isolated runtime environment
- No persistent storage
