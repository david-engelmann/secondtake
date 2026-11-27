# Security Policy

## Supported Versions

| Version | Supported |
| :--- | :--- |
| `0.1.x` (`main`) | Yes |

## Security Model

- **Local offline evaluation by default**: `secondtake compare` reads local `flighttape` `1.0` JSON files, `registration.json`, and optional `judgments.json` fixtures without making network connections.
- **Explicit live judge opt-in**: Subprocess execution occurs only when the caller explicitly passes `--judge live` and sets the `JEV_CALL` environment variable to a trusted local Python script. Never point `JEV_CALL` at untrusted files or untrusted directories.
- **Input and write-path safety**: `secondtake` enforces JSON nesting caps, token magnitude bounds (`<= 10^18`), single-line control-character checks on registration metadata, symlink-loop detection on output paths, and exclusive (`O_CREAT | O_EXCL`, mode `0600` on Unix) temporary file creation before atomic rename.

## Reporting a Vulnerability

Please report suspected security vulnerabilities privately via [GitHub Security Advisories](https://github.com/david-engelmann/secondtake/security/advisories/new) rather than opening a public issue. Include a minimal reproduction and affected commit SHA so the issue can be locked with a contract obligation and regression test before disclosure.
