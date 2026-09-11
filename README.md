# veriflow-rs
A minimal, async file-transfer system with SHA-256 integrity checks. Includes client CLI (`client/`) and a server (`server/`) that communicate over a custom TCP protocol utilising length-prefixed JSON headers and chunked binary streaming.

## Running
Build the workspace:
```bash
cargo build --workspace
```
### Server
Start the server:
```bash
cargo r -p server
```
_Note: by default, the server listens on localhost; settings can be manually edited in `config.toml`._ 

### Client
Client commands (localhost is also default):
```bash
cargo r -p veriflow -- transfer --upload /somefolder/somefile.mp4 --ip 127.0.0.1:8080
cargo r -p veriflow -- transfer --download some/file.txt --ip 127.0.0.1:8080
cargo r -p veriflow -- transfer --list --ip 127.0.0.1:8080
cargo r -p veriflow -- transfer --delete some/file.txt --ip 127.0.0.1:8080
```

This is made simpler via the created `config.toml` file, thus you can just specify the command, flag and the file as long as the IP and Port are specified in `config.toml`.
```bash
cargo r -p veriflow -- transfer --upload /somefolder/somefile.mp4
cargo r -p veriflow -- transfer --list
```

_Note: configure the default ip, port (and download dir) settings via (e.g.):_
```bash
cargo r -p veriflow -- config --ip 0.0.0.0 --port 8080 --dir ./downloads
```

More info:
```bash
cargo r -p veriflow -- -h
```

### Tests
Simply run:
```bash
cargo test --workspace
```

## Notable Architectural Decisions
The workspace is divided into three components: a shared core (protocol types, error handling, async file hashing), a server (connection handling, validation, file operations), and a client CLI.
- A custom Protocol that ensures messages begin with a 4-byte big-endian length prefix, followed by a JSON header describing the command (Upload, Download, Delete, List) or response. For file transfers, binary data is chunked and streamed after the header which avoids loading large files into memory.
- For data integrity, uploads and downloads exchange expected SHA256 digests in the header. The receiver recomputes the hash during the stream and rejects or deletes the payload on mismatch. Strict limits are enforced for headers (4KB) and one-shot payloads like lists (capped at 10MB).
- Server enforces safe path joining to prevent traversal attacks. Incoming JSON headers have strict size limits to prevent OOM exploits, and most error handling utilises the `?` operator with custom VeriflowError handling over `.unwrap()` to prevent server panics.
- Both client and server use a separate `config.toml` file for setting defaults, which can be overridden in the aforementioned CLI arguments or manual file amendments.

## Security Notes & V2 Roadmap
Current implementation assumes trust via the network boundary (no authentication or encryption). The next iteration (V2) will adopt a strict "Never Trust the Client" philosophy. This will prioritise additions as outlined in `docs/2026-07-06-Meeting.md`:
- Authenticating users via `.pem` keys mapping to isolated server sub-directories.
- Transfer encryption, challenge-response nonces to prevent replay attacks, and per-IP rate limiting to prevent DoS.
- Atomic temporary-file writes for uploads, resumable transfers, and pre-flight disk exhaustion checks.
