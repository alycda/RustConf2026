//! The puzzle, through a speech synthesiser — and the one variant in this
//! repo that does not solve it.
//!
//! That is on purpose, and it is the interesting part. Every other C library
//! here either fits the puzzle or is funny for not fitting. This one fits
//! *better than anything else in the repo* on the single hardest point, and
//! still cannot finish, and the reason it cannot finish is the same property
//! that makes it fit.
//!
//! # What it gets exactly right
//!
//! Part two's whole difficulty is that `1` and `one` mean the same thing while
//! being nothing alike as text. A text-to-speech front end has to solve that
//! before it can say anything, because both have to come out of a speaker
//! sounding identical. So it already holds the equivalence the puzzle is
//! about — not as a lookup table someone wrote for this, but as the ordinary
//! job of reading English aloud, with pronunciation dictionaries for a hundred
//! languages behind it:
//!
//! ```text
//! espeak_TextToPhonemes("1")  ->  wɒn        espeak_TextToPhonemes("one")  ->  wɒn
//! espeak_TextToPhonemes("2")  ->  tuː        espeak_TextToPhonemes("two")  ->  tuː
//! espeak_TextToPhonemes("8")  ->  eɪt        espeak_TextToPhonemes("eight") -> eɪt
//! ```
//!
//! Identical, once the stress marks are stripped. No table, no nine special
//! cases, no overlap rule. That is a library whose vocabulary contains the
//! exact sentence the puzzle needed — 2021-12-02's DuckDB window function,
//! one better.
//!
//! # What it cannot do, in increasing order of how stuck it is
//!
//! **Multi-digit runs are read as numbers.** `23seven` at position 0 is not
//! "two, then three"; it is "twenty three", `twɛnti…`, which does not begin
//! with `tuː`. Feeding suffixes one character at a time (the [`Speaker`]
//! below does exactly what `Day::digit_at` does) fixes the *last* digit and
//! not the first, because position 1 of `23` is `3seven` and reads fine while
//! position 0 never does.
//!
//! **Coarticulation blurs the boundary.** `twobfr` does not begin with `tuː`:
//! the engine re-syllabifies across the letters that follow, because that is
//! what speaking does. Sometimes it invents a digit that isn't there —
//! `djnrmpxjbsbpgzvtjkhq6pkkfshx` contains one `6` and reports an `8`, from
//! letters that happen to sound like one.
//!
//! **Part one is unavailable to *this* design.** Part one counts literal
//! digits and ignores the word `one`, and after phonemisation those are the
//! same three sounds — so the one question this module asks (phonemise the
//! whole suffix, prefix-match) cannot separate them, and both parts return
//! the same total.
//!
//! An earlier version of this comment said part one was unavailable *by
//! construction* and that no amount of tuning would fix it. That was too
//! strong, and the measurement that disproves it is in the
//! [two-window](#the-most-promising-lead-two-window-sizes) note below: a
//! one-character window separates them cleanly. The distinction part one is
//! made of survives after all — it is the *context* that destroys it, not the
//! phonemes.
//!
//! # The scoreboard
//!
//! On a real 1000-line puzzle input, against [`crate::sum_calibration_with_words_pure_rust`]:
//!
//! ```text
//! part two   728 / 1000 lines agree   (72.8%)
//! part one   276 / 1000               — the SAME output, scored against
//!                                       part one's answers, because there is
//!                                       only ever one output to score
//! ```
//!
//! [`agreement_with_pure_rust`] is that number, computed rather than quoted,
//! so the figure in `days/2023-12-01/README.md` can be checked and so anyone
//! who improves it can see by how much.
//!
//! **This is an open challenge, not a finished variant.** The failure modes
//! above are each pinned by a test named `unsolved_*`. A test in that group
//! starting to fail means someone got further, and the right response is to
//! update the number above, the README, and that test's name.
//!
//! ## The most promising lead: two window sizes
//!
//! Ask espeak twice per position — once with a **one-character** window and
//! once with the **full suffix** — and take the one-character answer if it is
//! a digit, falling back to the suffix otherwise.
//!
//! It works because a lone character is pronounced as itself or as its
//! *letter name*, and no letter name collides with a digit name:
//!
//! ```text
//! "1" -> wˈɒn      "o" -> ˈəʊ      "n" -> ˈɛn
//!                  "e" -> ˈiː      "t" -> tˈiː      "w" -> dˈʌbəljˌuː
//! ```
//!
//! So a one-character match identifies a *literal* digit — which is part one,
//! and is why the claim above had to be walked back. It also dissolves the
//! multi-digit run problem without any new vocabulary: `16` is read
//! position-by-position as `1` then `6`, while the *word* `sixteen` still
//! reaches the suffix window and matches `six`. That matters more than it
//! looks — see the wall below.
//!
//! Cost: two calls per position instead of one, on a variant already costing
//! ~600 µs per line. It does nothing for coarticulation (`twobfr`), which
//! would remain the last failure mode standing.
//!
//! ## The wall: re-parsing spoken numbers back into digits
//!
//! The obvious repair for `23seven` is to teach the reference table the
//! number words, so `twˈɛnti` maps back to a leading `2`. It needs less
//! machinery than it sounds like — the scan already visits every position, so
//! each position only needs the *leading* digit of the number starting there,
//! not a decomposition — and roughly eighteen more references (`ten`..
//! `nineteen`, `twenty`..`ninety`) would cover it. Two objections that look
//! fatal are not: input leading zeros are spoken (`07` -> `zˈiəɹəʊ sˈɛvən`,
//! `007` keeps both), and the zeros that `100` -> `wˈɒnhˈʌndɹɪd` swallows are
//! recovered anyway by the positions after the first.
//!
//! What kills it is a collision that no ordering survives:
//!
//! ```text
//! "16"      -> sˈɪkstiːn        "19"  -> nˈaɪntiːn
//! "sixteen" -> sˈɪkstiːn        "nine" -> nˈaɪn
//! "six"     -> sˈɪks
//! ```
//!
//! `16` and `sixteen` are the same sound and want different answers — the
//! digits `16` are a `1` and a `6`, the word `sixteen` is only a `6`.
//! Longest-match-first reads both as `1`; shortest-first reads both as `6`.
//! The puzzle's own statement example, `7pqrstsixteen`, is on the losing side
//! of longest-match. The information that separates them was destroyed by
//! phonemisation, which is why the two-window design above — which never has
//! to ask the question — is the better lead.
//!
//! ## Ruled out, with measurements
//!
//! - **SSML is not available on this API.** `espeak_TextToPhonemes` reads the
//!   tags aloud as words: `<say-as interpret-as="characters">23</say-as>`
//!   comes back as `sˈeɪaz ɪntˈɜːpɹɪtaz ˈiːkwəlz kˈaɹɪktəz twˈɛnti θɹˈiː
//!   slˈaʃ sˈeɪaz`. Still true with `espeakSSML` (0x10) OR'd into `textmode`
//!   — that flag belongs to `espeak_Synth`, and this function ignores it.
//! - **Injecting a leading `0` to force digit mode does not work.** A leading
//!   zero does trigger digit-by-digit reading, but only once the run reaches
//!   four digits: `0123` -> "zero one two three", while `016` -> "zero
//!   sixteen" and `023seven` -> "zero, twenty three, seven". Puzzle runs are
//!   one to three digits, so it fires exactly where it isn't needed.
//! - **Length alone never triggers digit mode.** Unlike engines that give up
//!   past four or five digits, espeak-ng scales all the way:
//!   `12345678901` -> "twelve billion three hundred and forty five
//!   million…". There is no threshold to reach.
//! - **Phone-number shapes are not recognised.** `555-1234` -> "five hundred
//!   and fifty five, dash, one thousand two hundred and thirty four".
//!
//! Still untried: `espeakPHONEMES_IPA`, comparing IPA rather than espeak's
//! own notation; `espeak_SetPhonemeTrace` with a per-phoneme callback, to get
//! *positions* back instead of prefix-matching a string, which would sidestep
//! prefix collisions entirely; or a voice whose dictionary treats digits
//! differently. If you want to explore text-shaping, `_` is the only
//! separator espeak splits on silently (`2_3` -> "two three"); space and a
//! non-grouping comma also work, while `-`, `.`, `:` and `/` each insert a
//! spoken word.
//!
//! Because it does not answer the puzzle, nothing routes to it —
//! `Solution::part1`/`part2` never call this module. A backend that returns a
//! confidently wrong number is worse than no backend, and the whole point of
//! keeping `..._pure_rust` alive is that a variant is allowed to be a
//! demonstration instead of an answer.

