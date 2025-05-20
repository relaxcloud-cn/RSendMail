# RSendMail

A high-performance testing tool for bulk email sending

English | [简体中文](README_zh.md)

![Release](https://img.shields.io/github/v/release/kpassy/RSendMail?color=blue&include_prereleases)
![License](https://img.shields.io/github/license/kpassy/RSendMail)
![Stars](https://img.shields.io/github/stars/kpassy/RSendMail?style=social)

## Features

- Batch processing and sending of multiple emails
- Multi-threaded processing for improved performance
- Custom SMTP server configuration
- Multiple logging levels (error/warn/info/debug/trace)
- Detailed error tracking and statistics
- Docker support for easy deployment
- Batch sending in a single SMTP session
- Support for sending regular files as attachments
- Support for sending all files in a directory as separate emails

## Build

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
Download Windows executable (`rsendmail-windows-x86_64.exe`) from the [Releases](https://github.com/kpassy/RSendMail/releases) page.
```bash
rsendmail-windows-x86_64.exe --smtp-server <smtp_server> --port <port> --from <sender> --to <recipient> --dir <email_directory> --processes <num_processes> --batch-size <batch_size>
```

### Local Usage
```bash
rsendmail --smtp-server <smtp_server> --port <port> --from <sender> --to <recipient> --dir <email_directory> --processes <num_processes> --batch-size <batch_size>
```

### Docker Usage
```bash
docker run --rm -v /path/to/emails:/data rsendmail --smtp-server <smtp_server> --port <port> --from <sender> --to <recipient> --dir /data --processes <num_processes> --batch-size <batch_size>
```

### Command-line Options

- `--smtp-server`: SMTP server address
- `--port`: SMTP server port (default: 25)
- `--from`: Sender email address (for SMTP envelope, doesn't modify email content by default)
- `--to`: Recipient email address(es) (for SMTP envelope, comma-separated for multiple recipients, doesn't modify email content by default)
- `--dir`: Directory containing email files (only required when sending EML files, not needed when using --attachment or --attachment-dir)
- `--extension`: Email file extension (default: eml)
- `--processes`: Number of processes, auto sets to CPU core count, or specify a number (default: auto)
- `--batch-size`: Number of emails to send in each SMTP session (default: 1)
- `--smtp-timeout`: SMTP session timeout in seconds (default: 30)
- `--log-level`: Logging level (error/warn/info/debug/trace) (default: info)
- `--keep-headers`: Preserve original email headers (default: false, takes precedence over modify-headers)
- `--modify-headers`: Use --from and --to to modify From and To in email headers (default: false)
- `--anonymize-emails`: Anonymize all email addresses (default: false)
- `--anonymize-domain`: Domain to use for anonymization (default: example.com)
- `--loop`: Loop indefinitely until interrupted (default: false)
- `--repeat`: Number of times to repeat sending (default: 1)
- `--loop-interval`: Interval between loops in seconds (default: 1)
- `--retry-interval`: Interval between retries after failure in seconds (default: 5)
- `--attachment`: Path to a file to send as an attachment
- `--attachment-dir`: Path to a directory with files to send as separate emails (one email per file)
- `--subject-template`: Subject template, supports {filename} variable (default: "Attachment: {filename}")
- `--text-template`: Text content template, supports {filename} variable (default: "Please find the attached file: {filename}")
- `--html-template`: HTML content template, supports {filename} variable
- `--email-send-interval`: Interval in milliseconds between sending each email in a batch (default: 0)
- `--auth-mode`: Enable email account authentication mode
- `--username`: Email account username (required when auth-mode is enabled)
- `--password`: Email account password (required when auth-mode is enabled)
- `--use-tls`: Use TLS encryption for SMTP connection (auto-enabled when port is 465)
- `--email-send-interval-ms`: Milliseconds to wait after each email processing batch (or after each file processing attempt if batch size is 1) is completed. This delay applies after connection failures, authentication failures, or email send successes/failures. Default is 0 (no wait).
- `--smtp-timeout`: Timeout in seconds for SMTP operations. Default is 60.
- `--use-tls`: If set, try to use TLS (STARTTLS) for SMTP connection (port 25 or 587). Implicit TLS is always used for port 465.
- `--accept-invalid-certs`: (TLS only) Accept invalid TLS certificates (e.g., self-signed). WARNING: This reduces security; use only if you trust the target server.

## Logging Levels

The application supports different logging levels to control verbosity:

- `error`: Show only error messages
- `warn`: Show warnings and errors
- `info`: Show general progress information (default)
- `debug`: Show detailed debugging information
- `trace`: Show most detailed tracing information

## Usage Examples

```bash
# Default logging level (info)
rsendmail --smtp-server 192.168.1.100 --port 25 --from sender@example.com --to recipient@example.com --dir ./emails --processes 10 --batch-size 5

# Detailed debugging output
rsendmail --smtp-server 192.168.1.100 --port 25 --from sender@example.com --to recipient@example.com --dir ./emails --processes 10 --batch-size 5 --log-level debug

# Only error messages
rsendmail --smtp-server 192.168.1.100 --port 25 --from sender@example.com --to recipient@example.com --dir ./emails --processes 10 --batch-size 5 --log-level error

# Docker example
docker run --rm -v $(pwd)/emails:/data rsendmail --smtp-server 192.168.1.100 --port 25 --from sender@example.com --to recipient@example.com --dir /data --processes 10 --batch-size 5 --log-level info

# Sending a single attachment (--dir parameter not needed)
rsendmail --smtp-server 192.168.1.100 --port 25 --from sender@example.com --to recipient@example.com --attachment ./document.pdf

# Using custom templates with attachment (--dir parameter not needed)
rsendmail --smtp-server 192.168.1.100 --port 25 --from sender@example.com --to recipient@example.com --attachment ./document.pdf --subject-template "Important file: {filename}" --text-template "Hello,\n\nPlease find the attached file: {filename}.\n\nRegards,\nRSendMail Team"

# Sending all files in a directory as separate emails (--dir parameter not needed)
rsendmail --smtp-server 192.168.1.100 --port 25 --from sender@example.com --to recipient@example.com --attachment-dir ./documents --subject-template "File: {filename}"
```

## Attachment Feature Details

RSendMail now supports sending regular files as email attachments without the need to create EML files first. This is useful for quickly testing file sending.

### Attachment Mode Features

- Automatic MIME type detection
- Customizable email subject and content
- Template variables for automatic filename insertion
- Optional HTML content support
- Independent from batch EML sending functionality
- Support for sending a single file (using `--attachment`) or all files in a directory (using `--attachment-dir`)
- No need to provide the `--dir` parameter when using attachment modes

### Template Variables