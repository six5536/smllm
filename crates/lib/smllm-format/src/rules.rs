//! The rules that `smllm validate` cites: each finding ends with the code of
//! the rule that it breaks, such as `(CFG-7)`. [`RULES`] lists every code that
//! a finding can carry, and [`markdown`] renders the list as the page that
//! `smllm info rules` prints and `docs/validate-rules.md` holds. A test of the
//! CLI checks that the page is current, and a test of this module checks that
//! every code in the source is listed.
// @zen-component: CFG-Findings
// @zen-impl: CFG-14_AC-1

/// One rule that a finding can cite.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rule {
    /// The code that a finding prints, such as `CFG-7`.
    pub code: &'static str,
    /// The rule's name.
    pub title: &'static str,
    /// What `smllm validate` checks under the rule.
    pub checks: &'static str,
}

/// Every rule that a finding can cite, in the order of their codes.
pub const RULES: &[Rule] = &[
    Rule {
        code: "CFG-1",
        title: "The machine file",
        checks: "The file is YAML that parses, sets `meta.smllm: 1`, and has at least one state. Its keys are the known ones, each required key is present, and each value has the expected type. The machine's ID and its state names have the allowed characters, and no other machine in the config has the same ID or one that differs only in case.",
    },
    Rule {
        code: "CFG-2",
        title: "The supported part of XState",
        checks: "The file uses only the part of XState v5 that smllm supports: flat atomic and final states, without nested or parallel states, delays, invocations, or context.",
    },
    Rule {
        code: "CFG-3",
        title: "Transitions",
        checks: "The initial state and each transition's target exist, each state can be reached, and a transition that should re-enter its state names it as the target.",
    },
    Rule {
        code: "CFG-4",
        title: "Actions",
        checks: "Each action is a `{type, params}` object of a known type, with the params that its type needs. A prompt has exactly one of `text` and `file`, a file path is relative to the file that gives it, and `setRef` appears only where an event carries the ref param.",
    },
    Rule {
        code: "CFG-5",
        title: "Guards",
        checks: "Each guard is a `{type, params}` object of a known type, with valid params. A `visits` guard counts a state that exists, and its `atLeast` is more than 0.",
    },
    Rule {
        code: "CFG-6",
        title: "The instance's names",
        checks: "The ref param's name contains only letters, digits, `_`, and `-`.",
    },
    Rule {
        code: "CFG-7",
        title: "Event declarations",
        checks: "Each declared event and param has a name that contains only letters, digits, `_`, and `-`. Its params are an object of string properties, each required param is in `properties`, and each `enum` and `pattern` is valid and allows at least one value.",
    },
    Rule {
        code: "CFG-8",
        title: "Rewording for one state",
        checks: "A state rewords only the events that it has a transition on, and only params that the event declares in `meta.events`.",
    },
    Rule {
        code: "CFG-9",
        title: "Using events",
        checks: "Each declared event is used by a state, and each ref pattern can match a ref.",
    },
    Rule {
        code: "CFG-11",
        title: "Shared actions",
        checks: "`meta.sharedActions` lists only states that exist.",
    },
    Rule {
        code: "CFG-12",
        title: "Final states",
        checks: "A final state has no `on` or `always`, isn't an entry point, and can be reached. A machine without a final state gets a warning, because its instances never complete.",
    },
    Rule {
        code: "CFG-13",
        title: "The machine's structure",
        checks: "At most one state is the fallback, and the fallback isn't final. A state doesn't handle a built-in event under `on`, a list of guarded transitions ends with one without a guard, and neither the ref param nor any other param has the name of a param of `enter`. An event with the ref param sets the ref.",
    },
    Rule {
        code: "CFG-14",
        title: "Findings",
        checks: "smllm can read each file that the config names.",
    },
    Rule {
        code: "CFG-15",
        title: "Combining configs",
        checks: "A config with errors doesn't replace the machines of the configs before it, and validate says which machines it leaves out and which machine replaces which.",
    },
    Rule {
        code: "CFG-16",
        title: "String params",
        checks: "Each param has `type: string`.",
    },
    Rule {
        code: "DEC-2",
        title: "Always states",
        checks: "A state with `always` has no `on` events, because smllm leaves it at once, and no chain of `always` transitions loops back to its start.",
    },
    Rule {
        code: "DEC-4",
        title: "Command forms",
        checks: "A command's `run` is a non-empty string, run by a shell, or a non-empty list, run without one.",
    },
    Rule {
        code: "DEC-5",
        title: "Command results",
        checks: "A command's `timeoutSecs` is at least 1.",
    },
    Rule {
        code: "DEC-9",
        title: "Commands without a shell",
        checks: "A list-form `run` doesn't rely on `$SMLLM_*` variables, because no shell expands them.",
    },
    Rule {
        code: "IDLE-1",
        title: "Built-in events",
        checks: "A built-in event changes only its description, never its params.",
    },
    Rule {
        code: "IDLE-4",
        title: "Saved instances",
        checks: "Each saved instance is in a state that still exists.",
    },
    Rule {
        code: "STO-1",
        title: "The instance files",
        checks: "smllm can read each instance file and each ref marker of the config's machines.",
    },
];

