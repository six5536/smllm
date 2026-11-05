# @six5536/smllm-darwin-arm64

This package holds the prebuilt `smllm` binary for darwin-arm64 (macOS on Apple silicon). It is an
internal part of [`@six5536/smllm`](https://www.npmjs.com/package/@six5536/smllm),
which installs it as an optional dependency on a matching machine. Don't depend
on this package directly. Install `@six5536/smllm` instead.

smllm is licensed under the MIT licence. Its source is in the
[smllm repository](https://github.com/six5536/smllm).

## Third-party notices

The binary embeds the JSON Schema meta-schemas of draft-07 and draft 2020-12, which json-schema.org
publishes in the [json-schema-spec repository](https://github.com/json-schema-org/json-schema-spec),
to check the schema of each JSON param. They are under the BSD 3-Clause licence:

```text
Copyright (c) 2022 JSON Schema Specification Authors

Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following conditions are
met:

1. Redistributions of source code must retain the above copyright
notice, this list of conditions and the following disclaimer.

2. Redistributions in binary form must reproduce the above copyright
notice, this list of conditions and the following disclaimer in the
documentation and/or other materials provided with the distribution.

3. Neither the name of the copyright holder nor the names of its
contributors may be used to endorse or promote products derived from
this software without specific prior written permission.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
"AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED
TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF
LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING
NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
```
