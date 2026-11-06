---
type: HowTo
id: how-to-take-json-in-a-param
title: Take JSON in a param
description: Give an event a param that takes a JSON value, such as a plan with a list of steps, and describe its shape with a JSON Schema.
order: "60"
---

# Take JSON in a param

When an event needs more than a line of text from the agent, such as a plan with a list of steps, give the event a param of `type: json`. The agent then passes a JSON value, and a JSON Schema tells it the shape of the value.

## Steps

The steps use a schema of the value, in a JSON file beside the machine. This schema describes a plan as an object with a list of steps:

**`.smllm/plan.schema.json`**

```json
{
  "type": "object",
  "properties": {
    "steps": { "type": "array", "items": { "type": "string" }, "minItems": 1 }
  },
  "required": ["steps"]
}
```

The config lists the machine. The machine declares the param in the event's `params`, with `type: json` and the schema's path, relative to the machine file:

**`.smllm/config.toml`**

```toml
[machines]
files = ["plan.smllm.yaml"]
```

**`.smllm/plan.smllm.yaml`**

```yaml
id: plan
description: Plan a change as a list of steps, then carry it out.
initial: PLAN
meta:
  smllm: 1
  events:
    planned:
      description: Fire with the plan, after you write it.
      params:
        type: object
        properties:
          plan:
            type: json
            description: The plan, as an object with its steps in order.
            schema: plan.schema.json
        required: [plan]
    done:
      description: Fire after the last step.
states:
  PLAN:
    meta: { entryPoint: true }
    entry: { type: prompt, params: { text: "Write a plan, then fire planned with it." } }
    on:
      planned: WORK
  WORK:
    entry: { type: prompt, params: { text: "Carry out the plan, one step at a time." } }
    on:
      done: DONE
  DONE:
    type: final
```

`schema` can also hold the schema itself as a map, in place of a path. A schema declares draft-07 or 2020-12 in `$schema`, or no draft, which smllm reads as 2020-12.

1. Check the machine. `smllm validate` reads the schema, and reports an error when the schema is invalid or names a file that smllm can't read:

   ```console
   $ smllm validate
   0 errors, 0 warnings, 0 info
   ```

2. Enter the machine, as the agent does:

   ```console
   $ smllm fire --session sm-nqvfvw enter --param stateMachine=plan
   <smllm>
   session sm-nqvfvw · plan › PLAN · instance i-0ekd6p
   Arrived by: enter (new instance)
   Params: stateMachine = "plan"
   <instructions>
   Write a plan, then fire planned with it.
   </instructions>
   </smllm>
   ```

   `sm-nqvfvw` is the key of the session. Your key differs.

   The events list shows the param as a JSON value, with its schema, so the agent knows what to pass:

   <!-- example:
     hook stop
   -->
   ```text
   <smllm>
   session sm-nqvfvw · plan › PLAN · instance i-0ekd6p
   Fire one event: smllm({ session: "sm-nqvfvw", event, params })
   <events>
   - planned: Fire with the plan, after you write it.
       plan (required, a JSON value, schema: {"type":"object","properties":{"steps":{"type":"array","items":{"type":"string"},"minItems":1}},"required":["steps"]}): The plan, as an object with its steps in order.
   - yield: Stop for now and stay in PLAN.
       note (optional): What you are waiting for.
   - pause: Put instance i-0ekd6p aside and return to idle.
   - unmatched: The request fits none of these; handle it from idle, then resume.
   </events>
   </smllm>
   ```

3. Fire the event with a JSON value. The agent passes the value as JSON in the smllm tool's `params`. From the shell, `--param` reads the text after `=` as JSON, so quote it for the shell. smllm checks the value against the schema, and refuses a value that doesn't match:

   ```console
   $ smllm fire --session sm-nqvfvw planned --param 'plan={"steps": []}'
   <smllm>
   session sm-nqvfvw · plan › PLAN · instance i-0ekd6p
   error: planned param plan doesn't match its schema. At /steps: [] has less than 1 item
   Fire one event: smllm({ session: "sm-nqvfvw", event, params })
   <events>
   - planned: Fire with the plan, after you write it.
       plan (required, a JSON value, schema: {"type":"object","properties":{"steps":{"type":"array","items":{"type":"string"},"minItems":1}},"required":["steps"]}): The plan, as an object with its steps in order.
   - yield: Stop for now and stay in PLAN.
       note (optional): What you are waiting for.
   - pause: Put instance i-0ekd6p aside and return to idle.
   - unmatched: The request fits none of these; handle it from idle, then resume.
   </events>
   </smllm>
   [exit 1]
   ```

   Each sentence after the error names a part of the value that fails, by its JSON path. A value that matches the schema moves the instance on:

   ```console
   $ smllm fire --session sm-nqvfvw planned --param 'plan={"steps": ["Write a test that fails", "Fix the parser"]}'
   <smllm>
   session sm-nqvfvw · plan › WORK · instance i-0ekd6p
   Arrived by: planned from PLAN
   Params: plan = {"steps":["Write a test that fails","Fix the parser"]}
   <instructions>
   Carry out the plan, one step at a time.
   </instructions>
   </smllm>
   ```

   A command action or guard of the transition reads the value as compact JSON text from the variable `SMLLM_PARAM_PLAN`, the param's name in upper snake case.

## Troubleshooting

- smllm refuses the event with `planned param plan must be a JSON value`: the text isn't JSON. Quote a string as JSON, such as `plan="\"one step\""`, and quote the whole param for your shell.
- smllm refuses the event with `planned param plan doesn't match its schema`: the value is JSON, but a part of it fails the schema. Correct each part that the error names by its JSON path, and fire the event again.
- A string param with `schema`, or a JSON param with `enum` or `pattern`, is an error: `schema` belongs only to a JSON param, and `enum` and `pattern` only to a string param.

## See also

- [The machine file][sokf:reference-machine-file]
- [The rules of smllm validate][sokf:reference-validate-rules]
- [The agent's text][sokf:reference-agent-text]

<!-- sokf:links -->

[sokf:reference-agent-text]: /docs/reference/agent-text.md
[sokf:reference-machine-file]: /docs/reference/machine-file.md
[sokf:reference-validate-rules]: /docs/reference/validate-rules.md
