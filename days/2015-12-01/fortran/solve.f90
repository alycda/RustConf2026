! Exercise 3 (Fortran track): call the Exercise 2 C API from Fortran.
!
! Where this track differs from the others, and it is a real difference:
!
!   python/cffi   reads the real cbindgen header at runtime and derives its
!                 declarations from it.
!   dart:ffi      needs every signature written twice, hand-transcribed, and
!                 checks neither copy against the header.
!   JNA (Kotlin)  needs each function written once and marshals by convention.
!   Swift         needs no signature at all — clang reads the header.
!   Fortran (here)
!                 needs every signature written once, by hand, in the
!                 `interface` block below — and then *enforces* it. gfortran
!                 type-checks every call against that block, so a wrong
!                 transcription is a compile error, not a runtime surprise.
!                 Transcribed like Dart, checked like Swift; checked against
!                 what we wrote down, not against what Rust exports. A
!                 transcription that is wrong *and self-consistent* still
!                 compiles and still corrupts.
!
! The other thing only this track can show: nothing here allocates an `int *`.
! Python does `ffi.new("int *")`, Dart `calloc`s, Swift takes `&slot`. Fortran
! passes everything by reference already — `integer(c_int), intent(out) :: x`
! *is* the out-parameter, and the call site is the puzzle's own variable. The
! C API's most C-shaped design decision costs this language nothing.
!
! The treaty itself is in the language standard, not a library: ISO_C_BINDING
! and `bind(C)` are Fortran 2003. No package manager, no jar, no pinned
! dependency — `use iso_c_binding` and gfortran.
!
! Run via: just days fortran-demo 2015-12-01 (regenerates the header, builds
! the cdylib, compiles and runs this).