use std::ffi::{CStr, CString, c_char, c_int, c_void};
use std::sync::{Mutex, MutexGuard, OnceLock};

use crate::WORDS;

/// espeak-ng's audio-output mode. `SYNCHRONOUS` is the one that produces no
/// sound at all unless you ask for samples — this module never synthesises
/// audio, only the text front end that precedes it.
const AUDIO_OUTPUT_SYNCHRONOUS: c_int = 2;
/// Input is UTF-8 rather than the locale's 8-bit encoding.
const ESPEAK_CHARS_UTF8: c_int = 1;
/// Phoneme mode bit 1: separate phonemes with a space. Bit 0 (IPA) is
/// deliberately off — see the untried-ideas list in the module docs.
const PHONEME_MODE_SEPARATE: c_int = 0x02;

unsafe extern "C" {
    fn espeak_Initialize(
        output: c_int,
        buflength: c_int,
        path: *const c_char,
        options: c_int,
    ) -> c_int;
    fn espeak_SetVoiceByName(name: *const c_char) -> c_int;
    fn espeak_TextToPhonemes(
        textptr: *mut *const c_void,
        textmode: c_int,
        phonememode: c_int,
    ) -> *const c_char;
}

/// espeak-ng keeps its translator, its voice and its output buffer in process
/// globals, and `espeak_TextToPhonemes` returns a pointer into a *static*
/// buffer that the next call overwrites. Two threads in it at once is a data
/// race and a use-after-overwrite at the same time.
///
/// So every call in this module happens under one process-wide lock, and the
/// returned phonemes are copied into an owned `String` before it is released.
/// This is the sharpest contrast with the YARA variant next door, which gets a
/// fresh `YR_RULES` per solve and needs no lock at all: "does this library
/// have per-handle state or global state" is a question to answer before
/// designing around it, not after `cargo test` starts failing intermittently.
///
/// The lock guards a `bool`: whether espeak has been initialised. That is
/// part of the global state too, and `Speaker::new` consults it so that
/// `espeak_Initialize` runs once per process — see there for why twice is
/// not a reset but a corruption.
static ESPEAK: OnceLock<Mutex<bool>> = OnceLock::new();

