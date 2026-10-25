# Security policy

## Report a vulnerability privately

Use [GitHub private vulnerability reporting](https://github.com/cubearc/cube/security/advisories/new).
From the repository's **Security → Advisories** page, select **Report a vulnerability**.
Include the affected commit or version, reproduction steps, impact, and a suggested
fix if available. Keep vulnerability details out of public issues and pull requests
until maintainers agree on disclosure.

If the reporting button is unavailable, open a public issue asking maintainers to
enable private vulnerability reporting, without including vulnerability details.
There is no separate security email address.

## Maintainer setup

A public repository is required for GitHub private vulnerability reporting. For
a private repository, a maintainer must first decide whether to make it public;
do not assume the reporting form is available.

A repository administrator must enable **Private vulnerability reporting** under
repository **Settings → Advanced Security** and monitor advisory notifications.
See [GitHub's setup instructions](https://docs.github.com/en/code-security/how-tos/report-and-fix-vulnerabilities/configure-vulnerability-reporting/configure-for-a-repository).
Coordinate remediation and disclosure with the reporter through the private advisory.
Security fixes currently target `main`; no released-version support policy exists yet.

## Intended use

ROT13 and repeating-key XOR provide no modern confidentiality. SHA-256 is an
unkeyed hash, not a password hashing scheme or authentication protocol. Cube
provides building blocks for data transformation and forensic investigation.
