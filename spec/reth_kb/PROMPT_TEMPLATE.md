# Gemini Prompt Template

Use this template to generate consistent, well-cited answers from Gemini.

## Template

```text
You are analyzing the Rust codebase for Reth. I attached the following Repomix XML packs:
- {CONTEXT_PACKS}

Question:
{QUESTION}

Requirements:
1. Cite file paths and function/type names for every major claim.
2. Prefer short quoted snippets or signatures when possible.
3. Separate facts from hypotheses; call out uncertainty explicitly.
4. Use plain language explanations in addition to citations.

Please structure the answer as:
1. Summary (plain English, 5-8 bullets)
2. Architecture map (major modules/crates and how they connect)
3. Key data flows (bulleted steps)
4. Key abstractions/types (with where they live)
5. File/function references (bullet list, each with a one-line purpose)
6. Suggested follow-up questions
```

## Notes
- Keep the response grounded in the provided packs only.
- If the packs are insufficient, explicitly say what additional context is needed.
