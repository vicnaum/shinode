# Contributing to SHiNode

Thanks for your interest in contributing!

SHiNode is a very young project and contributions of all sizes are welcome — from bug reports and small fixes to new features and performance improvements. And now with AI all this is easier and faster to do.

## Getting Started

1. **Fork and clone** the repository
2. **Build** the project:
   ```bash
   cargo build
   ```
3. **Run tests**:
   ```bash
   cargo test
   ```
4. See [CLAUDE.md](CLAUDE.md) for a detailed architecture overview and build commands.

## How to Contribute

### Bug Reports & Feature Requests

- Open a [GitHub Issue](https://github.com/vicnaum/shinode/issues) with as much detail as possible
- For bugs: include your OS, hardware, sync range, and any relevant log output
- For features: describe the use case and why it would be useful

### Code Contributions

1. **Open an issue first** for non-trivial changes so we can discuss the approach
2. Fork the repo and create a feature branch from `master`
3. Keep PRs focused — one concern per PR
4. Include tests for new functionality
5. Make sure `cargo test` passes before submitting

### Documentation

Docs improvements are always welcome — README, code comments, examples, or guides.

## Code Style

- Run `cargo fmt` before committing
- Run `cargo clippy` and address warnings
- Follow standard Rust conventions

## Testing

```bash
# Run all tests
cargo test

# Run a specific test
cargo test --manifest-path node/Cargo.toml <test_name>
```

New features should include tests. Bug fixes might benefit from including a test that reproduces the issue when possible.

## Areas Where Help Is Wanted

Check the [Roadmap](ROADMAP.md) for planned features. Some areas where contributions are especially useful:

- Testing, testing testing! We need to bump the coverage and add all kinds of tests, so we can move faster.
- Manual testing on different hardware and OS configurations
- Indexer compatibility testing (rindexer, Ponder, etc.)
- Performance profiling and optimization
- Documentation and guides, articles, blog posts
- Add new chains that support P2P reeipts

## AI-Assisted Contributions

This project is 100% AI-coded, so AI-assisted contributions are welcome.

That said, please review and understand the code you submit — you're responsible for your PR regardless of how it was written.

## Communication

- **Questions & discussions**: open a [GitHub Issue](https://github.com/vicnaum/shinode/issues)
- **Bug reports**: [GitHub Issues](https://github.com/vicnaum/shinode/issues)

## License

By contributing, you agree that your contributions will be licensed under the same [MIT OR Apache-2.0](LICENSE-MIT) dual license as the project.
