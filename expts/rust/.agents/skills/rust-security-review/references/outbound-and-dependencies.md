# Outbound requests and dependencies: SSRF, vulnerable components

Scope: any server-side network request influenced by user input, plus the dependency tree, CI, and the container image.

## SSRF (server-side request forgery)

**Threat:** the server is tricked into requesting a URL the attacker chose, reaching internal services, cloud metadata endpoints, or localhost.

First, find every outbound request: `rg -n 'reqwest|hyper|ureq|surf|isahc|TcpStream|connect\(' crates/` and check `Cargo.toml` for HTTP client crates. Include webhooks, URL previews/unfurling, avatar or image fetch-by-URL, import/feed features, OAuth/OIDC discovery, and email or link-checking features.

If no code fetches a user-influenced URL, say so with the evidence (search commands and results) and mark SSRF as **not applicable**, not "safe".

Check, for every user-influenced outbound request:
- [ ] **Prefer not to fetch at all.** Is there a design that avoids server-side fetching of user URLs?
- [ ] **Allowlist** of hosts or domains where the feature allows it. This is far stronger than a blocklist.
- [ ] Scheme restricted to `https` (and `http` only if truly required). Reject `file:`, `gopher:`, `ftp:`, and others.
- [ ] **Resolve DNS yourself and validate the resulting IP** before connecting, then connect to that validated IP (prevents DNS rebinding and TOCTOU). Reject loopback (`127.0.0.0/8`, `::1`), private ranges (`10/8`, `172.16/12`, `192.168/16`, `fc00::/7`), link-local (`169.254.0.0/16`, including the cloud metadata address `169.254.169.254`, and `fe80::/10`), unspecified (`0.0.0.0`, `::`), multicast and broadcast, carrier-grade NAT (`100.64/10`), and IPv4-mapped IPv6 forms of all of these.
- [ ] Watch for URL parsing tricks: userinfo (`http://allowed.com@evil.com`), alternate IP encodings (decimal, octal, hex), `localhost` aliases, trailing dots, Unicode/IDN hosts. Parse with a real URL parser (`url` crate) and validate the parsed host, never substring-match the raw string.
- [ ] **Redirects:** disable automatic following, or re-validate every hop with the same checks and a low hop limit.
- [ ] Timeouts (connect and total), response size limit, and concurrency limit on the fetch.
- [ ] The response body is not echoed back raw to the user (blind vs full SSRF), and error messages do not reveal internal host information.
- [ ] Outbound requests run with minimal credentials; internal service credentials are never attached to user-driven requests.
- [ ] Network-level egress restrictions where the platform allows, as a second layer.

Test: URLs pointing at `http://127.0.0.1`, `http://169.254.169.254/`, a private IP, a redirect to a private IP, and a hostname resolving to a private IP must all be rejected.

## Component vulnerabilities (dependencies and images)

Check:
- [ ] `Cargo.lock` is committed and CI builds with `--locked`.
- [ ] `cargo deny check advisories` (RustSec database) runs in CI and on a weekly schedule. Alternative or addition: `cargo audit`. A failing advisory blocks the pipeline unless explicitly ignored in `deny.toml` **with a written reason and an expiry date**.
- [ ] `deny.toml` also bans unwanted licenses, unknown registries and git sources, and flags **duplicate versions** of security-relevant crates.
- [ ] Dependabot (or Renovate) enabled for Cargo, GitHub Actions, and Docker base images. Security updates are reviewed promptly; major updates are read, not auto-merged blindly.
- [ ] Unmaintained and yanked crates: report them. Check `cargo deny` output for "unmaintained" and "yanked" advisories.
- [ ] **New dependencies are justified.** Before adding one: is it needed, maintained, widely used, and small? Check its `unsafe` usage, its own dependency tree (`cargo tree -p <crate>`), and its repository activity. Prefer fewer, well-known crates.
- [ ] Features minimized: `default-features = false` and enable only what is needed, to shrink attack surface and binary size.
- [ ] `#![forbid(unsafe_code)]` in workspace crates that do not need `unsafe`; any `unsafe` block has a safety comment and a reviewer.
- [ ] Optional deeper checks: `cargo vet` or `cargo supply-chain` for trust review; `cargo geiger` to count `unsafe` in the tree.
- [ ] **CI supply chain:** GitHub Actions pinned to a commit SHA (not a moving tag) for third-party actions; workflow `permissions:` minimal (`contents: read` by default; `packages: write` only in the publish job); secrets never exposed to pull requests from forks; no `pull_request_target` with checkout of untrusted code.
- [ ] **Container image:** Trivy (or equivalent) scan on the built image, failing on high/critical findings. Base image pinned by digest or at least a specific version. Minimal contents (`scratch` leaves little to scan, but still scan the binary's dependencies via SBOM).
- [ ] Optional: SBOM and build provenance attestation for the published image.
- [ ] Tool versions (Rust toolchain) pinned in `rust-toolchain.toml` and updated regularly, since compiler and standard library fixes are part of the component surface.

When reporting a vulnerable component, include: crate, locked version, advisory id, whether the vulnerable code path is actually reachable in this project (verify, do not assume), the fixed version, and whether upgrading needs code changes.
