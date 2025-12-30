# Security Policy

## Supported Versions

We currently support the latest stable release of this CLI tool. Users are encouraged to always run the latest version available on Crates.io or GitHub Releases to ensure all security patches are applied.

| Version | Supported |
| --- | --- |
| Latest Stable (Crates.io) | ✅ |
| Older Versions | ❌ |

## Reporting a Vulnerability

If you discover a security vulnerability in this project, **please do not open a public issue** on GitHub. Instead, follow these steps:

1. **Contact us privately** via email at `microdaika1@gmail.com`.
2. Provide detailed information about the vulnerability, including:
* A description of the issue (e.g., buffer overflow, panic injection, argument parsing exploit).
* Steps to reproduce the issue via the command line.
* Example of malicious input or arguments.
* Potential impact (e.g., denial of service, privilege escalation).



We aim to respond within **5 business days** and will work with you to assess and address the issue as quickly as possible.

## Disclosure Policy

Once a fix has been developed and tested, we will:

* Notify the reporter of the resolution.
* Release a patched version on Crates.io and GitHub.
* Publish release notes detailing the security update (advising users to run `cargo install --force ...` or update via their package manager).

## Security Best Practices for Users

We strongly recommend that users:

* **Keep the tool updated:** Regularly check for updates via Cargo (`cargo install <crate_name>`) or your distribution's package manager.
* **Least Privilege:** Avoid running the binary with `sudo` or Administrator privileges unless explicitly required by the command functionality.
* **Input Validation:** Be cautious when passing untrusted file paths or raw data as arguments to the CLI.

## Dependency Management & Supply Chain

We take supply chain security seriously:

* We use **`Cargo.lock`** to ensure reproducible builds and verify dependency versions.
* We utilize automated tools (such as `cargo-audit` or GitHub Dependabot) to regularly scan our dependency tree for known vulnerabilities filed in the RustSec Advisory Database.
4. **Зависимости:** Упомянут `Cargo.lock` (стандарт для бинарных крейтов) и `cargo-audit` / RustSec (стандарт аудита в Rust).

Хотите, я помогу настроить GitHub Action, который будет автоматически запускать `cargo audit` при каждом пулл-реквесте?
