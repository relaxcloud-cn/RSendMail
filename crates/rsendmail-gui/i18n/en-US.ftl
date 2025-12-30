# RSendMail GUI - English (en-US)
# Fluent Translation File

## Application
app-title = RSendMail

## SMTP Configuration
smtp-server = SMTP Server
server-address = Server Address
port = Port
use-tls = Use TLS
accept-invalid-certs = Accept Invalid Certificates
auth-required = Authentication Required
username = Username
password = Password
sender = Sender
recipient = Recipient
recipient-hint = (comma separated for multiple)

## Send Mode
send-mode = Send Mode
eml-batch = EML Batch
single-attachment = Single Attachment
dir-attachment = Directory Attachment
eml-directory = EML Directory
attachment-file = Attachment File
attachment-directory = Attachment Directory
extension = Extension
browse = Browse...
found-files = Found { $count } files
email-subject = Email Subject
email-body = Email Body
filename-hint = Use {"{filename}"} for auto filename insertion

## Advanced Options
advanced-options = Advanced Options
performance = Performance
processes = Processes
batch-size = Batch Size
send-interval = Send Interval (ms)
timeout = Timeout (sec)
loop-settings = Loop Settings
infinite-loop = Infinite Loop
repeat-count = Repeat Count
loop-interval = Loop Interval (sec)
retry-interval = Retry Interval (sec)
email-processing = Email Processing
keep-headers = Keep Original Headers
modify-headers = Modify Headers
anonymize-emails = Anonymize Emails
domain = Domain
logging = Logging & Error Handling
log-level = Log Level
log-file = Log File
failed-emails-dir = Failed Emails Directory
optional = (optional)

## Statistics
statistics = Statistics
total = Total
success = Success
failed = Failed
qps = QPS
current-round = Current Round
elapsed-time = Elapsed Time

## Logs
send-log = Send Log
clear = Clear
export-log = Export Log

## Status
status-ready = Ready
status-preparing = Preparing...
status-sending = Sending...
status-stopped = Stopped
status-completed = Completed
status-stopping = Stopping...
status-error = Error

## Buttons
save-config = Save Config
load-config = Load Config
test-connection = Test Connection
start-send = Start Send
stop-send = Stop Send

## Messages - Errors
error-smtp-required = Please enter SMTP server address
error-sender-required = Please enter sender address
error-recipient-required = Please enter recipient address
error-eml-dir-required = Please select EML directory
error-attachment-required = Please select attachment file
error-attachment-dir-required = Please select attachment directory
error-username-required = Authentication requires username
error-password-required = Authentication requires password

## Messages - Info
info-testing-connection = Testing connection...
info-connecting = Connecting to { $server }:{ $port } (TLS: { $tls })
info-connection-success = Connection test successful
info-connection-failed = Connection test failed: { $error }
info-starting-round = Starting round { $current }/{ $total }
info-round-completed = Round { $round } completed
info-send-completed = Send completed! Success: { $success }, Failed: { $failed }
info-waiting = Waiting { $seconds } seconds before next round...
info-stopping = Stopping send...
info-config-saved = Config saved to: { $path }
info-config-loaded = Config loaded from: { $path }
info-log-exported = Log exported to: { $path }
info-save-failed = Save failed: { $error }
info-load-failed = Load failed: { $error }
info-export-failed = Export failed: { $error }
info-parse-failed = Parse failed: { $error }

## Language
language = Language
