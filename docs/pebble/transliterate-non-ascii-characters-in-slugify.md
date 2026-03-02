---
id: pebl-izOTJ6
title: Transliterate non-ASCII characters in slugify
status: done
created_at: 2026-02-22T22:32:50.516148+00:00
modified_at: 2026-03-01T22:42:43.925390+00:00
resolved_at: 2026-03-01T22:42:43.925377+00:00
tags:
  - feature
---

Currently slugify drops non-ASCII characters entirely (e.g. café → caf). Consider using a transliteration library like deunicode to convert them to ASCII equivalents instead (e.g. café → cafe). This would improve usability for non-English users.
