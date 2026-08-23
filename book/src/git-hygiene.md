# Git

> **Workshop note: this step is already handled for you — we bypass it.**
> The repo's ignore rules cover *every* language track (Rust, C, Python,
> Swift, Kotlin/JNA, Dart) plus OS noise, even though Exercise 3 only needs
> one track. Spare ignore rules cost nothing, and trying a second language
> is an extension exercise — so nothing you build during the workshop will
> ever show up as git noise. This chapter stays as the *why*.

Git needs no introduction—it's the ubiquitous version control system you already know. This chapter isn't about teaching Git itself, but about why the very first commit matters more than you might think.

## The Initial Commit

The template this repository was cloned from started with a single commit containing only a `.gitignore`:

```gitignore
# Ignore build outputs from performing a nix-build or `nix build` command
result
result-*

# Ignore automatically generated direnv output
.direnv
```

That's it. No code, no configuration—just ignore rules.

## Why This Matters

This template is a Nix-based development environment, but the principle applies universally: **get your `.gitignore` right from the very beginning**.

### Step Zero, Not Step One

Most tutorials treat `.gitignore` as an afterthought—something you add when you notice unwanted files sneaking into your commits. This is a mistake, and the bill comes due the first time you need to rewrite history.

### The Cost Shows Up Later

Every interesting Git operation replays old commits: `rebase` reapplies your work onto a new base, `cherry-pick` lifts one commit somewhere else, `bisect` checks out arbitrary points in the past to find where a bug appeared.

If your early commits contain files that *should* have been ignored—build artifacts, editor configs, generated files—each of those operations drags them along. You get conflicts in files nobody edited on purpose, because two branches both regenerated the same `target/` directory differently. You get diffs where three real lines hide inside three hundred generated ones. And `bisect` starts checking out revisions whose committed build output doesn't match the source, so the build you're testing isn't the build you think it is.

Removing the files later doesn't undo this. A `.gitignore` added in commit fifty doesn't retroactively clean commits one through forty-nine—the objects are still in history, still replayed by every rebase, still cloned by everyone. Getting them out means `git filter-repo` and a force-push that rewrites every hash, which is a bad afternoon and a worse conversation with your collaborators.

By establishing ignore patterns in the literal first commit, you ensure that:

1. **No garbage ever enters the repository** - Build outputs, cache directories, and environment-specific files are excluded from day one
2. **History stays clean** - Rebasing, cherry-picking, and bisecting work smoothly
3. **The pattern is established** - Contributors see immediately that this project takes repository hygiene seriously

## For Any Project

> Use https://github.com/github/gitignore as a starting point.

While this template uses Nix, the same principle applies everywhere:

- **Node.js**: Ignore `node_modules/`, `.next/`, `dist/`
- **Python**: Ignore `__pycache__/`, `*.pyc`, `.venv/`, `*.egg-info/`
- **Rust**: Ignore `target/`

The specific patterns vary; the principle doesn't. Start clean, stay clean.

---

- https://git-scm.com/docs/gitignore
- https://www.kernel.org/pub/software/scm/git/docs/gitignore.html