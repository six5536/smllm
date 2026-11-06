---
type: ErrorReference
id: error-fmt-10
title: 'FMT-10: JSON params'
description: The findings that smllm validate reports when a param that takes a JSON value, or its schema, breaks a rule of JSON params.
code: FMT-10
---

# FMT-10: JSON params

A finding of `FMT-10` means that a param with `type: json` has a constraint that only a string param takes, or that smllm can't read or use the param's JSON Schema. The [rules page][sokf:reference-validate-rules] says what the rule checks.

## Messages

```text
<file>:<line>: error: <path>.schema: param <param> is a string, so it takes no schema. Set type: json for a param that takes a JSON value with a schema (FMT-10)
<file>:<line>: error: <path>.enum: param <param> takes a JSON value, so it takes no enum. Give the constraint in the param's schema (FMT-10)
<file>:<line>: error: <path>.pattern: param <param> takes a JSON value, so it takes no pattern. Give the constraint in the param's schema (FMT-10)
<file>:<line>: error: <path>: <event> sets the ref, so its param <param> must be a string. Remove type: json, or give the ref another param (FMT-10)
<file>:<line>: error: <path>.schema: the schema file <schema file> does not exist (FMT-10)
<file>:<line>: error: <path>.schema: <schema> can't be read: <error> (FMT-10)
<file>:<line>: error: <path>.schema: <schema> is not JSON: <error> (FMT-10)
<file>:<line>: error: <path>.schema: <schema> declares the draft <uri>, but smllm reads only draft-07 and 2020-12. Set $schema to http://json-schema.org/draft-07/schema# or https://json-schema.org/draft/2020-12/schema (FMT-10)
<file>:<line>: error: <path>.schema: <schema> has a $schema of <value>, but $schema must be a string (FMT-10)
<file>:<line>: error: <path>.schema: <schema> is not a valid <draft> schema: <error> (FMT-10)
<file>:<line>: error: <path>.schema: ref <ref> in <schema> names a file that does not exist (FMT-10)
<file>:<line>: error: <path>.schema: ref <ref> in <schema> resolves to no bundled file. smllm doesn't fetch a schema from the network: write the ref as a path relative to the schema that holds it (FMT-10)
```

## Cause

In each message, `<path>` is the param, such as `meta.events.planned.params.properties.plan`. `<schema>` is `schema` and the name of a schema file, or `the inline schema` for a schema written in the machine file.

Each message has its own cause:

- A string param has a `schema`.
- A JSON param has an `enum`.
- A JSON param has a `pattern`.
- An event that sets the ref gives the ref param `type: json`. A ref is a string.
- The schema file that `schema` names doesn't exist. smllm reads the path from the folder of the machine file.
- smllm can't read the schema file, for example because the path names a folder.
- The schema file isn't JSON.
- The schema's `$schema` declares a draft other than draft-07 and 2020-12.
- The schema's `$schema` isn't a string.
- The schema isn't valid against its draft. A schema without a `$schema` is 2020-12, or the draft of the schema that refers to it.
- A `$ref` names a file that doesn't exist. smllm reads the path from the folder of the schema that holds the `$ref`.
- A `$ref` is a URL that names no schema file of the param. smllm doesn't fetch a schema from the network.

## Fix

Make the change that the message names:

- Set `type: json` on the param, or remove `schema`.
- Remove `enum`, and give the values in the schema.
- Remove `pattern`, and give the pattern in the schema.
- Remove `type: json` from the ref param, or set the ref from another param.
- Correct the path, relative to the folder of the machine file, or create the file.
- Correct the path, so that it names a file that smllm can read.
- Correct the JSON.
- Set `$schema` to `http://json-schema.org/draft-07/schema#` or `https://json-schema.org/draft/2020-12/schema`, or remove it.
- Write `$schema` as a string.
- Correct the schema at the place that the message names.
- Correct the path of the `$ref`, or create the file.
- Save the schema as a file next to the schema that refers to it, and write the `$ref` as a relative path.

## Example

This machine's schema file declares draft-04:

**`.smllm/schemas/plan.json`**

```json
{
  "$schema": "http://json-schema.org/draft-04/schema#",
  "type": "object",
  "required": ["steps"],
  "properties": {
    "steps": { "type": "array", "items": { "type": "string" } }
  }
}
```

**`.smllm/plan.smllm.yaml`**

```yaml
id: plan
initial: PLAN
meta:
  smllm: 1
  events:
    planned:
      params:
        type: object
        required: [plan]
        properties:
          plan:
            type: json
            schema: schemas/plan.json
states:
  PLAN:
    on:
      planned: DONE
  DONE:
    type: final
```

`smllm validate` reports the draft:

```console
$ smllm validate .smllm/plan.smllm.yaml
.smllm/plan.smllm.yaml:13: error: meta.events.planned.params.properties.plan.schema: schema schemas/plan.json declares the draft http://json-schema.org/draft-04/schema#, but smllm reads only draft-07 and 2020-12. Set $schema to http://json-schema.org/draft-07/schema# or https://json-schema.org/draft/2020-12/schema (FMT-10)
1 error, 0 warnings, 0 info
[exit 1]
```

The fixed schema file declares draft-07:

**`.smllm/schemas/plan.json`**

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["steps"],
  "properties": {
    "steps": { "type": "array", "items": { "type": "string" } }
  }
}
```

`smllm validate` reports no finding:

```console
$ smllm validate .smllm/plan.smllm.yaml
0 errors, 0 warnings, 0 info
```

<!-- sokf:links -->

[sokf:reference-validate-rules]: /docs/reference/validate-rules.md
