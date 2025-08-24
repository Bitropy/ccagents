# GitHub Repository Setup for Publishing

This guide explains how to configure your GitHub repository for automated publishing to crates.io.

## 1. Create the crates-io Environment

1. Go to your repository settings: `https://github.com/Bitropy/ccagents/settings/environments`
2. Click **"New environment"**
3. Name it exactly: `crates-io`
4. Configure the environment:
   - ✅ **Required reviewers**: Add yourself or team members who should approve releases
   - ✅ **Deployment branches**: Restrict to `main` branch only
5. Click **"Save protection rules"**

## 2. Add the CARGO_REGISTRY_TOKEN Secret

### Get your crates.io token:
1. Login to [crates.io](https://crates.io) with your GitHub account
2. Go to [Account Settings](https://crates.io/me)
3. Click on **"API Tokens"**
4. Click **"New Token"**
5. Name it: `ccagents-github-actions`
6. Copy the token (you won't see it again!)

### Add to GitHub:
1. Go to `https://github.com/Bitropy/ccagents/settings/secrets/actions`
2. Click **"New repository secret"**
3. Name: `CARGO_REGISTRY_TOKEN`
4. Value: Paste your crates.io token
5. Click **"Add secret"**

## 3. How to Use the Workflows

### Option 1: Manual Release (Recommended for first release)

1. Go to the **Actions** tab
2. Select **"Publish to crates.io"** workflow
3. Click **"Run workflow"**
4. Enter the version (e.g., `0.1.0`)
5. Click **"Run workflow"**
6. Wait for the `verify` job to complete
7. Go to the workflow run and **approve** the `crates-io` environment
8. The workflow will:
   - Publish to crates.io
   - Create a git tag
   - Create a GitHub release with binaries

### Option 2: Release via GitHub UI

1. Go to **Releases**
2. Click **"Create a new release"**
3. Create a new tag (e.g., `v0.1.0`)
4. Fill in release notes
5. Click **"Publish release"**
6. The workflow will trigger automatically
7. Approve the `crates-io` environment when prompted

## 4. Workflow Features

### The publish workflow (`publish.yml`):

- **Verify stage**: 
  - Checks version consistency
  - Runs tests and clippy
  - Validates package with `cargo publish --dry-run`

- **Manual approval**: 
  - Requires approval for the `crates-io` environment
  - Prevents accidental publishes

- **Publish stage**:
  - Publishes to crates.io
  - Only runs after approval

- **Release artifacts**:
  - Builds binaries for Linux (x64, ARM64) and macOS (x64, ARM64)
  - Creates tar.gz archives with SHA256 checksums
  - Uploads to GitHub release

## 5. Version Management

Before releasing:

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Commit changes: `git commit -am "Bump version to 0.1.0"`
4. Push to main: `git push origin main`
5. Then trigger the release workflow

## 6. Troubleshooting

### "Environment not found"
- Make sure you created the `crates-io` environment exactly as named

### "Secret not found"
- Verify `CARGO_REGISTRY_TOKEN` is added as a repository secret

### "Version mismatch"
- Ensure Cargo.toml version matches the version you're trying to release

### "Package already published"
- You cannot republish the same version
- Bump the version in Cargo.toml

## Security Notes

- Never commit the CARGO_REGISTRY_TOKEN
- Limit the token's scope if possible
- Rotate tokens periodically
- Use environment protection rules to control who can publish