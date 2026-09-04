# CI overlays

`exercises/` in this directory mirrors the attendee scaffold at the repo
root, with every TODO filled in once: Ex 1 solved (Advent of Code 2025
day 3, ported from `alycda/learning-in-public`), Ex 2's wrapper and
harness, and all four Ex 3 tracks. The Verify workflow copies it over
`exercises/` at test time —

    cp -R .github/ci/exercises/. exercises/

— and then runs the commands the READMEs and file headers tell attendees
to run, so a green cell means the attendee path works on that OS with
that track, not that some CI-only path does.

Kept outside `exercises/` so it never appears in the tree attendees work
in, and so a scaffold edit shows up as a red cell rather than as a
surprise in the room. When the scaffold changes shape, change the
matching overlay file; when a track's documented command changes, change
the workflow step that runs it. 2025 is deliberately not on the workshop
day menu, so nothing here spoils a day an attendee picks.

To reproduce a cell locally: apply the copy above in a scratch checkout
(not in your working copy — jj would snapshot the solved files into your
commit), then run `exercises/ex2-c-glue/build-and-test.sh` and the one
track command from that track file's header.
