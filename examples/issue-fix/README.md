# Issue fix example

This example is a machine that guides a coding agent through one GitHub issue, from triage to a
merged pull request. Use it as a starting point for your own development workflow.

The machine works in these steps:

1. In `TRIAGE`, the agent decides whether the issue is real. If GitHub has no issue for it yet,
   the agent creates one and fires `issueCreated`, which records the issue's id as the ref.
   Otherwise the agent fires `accept`.
2. `CHECK` runs `gh issue view` on the ref. The agent goes on to `WORK` if the issue exists, and
   back to `TRIAGE` if it doesn't.
3. `WORK` switches to the branch `issue/GH-123`, for issue GH-123, and the agent writes the fix and
   a test. On `submit`, the machine runs `cargo test`. If the tests pass, it commits, pushes, and
   opens a pull request. If they fail, the agent stays in `WORK`, and after three visits the
   machine moves to `ESCALATE`, where the agent asks the user for help.
4. In `REVIEW`, the agent reviews the pull request against a checklist. `approve` sets the pull
   request to merge after its checks pass, and `reject` sends the agent back to `WORK`.

The example has these files:

- `issue-fix.smllm.yaml` is the machine.
- `triage.md`, `work.md`, and `review.md` hold the instructions of `TRIAGE`, `WORK`, and `REVIEW`.
- `config.toml` lists the machine.

The machine needs `git`, the GitHub command `gh`, and `cargo`. For a project in another language,
change the test command of `WORK`.

To use the machine in a project, copy the machine and its instructions into the project's
`.smllm` folder:

```sh
cp examples/issue-fix/*.smllm.yaml examples/issue-fix/*.md .smllm/
```

Then add `"issue-fix.smllm.yaml"` to the `files` list of `.smllm/config.toml`, and run
`smllm validate`.
