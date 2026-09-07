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

<!-- comment: 08:00 - DOORS-->
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

<!-- comment: 09:55 - 10:05 - BREAK -->

<!-- comment: 10:00 - Part 2 -->

<!-- comment: slot - part-2 (wrap it in C) -->

<!-- comment: 11:00 - Part 3 -->

<!-- comment: slot - part-3 (bindings) -->

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