program solve
   use iso_c_binding, only: c_char, c_int, c_null_char
   use iso_fortran_env, only: error_unit
   implicit none

   ! The hand-written half of the treaty. Compare
   ! include/aoc_2015_12_01.h, which cbindgen generates from src/c_api.rs:
   !
   !   int aoc_2015_12_01_part1(const char *input, int *out_floor);
   !
   ! `bind(C, name=...)` spells the symbol out rather than relying on
   ! gfortran's default mangling, which lowercases the Fortran name and
   ! appends an underscore — `aoc_2015_12_01_part1_`, which the cdylib does
   ! not export.
   !
   ! `import ::` is not optional: an interface block is a scoping unit of its
   ! own and does not inherit the program's `use iso_c_binding`. Without it,
   ! `c_int` is an undefined name.
   !
   ! `character(kind=c_char), dimension(*)` — an assumed-size array of
   ! characters — is what `const char *` binds to. `character(len=*)` would
   ! compile and would be wrong: gfortran's ABI passes the length of every
   ! `character` argument as a hidden trailing `size_t`, after all the visible
   ! ones, and the C side never declared a parameter to receive it. That is
   ! the first sight of the hidden-argument rule, from the safe side — the
   ! call still works, because C ignores an argument it was not told about,
   ! but the signature we wrote and the signature C sees have stopped
   ! matching, and nothing checks that.
   interface
      function aoc_2015_12_01_part1(input, out_floor) &
         bind(C, name="aoc_2015_12_01_part1") result(status)
         import :: c_char, c_int
         character(kind=c_char), dimension(*), intent(in) :: input
         integer(c_int), intent(out) :: out_floor
         integer(c_int) :: status
      end function aoc_2015_12_01_part1

      function aoc_2015_12_01_part2(input, out_position) &
         bind(C, name="aoc_2015_12_01_part2") result(status)
         import :: c_char, c_int
         character(kind=c_char), dimension(*), intent(in) :: input
         integer(c_int), intent(out) :: out_position
         integer(c_int) :: status
      end function aoc_2015_12_01_part2
   end interface

   character(len=:), allocatable :: text
   integer(c_int) :: answer

   text = read_file(repo_path('inputs/2015-12-01.txt'))

   ! `trim(text) // c_null_char` is the NUL written by hand, for the third
   ! time in this workshop (Python's `.encode()`, Dart's `toNativeUtf8`, this).
   ! A Fortran string carries a length and no terminator, so there is nothing
   ! for `CStr::from_ptr` to stop at until we append one.
   !
   ! `trim` strips trailing *blanks*, and is defensive rather than decorative
   ! here: `read_file` below allocates the string to the file's exact size, so
   ! there is nothing to strip. Any ordinary Fortran read does not — a
   ! `character(len=4096)` buffer holding five characters holds 4091 spaces
   ! after them, and they are part of the string. Drop the `trim` there and
   ! Rust receives four kilobytes of padding that its parser dutifully maps to
   ! zeroes. Note what `trim` does *not* remove: a trailing newline is not a
   ! blank, so the input's final `\n` crosses the boundary intact. This day's
   ! parser maps anything that is not a paren to 0, so it costs nothing; a day
   ! that split on lines would care.
   call check('part1', aoc_2015_12_01_part1(trim(text)//c_null_char, answer))
   print '(A,I0)', 'Part 1 🧮(🦀): ', answer

   call check('part2', aoc_2015_12_01_part2(trim(text)//c_null_char, answer))
   print '(A,I0)', 'Part 2 🧮(🦀): ', answer

contains

   !> A repo-root-relative path, anchored to this executable rather than to
   !> the caller's working directory — the same promise python/solve.py makes
   !> with `__file__` and Swift with `Bundle.main.bundlePath`.
   !>
   !> Fortran's answer is `get_command_argument(0)`, which is argv[0]: the
   !> path the program was *invoked* by. That is weaker than either of the
   !> above — it resolves against the working directory at exec time, and a
   !> shell that found us on PATH would hand back a bare name — but it is
   !> what the standard offers, and the recipe always invokes `./solve` from
   !> this directory. The bare-name case falls back to the same relative walk.
   function repo_path(relative) result(path)
      character(len=*), intent(in) :: relative
      character(len=:), allocatable :: path, exe
      integer :: length, slash

      call get_command_argument(0, length=length)
      allocate (character(len=length) :: exe)
      call get_command_argument(0, value=exe)

      ! days/<day>/fortran/solve -> three levels up is the repo root.
      slash = index(exe, '/', back=.true.)
      path = exe(1:slash)//'../../../'//relative
   end function repo_path

   !> The whole file, as one string of exactly its byte length.
   !>
   !> `access='stream'` is Fortran 2003's unformatted byte stream — the only
   !> read here that does not impose a record structure, pad to a fixed width,
   !> or eat the line terminators. It is also what makes the `trim` above a
   !> no-op: the string is allocated to the size INQUIRE reports, so it holds
   !> the file and nothing else.
   function read_file(path) result(text)
      character(len=*), intent(in) :: path
      character(len=:), allocatable :: text
      integer :: unit, status, bytes
      logical :: exists

      inquire (file=path, exist=exists, size=bytes)
      if (.not. exists) then
         write (error_unit, '(A)') 'no puzzle input at '//path//' (see .gitignore)'
         flush (error_unit)
         stop 1
      end if

      if (bytes <= 0) then
         text = ''
         return
      end if

      open (newunit=unit, file=path, access='stream', form='unformatted', &
            status='old', action='read', iostat=status)
      if (status /= 0) then
         write (error_unit, '(A,I0)') 'cannot open '//path//', iostat=', status
         flush (error_unit)
         stop 1
      end if

      allocate (character(len=bytes) :: text)
      read (unit) text
      close (unit)
   end function read_file

   !> A nonzero status is an error the C side already classified — report it
   !> and exit nonzero rather than printing a phantom answer. The status code
   !> is the return value here, which is worth noticing: it is the one channel
   !> a Fortran `function` has that does not go through an argument, and the
   !> only part of this call that is not by reference.
   subroutine check(name, status)
      character(len=*), intent(in) :: name
      integer(c_int), intent(in) :: status

      if (status /= 0) then
         write (error_unit, '(A,I0,A)') name//' failed with status ', status, &
            ' (-1 bad input, -2 no answer, -3 overflow, -4 internal error;'// &
            ' see days/README.md)'
         ! gfortran buffers error_unit and prints its own `STOP 1` trailer
         ! from the runtime, unbuffered — without this the trailer lands
         ! above the message explaining it.
         flush (error_unit)
         stop 1
      end if
   end subroutine check

end program solve