/// The rule with `code`, if a finding can cite it.
pub fn rule(code: &str) -> Option<&'static Rule> {
    RULES.iter().find(|r| r.code == code)
}

/// The page of the rules, in Markdown.
pub fn markdown() -> String {
    let mut out = String::from(
        "# The rules of smllm validate\n\n\
         This page lists the rules that `smllm validate` checks. Each finding ends with the code of \
         the rule that it breaks, such as `(CFG-7)`, and this page says what each rule checks. \
         `smllm info rules` prints the same page, so it always matches the smllm that you run.\n\n\
         | Code | Rule | What validate checks |\n\
         | ---- | ---- | -------------------- |\n",
    );
    for r in RULES {
        out.push_str(&format!("| `{}` | {} | {} |\n", r.code, r.title, r.checks));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Each code that the source passes to a finding, read from the source
    /// files of this crate and of the CLI's `validate`.
    fn codes_in_source() -> std::collections::BTreeSet<String> {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut files = vec![root.join("../../app/smllm/src/commands/config.rs")];
        let mut dirs = vec![root.join("src")];
        while let Some(d) = dirs.pop() {
            for e in std::fs::read_dir(d).unwrap() {
                let p = e.unwrap().path();
                if p.is_dir() {
                    dirs.push(p);
                } else if p.extension().is_some_and(|x| x == "rs") && !p.ends_with("rules.rs") {
                    files.push(p);
                }
            }
        }
        let re = regex::Regex::new(r#""((?:CFG|DEC|IDLE|STO)-\d+)""#).unwrap();
        let mut codes = std::collections::BTreeSet::new();
        for f in files {
            let text = std::fs::read_to_string(&f).unwrap();
            for c in re.captures_iter(&text) {
                codes.insert(c[1].to_string());
            }
        }
        codes
    }

    // @zen-test: CFG-14_AC-1
    #[test]
    fn every_code_in_the_source_is_listed_and_every_listed_code_is_used() {
        let used = codes_in_source();
        let listed: std::collections::BTreeSet<String> =
            RULES.iter().map(|r| r.code.to_string()).collect();
        let unlisted: Vec<_> = used.difference(&listed).collect();
        let unused: Vec<_> = listed.difference(&used).collect();
        assert!(
            unlisted.is_empty(),
            "add these codes to RULES: {unlisted:?}"
        );
        assert!(
            unused.is_empty(),
            "no finding cites these codes: {unused:?}"
        );
    }

    #[test]
    fn the_page_has_a_row_for_each_rule() {
        let page = markdown();
        for r in RULES {
            assert!(page.contains(&format!("| `{}` |", r.code)));
            assert_eq!(rule(r.code), Some(r));
        }
        assert_eq!(rule("CFG-10"), None);
    }
}
