You are reviewing one or more files in complete isolation. You have NOT seen the codebase, the project history, or any other files. Do not follow any links. Do not request additional context. Do not use any tools. Read only what is provided.

Your job: give honest, unbiased feedback on the document's hero section (typically the first 50 lines: title, tagline, install command, opening pitch) for a reader arriving cold -- an LLM agent, a contributor, or a user who just found this file.

The hero is the entry point. It must answer three questions in 30 seconds of reading:
1. What does this software do?
2. Why would I use it?
3. How do I install or try it?

Report the following sections:

## What I learned in 30 seconds

Plain language: what the software does, who would use it, and how you would try it. Cite the line numbers you read.

## Friction Points

Specific sentences, paragraphs, or terms in the hero that are confusing, ambiguous, or require outside knowledge not given in the document.

## Unanswered Questions

After reading the hero, what do you still not know?

## Install Path Clarity

Does the hero show a working install command? Does it lead with a package manager (`cargo install` / `cargo binstall` / `brew install` / `pip install` / `npm install` / `go install` -- any package-manager-first ordering)? OR does it lead with `git clone && build-from-source` (which is a worse first impression for a published tool)?

## Verdict

In 2 sentences: would an agent landing on this hero cold understand what the software does and how to try it?

Rate: CLEAR / NEEDS WORK / CONFUSING

Rationale: one paragraph explaining the rating.
