You are reviewing the INSTALL section of one or more documentation files in complete isolation. You have NOT seen the codebase or any other files. Read only what is provided.

Your job: grade whether the install copy gives a published-tool first impression OR a build-from-source first impression.

A "published-tool first impression" is when the FIRST install command in the hero / install section is a package-manager command:
- `cargo install <name>` or `cargo binstall <name>` (Rust packages)
- `brew install <name>` (Homebrew)
- `pip install <name>` (Python packages)
- `npm install <name>` or `npm i <name>` (Node packages)
- `go install <path>` (Go packages)
- `apt install <name>`, `dnf install <name>`, `pacman -S <name>` (system packages)
- `curl <installer-url> | sh` (one-liner installer that resolves to a published binary)
- `winget install <name>`, `choco install <name>` (Windows package managers)

A "build-from-source first impression" is when the FIRST install command is:
- `git clone <repo>` followed by a build step
- `cargo build` or `make` or `npm run build` against a checkout
- ANY command that requires the user to have the source tree before they can install

Build-from-source paths are FINE as a fallback. They are NOT fine as the lead. A user landing on the install section should see "you can install this with one command" before they see "or you can clone and build."

Report the following sections:

## Install paths observed

For EACH file you read, list the FIRST install command in the install section, in the order they appear. Cite line numbers.

## Lead command type

For each file: is the first install command a package-manager command, a one-liner installer, or a build-from-source step?

## Friction points

Specific issues with the install copy: missing target list (which platforms? which architectures?), unclear version pinning, broken-looking command syntax, etc.

## Verdict

Numeric score 1-10:
- 10: every file's hero leads with a package-manager command, lists supported targets, names a binstall/binary fallback explicitly.
- 7-9: leads with package-manager command but missing one of: target list, fallback path, or version pinning guidance.
- 4-6: leads with package-manager command but the build-from-source path is given equal billing OR appears confusingly close to the lead.
- 1-3: leads with `git clone && build` (worse first impression for a published tool).

Rate: <integer 1-10>
Verdict: <CLEAR if score >= 7 / NEEDS-WORK if 4-6 / CONFUSING if 1-3>
Rationale: <one paragraph>
