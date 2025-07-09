# Releasing FTL SDKs

This document describes how to release new versions of the FTL SDKs.

## Prerequisites

### For Rust SDK (ftl-sdk-rs)
- Rust and Cargo installed
- Authenticated with crates.io: `cargo login`
- Member of the ftl-sdk crate on crates.io

### For TypeScript SDK (ftl-sdk-ts)
- Node.js and npm installed
- Authenticated with npm: `npm login`
- Member of the ftl-sdk package on npm

## Release Process

### 1. Ensure Clean Working Directory
```bash
git status  # Should show no uncommitted changes
git pull origin main  # Ensure you're up to date
```

### 2. Run Pre-Release Checks

#### Rust SDK
```bash
cd src/ftl-sdk-rs
make check  # Runs format check, linting, and tests
make publish-dry-run  # Verifies the package
```

#### TypeScript SDK
```bash
cd src/ftl-sdk-ts
npm run check  # Runs format check, linting, and tests
npm run publish:dry-run  # Verifies the package
```

### 3. Bump Version

Choose the appropriate version bump based on [Semantic Versioning](https://semver.org/):
- **Patch** (0.0.x): Bug fixes, documentation updates
- **Minor** (0.x.0): New features, backward compatible
- **Major** (x.0.0): Breaking changes

#### Rust SDK
```bash
cd src/ftl-sdk-rs
make version-patch  # or version-minor, version-major
```

#### TypeScript SDK
```bash
cd src/ftl-sdk-ts
npm run version:patch  # or version:minor, version:major
```

### 4. Update Changelog (if exists)
Update CHANGELOG.md with the new version and changes.

### 5. Commit Version Bump
```bash
git add -A
git commit -m "chore: bump ftl-sdk-{rs|ts} to vX.Y.Z"
```

### 6. Create Git Tag
```bash
git tag ftl-sdk-rs-vX.Y.Z  # For Rust SDK
git tag ftl-sdk-ts-vX.Y.Z  # For TypeScript SDK
```

### 7. Publish

#### Rust SDK
```bash
cd src/ftl-sdk-rs
make publish
# Confirm when prompted
```

The package will be available at: https://crates.io/crates/ftl-sdk

#### TypeScript SDK
```bash
cd src/ftl-sdk-ts
npm run publish:npm
```

The package will be available at: https://www.npmjs.com/package/ftl-sdk

### 8. Push Changes and Tags
```bash
git push origin main
git push origin ftl-sdk-rs-vX.Y.Z  # If releasing Rust
git push origin ftl-sdk-ts-vX.Y.Z  # If releasing TypeScript
```

## Quick Release Commands

For convenience, the TypeScript SDK has combined release commands:

```bash
cd src/ftl-sdk-ts
npm run release:patch  # Bump patch version and publish
npm run release:minor  # Bump minor version and publish
npm run release:major  # Bump major version and publish
```

## Troubleshooting

### Rust Publishing Issues
- Ensure you're logged in: `cargo login`
- Check crate name availability
- Verify all dependencies are published
- Check for sensitive information in code

### TypeScript Publishing Issues
- Ensure you're logged in: `npm whoami`
- Check package name availability
- Verify .npmignore or files field in package.json
- Ensure dist/ is built: `npm run build`

## Post-Release

1. Create a GitHub Release with the tag
2. Announce the release in relevant channels
3. Update any example projects to use the new version