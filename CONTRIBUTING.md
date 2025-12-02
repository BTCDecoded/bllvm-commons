# Contributing to BTCDecoded Governance App (blvm-commons)

Thank you for your interest in contributing to the BTCDecoded Governance App! This document contains **repo-specific guidelines only**. For comprehensive contributing guidelines, see the [BLVM Documentation](https://docs.thebitcoincommons.org/development/contributing.html).

## Quick Links

- **[Complete Contributing Guide](https://docs.thebitcoincommons.org/development/contributing.html)** - Full developer workflow
- **[PR Process](https://docs.thebitcoincommons.org/development/pr-process.html)** - Governance tiers and review process
- **[Governance Documentation](https://docs.thebitcoincommons.org/governance/overview.html)** - Governance system

## Code of Conduct

This project follows the [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct). By participating, you agree to uphold this code.

## Repository-Specific Guidelines

### Development Setup

#### Prerequisites

- Rust 1.70+
- PostgreSQL 13+
- Git

#### Getting Started

1. Clone the repository:
   ```bash
   git clone https://github.com/BTCDecoded/blvm-commons.git
   cd blvm-commons
   ```

2. Set up environment:
   ```bash
   cp env.example .env
   # Edit .env with your configuration
   ```

3. Set up database:
   ```bash
   createdb governance
   ```

4. Run tests:
   ```bash
   cargo test
   ```

### Code Style

- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting
- Follow Rust naming conventions
- Document public APIs with `///` comments

### Testing

```bash
# Unit tests
cargo test

# Integration tests
cargo test --test integration_tests

# Coverage
cargo tarpaulin --out Html
```

### GitHub App Development

- **Test locally** with GitHub App development tools
- **Use test keys** for development (never commit real keys)
- **Follow GitHub App best practices** for webhook handling
- **Document webhook events** and their handling

### Security Considerations

- Never commit secrets or API keys
- Use environment variables for sensitive configuration
- Follow security boundaries defined in SECURITY.md
- All cryptographic operations require security review

## Getting Help

- **Documentation**: [docs.thebitcoincommons.org](https://docs.thebitcoincommons.org)
- **Issues**: Use GitHub issues for bugs and feature requests
- **Discussions**: Use GitHub discussions for questions
- **Security**: See [SECURITY.md](SECURITY.md)

Thank you for contributing to the BTCDecoded Governance App!
