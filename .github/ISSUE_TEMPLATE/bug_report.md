---
name: Bug report
about: Create a report to help us improve
title: ''
labels: bug
assignees: ''

---

**Describe the bug**
A clear and concise description of what the bug is.

**Enseada version**
Binary version [e.g. v0.1.0] or Docker tag [e.g. enseada/enseada:edge]

**To Reproduce**
Steps to reproduce the behavior:
1. Get token with scopes '...'
2. Call endpoint '...'
3. See error

**Expected behavior**
A clear and concise description of what you expected to happen.

**Logs**
Any relevant log entry is more than welcome. Please attach them in a code block like so:

```bash
{"level":"info","ts":1577983670.328825,"logger":"maven2","caller":"couch/databases.go:25","msg":"database maven2 already exists"}
{"level":"info","ts":1577983670.3288562,"logger":"maven2","caller":"couch/indexes.go:19","msg":"initializing index kind_index on db maven2"}
{"level":"info","ts":1577983670.3298516,"logger":"maven2","caller":"couch/indexes.go:19","msg":"initializing index file_index on db maven2"}
{"level":"info","ts":1577983670.3311126,"caller":"enseada-server/main.go:66","msg":"booted Enseada in 1293ms"}
``

**Host environment (please complete the following information):**
 - Type: [e.g. laptop, VM, container orchestrator]
 - OS: [e.g. macOS, CentOS 8]

**Additional context**
Add any other context about the problem here.