fn lock() -> miette::Result<MutexGuard<'static, bool>> {
    let mutex = ESPEAK.get_or_init(|| Mutex::new(false));
    mutex
        .lock()
        .map_err(|_| miette::miette!("the espeak lock was poisoned by an earlier panic"))
}

/// Phonemises `text` and returns it with stress marks and spaces removed.
///
/// Stripping the stress marks is not cosmetic: `1` is `wˈɒn` with primary
/// stress and `one` is `wˌɒn` with secondary, the same sounds said with
/// different emphasis. Comparing them as-is would miss the very equivalence
/// this module exists to use.
///
/// # Safety contract this upholds
/// The caller must hold the lock; the returned pointer is only read before it
/// is released.
fn phonemes_of(_guard: &MutexGuard<'static, bool>, text: &CStr) -> String {
    let mut pointer = text.as_ptr().cast::<c_void>();

    // SAFETY: `pointer` addresses `text`'s NUL-terminated bytes, which outlive
    // the call; espeak advances it rather than writing through it. The result
    // points into espeak's static buffer, valid until the next call — and we
    // hold the lock, so there is no next call until this returns.
    let raw =
        unsafe { espeak_TextToPhonemes(&mut pointer, ESPEAK_CHARS_UTF8, PHONEME_MODE_SEPARATE) };
    if raw.is_null() {
        return String::new();
    }
    // SAFETY: non-null and NUL-terminated by espeak's contract.
    let phonemes = unsafe { CStr::from_ptr(raw) };

    phonemes
        .to_string_lossy()
        .chars()
        .filter(|c| !matches!(c, 'ˈ' | 'ˌ' | ' ' | '\n'))
        .collect()
}

/// An initialised espeak plus the nineteen reference pronunciations.
pub struct Speaker {
    /// Indexed the same way YARA's rule strings are: 0..=9 are the literal
    /// digits, 10..=18 are `one`..`nine`. Both halves are here even though
    /// nothing can tell them apart — see the module docs — because having them
    /// side by side is what makes that fact visible rather than assumed.
    references: Vec<(String, u32)>,
}

