# Security policy

This policy helps you report a security problem in smllm. It says which versions receive fixes,
how to report a vulnerability in private, and which problems count as security issues.

- [Supported versions](#supported-versions)
- [Report a vulnerability](#report-a-vulnerability)
- [Scope](#scope)

## Supported versions

smllm hasn't reached version 1.0. Security fixes go only into the latest release and the `main`
branch. To get a fix, upgrade to the latest release.

## Report a vulnerability

To report a security issue in private, use GitHub's private vulnerability reporting:

1. Open the [Security tab](https://github.com/six5536/smllm/security) of the repository.
2. To start a private advisory, click **Report a vulnerability**.

Don't open a public issue for a suspected vulnerability. The maintainer acknowledges the report,
then agrees a fix and a disclosure date with you.

## Scope

smllm trusts a state machine file as `make` trusts a Makefile: its `command` guards and actions
run shell commands with your permissions. Treat a `.smllm/` folder from a repository that you
don't trust as you would treat its build scripts. Text from the large language model (LLM) reaches
a command only as an environment variable, such as `SMLLM_PARAM_*` or `SMLLM_REF`. smllm never
interpolates that text into a command line.

These problems are in scope:

- An event param, or any other input from the agent, that can reach a command other than as an
  environment variable.
- A write outside the paths that smllm documents: `.smllm/`, the harness files that it installs,
  `~/.config/smllm/`, and `~/.local/state/smllm`.
- A supply-chain problem in what the release pipeline publishes: a tampered binary or archive, a
  mismatched `SHA256SUMS`, a publish from outside the release workflow, or a dependency that got
  past the `cargo-deny` and `cargo-audit` gates.
