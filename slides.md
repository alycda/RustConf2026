---
theme:
    name: catppuccin-mocha
    override:
        footer:
            style: template
            left: "FFI Playground"
            center: github.com/alycda/RustConf2026
            # right: "Montréal: 2026-09-08"
---

<!-- comment: 08:00 - 09:00 - DOORS -->
<!-- comment: slide_background_color 0b112e -->
<!-- font_size: 7 -->

![image:w:30%](./img/qr-code.png)

# Using Advent of Code as an FFI Playground
## RustConf 2026

### Montréal
#### 2026-09-08

##### Alyssa Evans

<!-- no_footer -->

<!-- comment: 09:00 - Part 1 -->

<!-- comment: slot - speaker card -->

<!-- comment: slot - backstory -->

<!-- comment: slot - who am I -->

<!-- comment: slot - why are we here -->

<!-- comment: slot - agenda -->

<!-- comment: slot - self-check -->

<!-- comment: slot - part-1 (why is FFI hard) -->

<!-- comment: slot - exercise-1 (solve it in Rust) -->

<!-- comment: slot - exercise-1-bonus (find a C library, try to load it) -->

<!-- comment: 09:55 - 10:05 - BREAK -->

<!-- comment: 10:05 - Part 2 -->

<!-- comment: slot - lecture-2 (C as the bridge — live demo) -->

<!-- comment: slot - part-2 (wrap it in C) -->

<!-- comment: 10:55 - 11:05 - BREAK -->

<!-- comment: 11:05 - Part 3 -->

<!-- comment: slot - lecture-3 (one header, four runtimes) -->

<!-- comment: slot - part-3 (bindings) -->

<!-- comment: slot - exercise-3-bonus (try another language) -->

<!-- comment: 12:00 - Wrap -->

<!-- comment: slot - debrief -->

<!-- comment: 12:30 - END -->

<!-- end_slide -->

<!-- alignment: center -->
![image:w:40%](./workshop.png)

<!-- speaker_note: |

    (30s) Welcome to RustConf 2026 Workshops. I want to thank you all for being here with me today. My name is Alyssa and I'm going to share with you my process on how to break things in Rust while talking to other languages and their runtimes; and to help you choose your own adventure with FFI and Advent of Code.

    -->

<!-- end_slide -->

What Is This?
===

![image:w:40%](./img/rustla-talk.jpg)

