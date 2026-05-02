← [back to index](./index.md) · phase 30 research

## Competitor Narrative Scan

Patterns extracted per the source-of-truth note's explicit call-out (subagent #1). The copy subagent picks 3 of these; this is the menu.

### Pattern A — Linear's carousel-of-product-screenshots hero (reject for us)
**What:** Linear's home (`linear.app`) leads with "The product development system for teams and agents" + "Purpose-built for planning and building products. Designed for the AI era" + three carousel-style product interface screenshots. `[CITED: linear.app homepage analysis]`.
**Asymmetric before/after:** Appears below fold in the "Diffs" feature section — an old `HomeScreen.tsx` vs new progressive-loading version.
**"No new vocabulary" equivalent:** None — Linear leans into proprietary vocabulary ("Issues", "Cycles", "Initiatives") and markets it.
**Diátaxis nav:** Product → Customers → Pricing → Method (marketing-structured, not docs-structured). Linear's docs live at a separate subdomain.
**Why we reject:** Marketing-heavy, product-screenshot-led. Our hero is a code-block before/after, not a product dashboard.
**What we steal:** Nothing directly, but the "terse headline + declarative subline" pattern is solid. Linear's hero is one line.

### Pattern B — Fly.io's "Build fast. Run any code fearlessly." (STEAL the cadence)
**What:** Hero is a two-beat tagline — "Build fast. Run any code fearlessly." — plus a three-pillar breakdown (Machines / Sprites / Storage). No before/after code. `[CITED: fly.io homepage analysis]`.
**Asymmetric pattern:** Alternating text-block and image-block sections down the page. No code in hero.
**"No new vocabulary" equivalent:** Uses "Machines" (not "VMs") and "sandboxes" (not "containers") — accessible language for familiar concepts.
**Diátaxis nav:** Docs separate at `fly.io/docs/` with a clean Get Started / Guides / Reference split.
**Why it's relevant:** Fly.io's voice is "developer-friendly without being overly promotional" — closest to our "precise, dry, earned" voice requirement.
**What we steal:** The two-beat tagline cadence. Candidate for our hero: *"Close a ticket with `sed` and `git push`. Keep your REST API for the 20% that needs it."* (Under 20 words, echoes V1 vignette, includes complement.)

### Pattern C — Warp's agentic framing (align on positioning)
**What:** "Warp is the agentic development environment" + dual offering (Terminal + Oz). Lead with "700K+ developers" trust signal. `[CITED: warp.dev homepage analysis]`.
**Asymmetric pattern:** No before/after. Leads with brand authority.
**"No new vocabulary" equivalent:** Leans into "agentic" as a new term — bet that the AI developer audience has adopted the word.
**Diátaxis nav:** Standard docs nav.
**Why it's relevant:** Warp's audience is the same as ours — AI-agent-adjacent developers. They've validated that this audience accepts "agentic" as a hero-level term.
**What we steal:** Nothing for the hero (we don't want trust signals up top — too early). BUT — "agentic" or "autonomous agents" is safe above the fold, unlike "MCP" which is narrower.

### Pattern D — Val Town's "Zapier for know-code engineers" (STEAL the positioning-line technique)
**What:** Hero is "Instantly deploy" + the positioning line "Zapier for know-code engineers." Live editable code block beneath. `[CITED: val.town homepage analysis]`.
**Asymmetric pattern:** No before/after, but live code in hero.
**"No new vocabulary" equivalent:** "know-code engineers" is coinage — a bet that audience self-identifies with a new label.
**Diátaxis nav:** Less rigorous; docs are intermixed with blog.
**Why it's relevant:** The "X for Y-who-do-Z" positioning line is POWERFUL. One sentence tells you who it's for and what it substitutes for.
**What we steal:** The positioning-line technique. Candidate: *"Common tracker ops for autonomous agents. `cat` instead of curl, `git push` instead of PATCH."*

### Pattern E — Raycast's "Your shortcut to everything." (terse-line discipline)
**What:** Hero is "Your shortcut to everything." + subhead + Download buttons. Static keyboard image below. `[CITED: raycast.com homepage analysis]`.
**Asymmetric pattern:** No before/after. No code. All keyboard-icon visuals.
**"No new vocabulary" equivalent:** Extreme terseness. "Shortcut to everything" is four words.
**Diátaxis nav:** Standard docs.
**Why it's relevant:** Demonstrates the upper bound of hero terseness. A very confident product can get away with a four-word hero.
**What we steal:** The discipline. If we can say it in four words we should. But: our audience needs proof, not polish — the four-word hero only works for Raycast because their product is a launcher, instantly graspable. Ours is not. We keep terseness discipline but add a code-proof block.

### Pattern F — Turso concepts page (steal for mental-model.md length/shape)
**What:** Turso's `/concepts` landing lands as a hierarchical card layout, ~350-400 words, no code examples, no diagrams. `[CITED: docs.turso.tech/concepts analysis]`.
**Why it's relevant:** Sets the length precedent for our `mental-model.md` — keep it short, card-based, readable in one sitting.
**What we steal:** The 300-400 word ceiling. Mental model in 60 seconds is a READING target, not a scrolling target.

### Pattern G — Stripe quickstart's first "aha" moment (steal for tutorial.md structure)
**What:** Stripe's dev quickstart (`docs.stripe.com/development/quickstart`) has 6 structured sections. The "aha" lands in section 4: **after running `stripe products create ...`, the CLI returns a JSON response with a real `prod_LTenIrmp8Q67sa` ID**. The reader sees the API respond before writing any application code. `[CITED: docs.stripe.com/development/quickstart analysis]`.
**Why it's relevant:** This is the pattern for our 5-minute tutorial. The reader edits `issues/PROJ-42.md`, runs `git push`, and then `curl`s the simulator to see the version bumped from 1 → 2. The "aha" is the server-side confirmation that the file-edit flowed all the way through.
**What we steal:** The "confirm setup" step. After the git push, print the `curl | jq` that proves the state change landed. This mirrors existing demo.md step 7 and makes the tutorial feel like a demonstrable closed loop.

### Pattern H — Cloudflare Workers "Get Started" (steal for tutorial length)
**What:** 4 numbered sections, 70% prose / 30% code, reader runs first command (`npm create cloudflare@latest`) in the first major section. `[CITED: developers.cloudflare.com/workers/get-started/guide analysis]`.
**Why it's relevant:** Length precedent for a tutorial. "5-minute" is a promise about reading + doing time, not a hard word count, but 4 steps / ~500 words feels right.
**What we steal:** The 4-step structure. Our tutorial: (1) start the simulator, (2) mount it, (3) edit an issue, (4) git push and verify.

### Menu-style recap — which 3 to steal

The copy subagent should pick 3 of these. Research recommendation: **B (Fly.io two-beat cadence) + D (Val Town positioning-line) + G (Stripe "confirm setup" moment in tutorial)**. B + D shape the hero; G shapes the tutorial. Each is citable. None requires inventing a new pattern.
