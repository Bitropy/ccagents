# Publishing to crates.io

This document contains instructions for publishing `ccagents` to crates.io.

## Prerequisites

1. **Create a crates.io account**: Go to [crates.io](https://crates.io) and login with your GitHub account.

2. **Get your API token**: 
   - Go to [https://crates.io/me](https://crates.io/me)
   - Click on "API Tokens"
   - Generate a new token with a descriptive name (e.g., "ccagents-publish")
   - Copy the token (you won't be able to see it again!)

## Pre-publish Checklist

- [ ] All tests pass locally: `cargo test`
- [ ] All tests pass in CI (check GitHub Actions)
- [ ] Version number is updated in `Cargo.toml`
- [ ] CHANGELOG.md is updated with the new version changes
- [ ] Documentation is up to date
- [ ] No uncommitted changes: `git status`
- [ ] Code is pushed to main branch

## Publishing Steps

1. **Login to cargo with your token**:
   ```bash
   cargo login <your-api-token>
   ```

2. **Verify the package locally** (optional but recommended):
   ```bash
   cargo package --list
   ```

3. **Run a dry-run to catch any issues**:
   ```bash
   cargo publish --dry-run
   ```

4. **Publish to crates.io**:
   ```bash
   cargo publish
   ```

5. **Create a git tag for the release**:
   ```bash
   git tag -a v0.1.0 -m "Release version 0.1.0"
   git push origin v0.1.0
   ```

## Post-publish

1. **Verify on crates.io**: Check that your package appears at https://crates.io/crates/ccagents

2. **Create GitHub Release**:
   - Go to https://github.com/Bitropy/ccagents/releases
   - Click "Create a new release"
   - Choose the tag you just created
   - Add release notes from CHANGELOG.md
   - Publish the release (this will trigger the release workflow to build binaries)

3. **Update version for next development**:
   - Bump version in `Cargo.toml` to next development version (e.g., 0.2.0-dev)
   - Commit: `git commit -am "Bump version to 0.2.0-dev"`

## Version Numbering

We follow [Semantic Versioning](https://semver.org/):
- MAJOR version (1.0.0) - incompatible API changes
- MINOR version (0.1.0) - backwards-compatible functionality additions
- PATCH version (0.1.1) - backwards-compatible bug fixes

## Troubleshooting

### "crate name is already taken"
- The name might have been reserved. Check https://crates.io/crates/ccagents

### "missing required metadata fields"
- Ensure Cargo.toml has all required fields (name, version, authors/description, license)

### "authentication required"
- Run `cargo login <token>` with your crates.io API token

### "version already exists"
- You cannot republish the same version. Bump the version in Cargo.toml

## Notes

- Once published, a version cannot be deleted (only yanked)
- The first publish will reserve the crate name
- Documentation will be automatically built and hosted at https://docs.rs/ccagents

## Security

- Never commit your API token to git
- Store your API token securely (e.g., in a password manager)
- Consider using GitHub Secrets for CI/CD publishing