impl Speaker {
    /// Initialises espeak-ng once per process and phonemises the references.
    ///
    /// `path` is null: espeak finds its dictionaries at the location compiled
    /// into the library, which inside this repo's nix shell is the store path
    /// `shell.nix` put there. On a system install it is the packaged data
    /// directory. Nothing here hardcodes either.
    pub fn new() -> miette::Result<Self> {
        let mut guard = lock()?;

        // Once per process, never again. `espeak_Initialize` is not a
        // re-entrant reset: every call runs LoadPhData, which frees and
        // reallocates the phoneme data while pointers into the old block live
        // on in espeak's globals. Single-threaded that is invisible — the
        // freed block comes straight back at the same address. With other
        // threads allocating at the same time (every other test in this
        // crate) the new block lands elsewhere, and from the second init on
        // every phonemisation comes back empty. Measured: 0/10 failures with
        // the espeak tests alone or serial, 4/10 with the crate's other tests
        // alongside, 0/20 with this guard.
        if !*guard {
            // SAFETY: null `path` means "use the built-in data path"; the
            // other three arguments are plain integers.
            let rate =
                unsafe { espeak_Initialize(AUDIO_OUTPUT_SYNCHRONOUS, 0, std::ptr::null(), 0) };
            if rate < 0 {
                return Err(miette::miette!(
                    "espeak_Initialize failed ({rate}) — is espeak-ng's data directory present?"
                ));
            }

            let english = CString::new("en").expect("no NUL in a literal");
            // SAFETY: `english` is a live NUL-terminated string for the call.
            let status = unsafe { espeak_SetVoiceByName(english.as_ptr()) };
            if status != 0 {
                return Err(miette::miette!(
                    "espeak_SetVoiceByName(\"en\") failed ({status})"
                ));
            }
            *guard = true;
        }

        let mut references = Vec::with_capacity(19);
        for digit in 0..=9u32 {
            let text = CString::new(digit.to_string()).expect("no NUL in a digit");
            references.push((phonemes_of(&guard, &text), digit));
        }
        for (word, value) in WORDS {
            let text = CString::new(word).expect("no NUL in a literal");
            references.push((phonemes_of(&guard, &text), value));
        }

        drop(guard);

        if references.iter().any(|(p, _)| p.is_empty()) {
            return Err(miette::miette!(
                "espeak returned no phonemes for one of the nineteen references"
            ));
        }
        Ok(Self { references })
    }

    /// What the digit at `line[i..]` *sounds* like, if it sounds like one.
    ///
    /// The same shape as `Day::digit_at`: phonemise the suffix and ask whether
    /// it begins with a digit's pronunciation. Per-suffix rather than
    /// per-line because a whole line is one utterance, and an utterance
    /// containing `234` is read as a *number*.
    fn digit_at(&self, guard: &MutexGuard<'static, bool>, rest: &str) -> Option<u32> {
        let Ok(text) = CString::new(rest) else {
            return None;
        };
        let spoken = phonemes_of(guard, &text);
        if spoken.is_empty() {
            return None;
        }
        self.references
            .iter()
            .find(|(reference, _)| spoken.starts_with(reference.as_str()))
            .map(|(_, value)| *value)
    }

    /// One line's calibration value, as heard rather than as read.
    pub fn calibration_value(&self, line: &str) -> miette::Result<u32> {
        let guard = lock()?;
        let mut first = None;
        let mut last = None;

        for (i, _) in line.char_indices() {
            if let Some(value) = self.digit_at(&guard, &line[i..]) {
                first.get_or_insert(value);
                last = Some(value);
            }
        }

        Ok(first.unwrap_or(0) * 10 + last.unwrap_or(0))
    }
}

/// Part two, as far as a speech synthesiser gets. See the module docs for how
/// far that is and why.
///
/// There is no `sum_calibration_via_espeak` for part one. Not an omission: a
/// phoneme cannot distinguish `1` from `one`, so part one has no meaning on
/// this side of the boundary.
pub fn sum_calibration_with_words_via_espeak(lines: &[String]) -> miette::Result<u32> {
    let speaker = Speaker::new()?;
    let mut total: u32 = 0;
    for line in lines {
        total = total
            .checked_add(speaker.calibration_value(line)?)
            .ok_or_else(|| miette::miette!("calibration sum overflows a u32"))?;
    }
    Ok(total)
}

