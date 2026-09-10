# FFI smoke harnesses (Bazel evaluation only)

Not workshop content. These exist so the Bazel overlay has a real
cross-language consumer to hang off a day's `:c_api` target — the edge
`just` cannot express as a dependency. Attendees never see this directory;
`just` never builds it.

Each harness deps on `//days/<day>:c_api` and nothing else. Note what is
absent from the sources: no `-L`, no `-rpath`, no `dlopen`, no computing
`target/debug` vs `target/release`. That absence is the whole argument
against Bazel for the workshop and the whole argument for it in CI.
