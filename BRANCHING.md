# Git flow for this Rust crate

Goals
- Support stable (current) releases, plus alpha and beta pre-releases.
- Keep release process reproducible and automatable.
- Keep trunk (main) always releasable (stable), and use develop for integration.

Branches (short)
- main — stable production branch. Only merged PRs that are release-ready.
- develop — integration branch for the next stable release.
- feature/* — short-lived features branched off develop.
- fix/* — bugfix branches off develop (or main for hotfixes).
- hotfix/* — critical fixes made off main, merged back to main and develop.
- alpha/* — long-living or short-living branches for alpha tracks (optional). Merge into develop when ready.
- beta/* — beta release branch for wider testing before stable.

Tagging / versioning
- Use semantic versioning, with pre-release identifiers for alpha/beta.
  - Stable: v1.2.3
  - Beta:   v1.3.0-beta.1
  - Alpha:  v1.4.0-alpha.2
- The crate `version` in Cargo.toml must match the tag before publishing.
- Tags are canonical triggers for the publish workflow: `refs/tags/v*`.

Typical flow
- Day-to-day work:
  - Create feature branches from develop: `feature/awesome-thing`.
  - Open PRs against develop. When reliable and tested, merge to develop.
- Preparing a beta for the next release:
  - Create `beta/x.y.z` branch from develop (or bump the version directly in develop).
  - Bump Cargo.toml to `x.y.z-beta.N`.
  - Tag: `vX.Y.Z-beta.N` and push tag (or merge `beta/*` into develop and tag).
  - CI runs; publishing is triggered only for pushed tags (automated).
- Preparing an alpha:
  - Same as beta but use `alpha/x.y.z` and `x.y.z-alpha.N`.
- Releasing stable:
  - Merge develop into main, ensure version in Cargo.toml is `x.y.z`.
  - Tag `vX.Y.Z` and push tag. Publish workflow will run on the tag.
- Hotfix:
  - Branch from main -> `hotfix/x.y.z` -> change -> bump patch version -> merge into main and develop -> tag and publish.

Branch protection rules (recommended)
- Protect main and develop.
- Require PRs, at least one approving review for main (maybe two), passing CI.
- Require signed commits if you prefer.

Notes about pre-releases
- Pre-release crates are published to crates.io the same way as stable versions. The version string controls whether it's considered pre-release.
- Make sure you never reuse the same version string on crates.io.

Security / tokens
- Add `CRATES_IO_TOKEN` in the repository secrets for publishing.
- The release workflow will use this secret and runs only on tag pushes (or manual dispatch).

This file describes the flow and conventions. See RELEASE_PROCESS.md for step-by-step commands and CI/publish automation in .github/workflows.