/// How many of `lines` this variant gets right, and how many there were.
///
/// The scoreboard for the open challenge in the module docs — a computed
/// number rather than a quoted one, so the README's figure can be rechecked
/// and an improvement can be measured rather than claimed.
pub fn agreement_with_pure_rust(lines: &[String]) -> miette::Result<(usize, usize)> {
    let speaker = Speaker::new()?;
    let mut agreed = 0;
    for line in lines {
        let heard = speaker.calibration_value(line)?;
        let read = crate::Day::calibration_value(line, true);
        if heard == read {
            agreed += 1;
        }
    }
    Ok((agreed, lines.len()))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The claim the whole variant rests on: a speech synthesiser already
    /// knows `1` and `one` are the same thing, and says so in its own
    /// vocabulary without being told about this puzzle.
    #[test]
    fn a_digit_and_its_word_are_the_same_sounds() -> miette::Result<()> {
        let speaker = Speaker::new()?;
        for value in 1..=9usize {
            let (digit, _) = &speaker.references[value];
            let (word, _) = &speaker.references[9 + value];
            assert_eq!(
                digit,
                word,
                "{value} and {} should phonemise alike",
                WORDS[value - 1].0
            );
        }
        Ok(())
    }

    /// Lines it does get right — the ordinary case, where each digit or word
    /// is bounded by letters that don't reshape it.
    #[test]
    fn plain_lines_are_heard_correctly() -> miette::Result<()> {
        let speaker = Speaker::new()?;
        assert_eq!(speaker.calibration_value("two1nine")?, 29);
        assert_eq!(speaker.calibration_value("abcone2threexyz")?, 13);
        assert_eq!(speaker.calibration_value("treb7uchet")?, 77);
        Ok(())
    }

    // ---- pinned failure modes: the open challenge ----
    //
    // These assert what espeak currently does, which is the wrong answer. A
    // test here starting to FAIL is the good outcome: it means someone got
    // further. When that happens, rename it, fix the figure in the module
    // docs and in days/2023-12-01/README.md, and say what worked.

    /// A run of digits is read as a number, so the first digit's own
    /// pronunciation never appears: `23seven` opens with `twɛnti`, not `tuː`.
    /// Correct answer 27; espeak hears 37, having missed the 2 and found the 3.
    #[test]
    fn unsolved_multi_digit_runs_become_numbers() -> miette::Result<()> {
        let speaker = Speaker::new()?;
        assert_eq!(speaker.calibration_value("23seven")?, 37);
        assert_eq!(crate::Day::calibration_value("23seven", true), 27);
        Ok(())
    }

    /// Speaking re-syllabifies across whatever follows, so a word that runs
    /// into more letters stops sounding like itself: `twobfr` does not open
    /// with `tuː`.
    #[test]
    fn unsolved_coarticulation_hides_a_word() -> miette::Result<()> {
        let speaker = Speaker::new()?;
        assert_eq!(speaker.calibration_value("eighth33twobfr")?, 83);
        assert_eq!(crate::Day::calibration_value("eighth33twobfr", true), 82);
        Ok(())
    }

    /// The same mechanism in the other direction: letters that happen to sound
    /// like a digit report one that was never written. This line contains a
    /// single `6` and nothing else numeric.
    #[test]
    fn unsolved_letters_can_sound_like_a_digit() -> miette::Result<()> {
        let speaker = Speaker::new()?;
        let line = "djnrmpxjbsbpgzvtjkhq6pkkfshx";
        assert_eq!(crate::Day::calibration_value(line, true), 66);
        assert_ne!(
            speaker.calibration_value(line)?,
            66,
            "if this now agrees, the false-positive problem is solved"
        );
        Ok(())
    }

    /// Part one counts literal digits only, and in the *whole-suffix* window
    /// this module asks in, `one` and `1` are the same three sounds — so it
    /// answers part two whatever it is asked.
    ///
    /// This test used to claim no amount of tuning could fix that. Not so:
    /// the last clause of that claim — "it would need a different question
    /// put to the library" — turned out to be the whole answer. Ask with a
    /// one-character window and `o` is `ˈəʊ`, not `wˈɒn`, so a match there is
    /// a literal digit. See the two-window note in the module docs. This is
    /// therefore an `unsolved_*` with a known route, unlike the other three.
    #[test]
    fn unsolved_part_one_is_indistinguishable_from_part_two() -> miette::Result<()> {
        let speaker = Speaker::new()?;
        // `one` is not a part-one digit, but it sounds exactly like `1`.
        assert_eq!(speaker.calibration_value("one")?, 11);
        assert_eq!(crate::Day::calibration_value("one", false), 0);
        Ok(())
    }

    /// The scoreboard function itself, on the statement example.
    ///
    /// Seven out of seven — and that is the warning, not the reassurance. The
    /// published example is short lines with well-separated digits and no runs
    /// longer than `234`; it is exactly the shape espeak handles. A real
    /// 1000-line input, which is 30-to-45-character letter soup, agrees on
    /// about 73%. A variant validated only against the statement example would
    /// have looked finished.
    #[test]
    fn the_scoreboard_counts_lines() -> miette::Result<()> {
        let lines: Vec<String> = "two1nine\neightwothree\nabcone2threexyz\nxtwone3four\n4nineeightseven2\nzoneight234\n7pqrstsixteen"
            .lines()
            .map(str::to_string)
            .collect();
        let (agreed, total) = agreement_with_pure_rust(&lines)?;
        assert_eq!(total, 7);
        // Pinned so a regression is visible; the interesting number is the one
        // a real input produces, which no test can carry.
        assert_eq!(agreed, 7, "espeak agrees on all 7 example lines");
        Ok(())
    }
}
