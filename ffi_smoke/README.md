# FFI smoke harnesses (build-system evaluation only)

Not workshop content. These exist so the Bazel and Buck2 overlays each have
a real cross-language consumer to hang off a day's `:c_api` target — the edge
`just` cannot express as a dependency. Attendees never see this directory;
`just` never builds it.

`BUILD.bazel` and `BUCK` build the same `day_2015_12_06.c` against the same
generated header. They are kept side by side on purpose: the diff between
them is most of what separates the two build systems in practice.

Each harness deps on `//days/<day>:c_api` and nothing else. Note what is
absent from the sources: no `-L`, no `-rpath`, no `dlopen`, no computing
`target/debug` vs `target/release`. That absence is the whole argument
against either build system for the workshop and the whole argument for one
of them in CI.
