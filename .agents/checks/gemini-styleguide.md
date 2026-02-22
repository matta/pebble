---
name: Gemini Styleguide Reference Check
description: Keep the Gemini Styleguide up to date with `.agents/checks/*.md`
---
Every check in `.agents/checks/*.md` must be reflected in
`.gemini/styleguide.md` with a transinclude as such:

# Style checks
- @.agent/checks/check1.md
- @.agent/checks/check2.md