<!-- font_size: 2 -->
<!-- alignment: center -->
Prior Art: [alycda/aoc-ffi](https://github.com/alycda/aoc-ffi) · [alycda/AoC-Ornaments](https://github.com/alycda/AoC-Ornaments)

<!-- end_slide -->

What Is This?
===

![image:w:40%](./img/rustla-room.jpg)

<!-- font_size: 2 -->
<!-- alignment: center -->
Prior Art: [alycda/aoc-ffi](https://github.com/alycda/aoc-ffi) · [alycda/AoC-Ornaments](https://github.com/alycda/AoC-Ornaments)

photo: [Lawrence Harvey](https://www.lawrenceharvey.com/newsroom/rust-la-kicks-off-2026-with-a-packed-community-meetup-in-los-angeles)

<!-- end_slide -->

Who Am I?
===


Staff Software Engineer, SDKs at [Ditto](https://ditto.com) 

<!-- speaker_note: |

    Last year I joined Ditto as a Staff Software Engineer. FFI is literally my day job. 
    


    Before Ditto, I spent 6 years in Free Ad-Supported Streaming TV (FAST) 

        building Web Applications for Connected TVs and game consoles. 

        Then I became _that engineer_ who kept pushing to adopt Rust.

    -->

<!-- end_slide -->

Why Are We Here Today?
===

To break things, on purpose; and document the messy middle.

<!-- speaker_note: |

    ## Description

    Using Advent of Code (AoC) puzzles as the substrate, participants build a working Rust FFI library from scratch, wrapping a real AoC solution in a C glue layer and calling it from multiple different target languages. By the end of the workshop, attendees leave with a working multi-language project, a replicable methodology, and hands-on intuition for the pitfalls that make production FFI hard.

    AoC problems are uniquely well-suited for FFI practice because they have diverse input/output types (primitives, strings, iterators), a motivating narrative that makes repetition feel worthwhile, no production baggage — you can break things freely — and progressively increasing complexity across 12-25 days of problems each year. Unlike algorithm-focused competitive programming, this approach uses AoC’s rich problem variety to stress-test real cross-platform FFI patterns: string encoding mismatches between Java and Swift, async model incompatibilities, and the lowest-common-denominator constraints that make production FFI viable.

    ## Learning Outcomes

    After this workshop, attendees will be able to:

    - Use Advent of Code as a structured, low-stakes FFI practice environment — progressing from primitives to strings to async across multiple target languages.
    - Design a C glue layer that accommodates the lowest-common-denominator constraints of diverse language runtimes, using cbindgen or UniFFI.
    - Avoid common FFI pitfalls: string encoding mismatches between Swift/Java, async model incompatibilities, over-exposing Rust idioms that don’t translate across language boundaries.
    - Evaluate whether FFI is the right architectural choice for a given situation, including honest assessment of the maintenance burden, onboarding cost, and performance trade-offs.
    - Replicate the AoC-as-FFI-playground methodology in their own learning or team onboarding — with a structured progression and a working starter template to build from.
  -->

<!-- end_slide -->

Agenda
===

<!-- new_line -->

- **NOW** — self-check + pick your day — *you*
- **9:15** — why FFI is harder than it looks — *me*
- **9:35** — Ex 1 · your day, pure Rust — *you*
- **9:55** — break
- **10:05** — C as the bridge (live demo) — *me*
- **10:25** — Ex 2 · wrap it in a C boundary — *you*
- **10:55** — break
- **11:05** — one header, four runtimes — *me*
- **11:30** — Ex 3 · bindings in YOUR language — *you*
- **12:00** — debrief: what broke? — *all of us*

<!-- end_slide -->

Self-Check
===

```sh
git clone https://github.com/alycda/RustConf2026
cd RustConf2026 # optional: direnv allow
just check # or ./scripts/self-check.sh
```

<!-- new_line -->

<!-- speaker_note: |

 -->

<!-- end_slide -->

Why Is This Hard?
===

An FFI signature is a **treaty** between two runtimes that disagree about memory, types, errors, and encoding —

<!-- new_line -->

and neither side can enforce it.

<!-- speaker_note: |

    (3m) Slow down — this is the thesis of the whole morning.

    The compiler checks YOUR side of the treaty only. We're making a promise to the compiler that we know what we're doing — and we MUST keep it.

    That's why we test from the OTHER side today.

 -->

<!-- end_slide -->

Four Disagreements
===

<!-- incremental_lists: true -->

1. **Memory** — who allocates, who frees, who's *sure*?
2. **Types** — Rust's `String` doesn't exist over there. Neither does `Result`.
3. **Errors** — panics don't cross. Exceptions don't cross. Integers cross.
4. **Encoding** — "it's just a string"

<!-- incremental_lists: false -->

<!-- speaker_note: |

    (5m) After 20 years of breaking things on the internet, encoding bugs are still the ones that ship silently.

    Errors: so what actually crosses the boundary? Integers.

 -->

<!-- end_slide -->

"It's Just a String"
===

<!-- incremental_lists: true -->

* **Rust** — UTF-8; `CString` refuses interior NUL — validation at the boundary
* **C** — bytes + NUL terminator, no promises whatsoever
* **Swift** — UTF-8 inside (since Swift 5); the NSString bridge is UTF-16 — hidden work
* **JVM** — **modified** UTF-8 via JNI. Not quite UTF-8. Really.
* **Python** — unicode object; you `.encode()` first — explicit, honest
* **Dart** — UTF-16 code units; `toNativeUtf8()` — and **you** free it

<!-- incremental_lists: false -->

<!-- speaker_note: |

    (7m) Modified UTF-8: NUL encodes as TWO bytes, and supplementary characters differ too. Works with ASCII in every test — then someone's name has an emoji.

    Sometimes the failure is silent corruption, not a crash. The crash is the easy case.

    ---

    Rust String CAN hold interior NUL — CString is what refuses. Swift String is UTF-8 internally since Swift 5; UTF-16 lives in the ObjC bridge.

 -->
