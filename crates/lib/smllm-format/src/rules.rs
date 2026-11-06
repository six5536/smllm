//! The rules that `smllm validate` cites: each finding ends with the code of
//! the rule that it breaks, such as `(FMT-4)`. [`RULES`] lists every code that
//! a finding can carry, and [`markdown`] renders the list as the page that
//! `smllm info rules` prints and `docs/reference/validate-rules.md` holds. A test of the
//! CLI checks that the page is current, and a test of this module checks that
//! every code in the source is listed.
// sokf-implements: FMT-6_AC-1

/// One rule that a finding can cite.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rule {
    /// The code that a finding prints, such as `FMT-4`.
    pub code: &'static str,
    /// The rule's name.
    pub title: &'static str,
    /// What `smllm validate` checks under the rule.
    pub checks: &'static str,
}

/// Every rule that a finding can cite, grouped by contract: the code of a
/// rule is the key of the contract part whose promise the finding breaks.
pub const RULES: &[Rule] = &[
    Rule {
        code: "FMT-1",
        title: "The machine file",
        checks: "The file is YAML that parses, sets `meta.smllm: 1`, and has at least one state. Its keys are the known ones, each required key is present, and each value has the expected type. The machine's ID and its state names have the allowed characters, and no other machine in the config has the same ID or one that differs only in case. The file uses only the part of XState v5 that smllm supports: flat atomic and final states, without nested or parallel states, delays, or context, and with an `invoke` of only `src` and `input`.",
    },
    Rule {
        code: "FMT-2",
        title: "States and transitions",
        checks: "A final state has no `on` or `always`, isn't an entry point, and can be reached. A machine without a final state gets an info finding, because its instances never complete. At most one state is the fallback, and the fallback isn't final. A state doesn't handle a built-in event under `on`, a list of guarded transitions ends with one without a guard, and neither the ref param nor any other param has the name of a param of `enter`. An event with the ref param sets the ref. A state with an `invoke` handles an event of `source: host`, which carries the work's result, and each `invoke` names its `src`.",
    },
    Rule {
        code: "FMT-3",
        title: "Actions and guards",
        checks: "Each action is a `{type, params}` object of a known type, with the params that its type needs. A prompt has exactly one of `text` and `file`, a file path is relative to the file that gives it, and `setRef` appears only where an event carries the ref param. Each guard is a `{type, params}` object of a known type, with valid params. A `visits` guard counts a state that exists, and its `atLeast` is more than 0.",
    },
    Rule {
        code: "FMT-4",
        title: "Event and instance declarations",
        checks: "The ref param's name, and each declared event and param name, contain only letters, digits, `_`, and `-`. An event's params are an object of properties, each required param is in `properties`, and each `enum` and `pattern` is valid and allows at least one value. A state rewords only the events that it has a transition on, and only params that the event declares in `meta.events`. Each declared event is used by a state, and each ref pattern can match a ref. An event's `source` is `agent` or `host`, and a machine with an event of `source: host` gets an info finding, because only a host can fire the event, so the machine needs a host to drive it.",
    },
    Rule {
        code: "FMT-10",
        title: "JSON params",
        checks: "A `json` param takes no `enum` or `pattern`, a string param takes no `schema`, and a param that sets the ref is a string. A `json` param's schema, and each file that a relative `$ref` in it names, is a map or a JSON file that smllm can read, declares draft-07, 2020-12, or no draft, and is valid against its draft. A schema without a draft is 2020-12, or the draft of the schema that refers to it. Each `$ref` to another file is a path relative to the schema that holds it, because smllm doesn't fetch a schema from the network.",
    },
    Rule {
        code: "FMT-6",
        title: "Files that smllm reads",
        checks: "smllm can read each file that the config names.",
    },
    Rule {
        code: "FMT-7",
        title: "Combining configs",
        checks: "A config with errors doesn't replace the machines of the configs before it, and validate says which machines it leaves out and which machine replaces which.",
    },
    Rule {
        code: "AGT-1",
        title: "Author text",
        checks: "Author text, such as a prompt or an event's description, contains none of the closing tags `</smllm>`, `</instructions>`, and `</events>` that fence smllm's own text, because the agent couldn't then tell smllm's text from the author's. A finding under this rule is a warning.",
    },
    Rule {
        code: "AGT-2",
        title: "Events list",
        checks: "A built-in event changes only its description, never its params or its source.",
    },
    Rule {
        code: "AGT-5",
        title: "Transitions",
        checks: "The initial state and each transition's target exist, each state can be reached, and a transition that should re-enter its state names it as the target. A state with `always` has no `on` events, because smllm leaves it at once, and no chain of `always` transitions loops back to its start.",
    },
    Rule {
        code: "AGT-6",
        title: "Guards and actions",
        checks: "`meta.sharedActions` lists only states that exist.",
    },
    Rule {
        code: "MCP-1",
        title: "Param types",
        checks: "Each param has `type: string`, the default, or `type: json`.",
    },
    Rule {
        code: "CLI-3",
        title: "Configs and saved instances",
        checks: "smllm can read and parse each config file, and each saved instance is in a state that still exists.",
    },
    Rule {
        code: "CLI-11",
        title: "Command guards and actions",
        checks: "A command's `run` is a non-empty string, run by a shell, or a non-empty list, run without one. A list-form `run` doesn't rely on `$SMLLM_*` variables, because no shell expands them. A command's `timeoutSecs` is at least 1.",
    },
    Rule {
        code: "STORE-1",
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
         the rule that it breaks, such as `(FMT-4)`, and this page says what each rule checks. \
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
        let re = regex::Regex::new(r#""((?:FMT|AGT|MCP|CLI|STORE)-\d+)""#).unwrap();
        let mut codes = std::collections::BTreeSet::new();
        for f in files {
            let text = std::fs::read_to_string(&f).unwrap();
            for c in re.captures_iter(&text) {
                codes.insert(c[1].to_string());
            }
        }
        codes
    }

    // sokf-tests: FMT-6_AC-1
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
        assert_eq!(rule("FMT-5"), None);
    }
}
