# 2026 agent-engineering basis

This bootstrap deliberately applies recent coding-agent findings instead of copying older "huge prompt file" patterns.

## Applied conclusions

### 1. Root instructions are a map, not an encyclopedia
OpenAI's 2026 harness-engineering write-up reports that a large monolithic `AGENTS.md` crowded out useful task/code context and rotted quickly. Their agent-first repository moved durable knowledge into structured repository docs and used a short `AGENTS.md` as a table of contents.

**GE4G consequence:** root `AGENTS.md` routes to canonical docs; detailed engine knowledge lives elsewhere.

### 2. Progressive disclosure beats mandatory pre-reading
OpenAI's September 2026 guidance for GPT-6 Astra recommends shorter skill descriptions, progressive disclosure, and task-conditional document reading. It explicitly warns against forcing the model to read a stack of docs before every edit.

**GE4G consequence:** "read by task, not by ritual."

### 3. Context files can make agents worse when they add unnecessary requirements
Gloaguen et al., *Evaluating AGENTS.md: Are Repository-Level Context Files Helpful for Coding Agents?* (arXiv:2602.11988, 2026) found that context files often reduced task success and increased inference cost; their conclusion favors minimal human-written requirements.

**GE4G consequence:** standing instructions contain only durable, non-obvious constraints.

### 4. Durable work state belongs in repository artifacts
OpenAI's harness-engineering and Symphony work emphasize repository-local plans, task state, and machine-checkable artifacts instead of relying on session memory.

**GE4G consequence:** ExecPlans, test evidence, failure notes, schemas, and state registries live in the repository.

### 5. Work should be organized around deliverables
Symphony treats tasks/deliverables as the durable unit rather than individual agent sessions.

**GE4G consequence:** plans define observable outcomes and acceptance evidence. Agent sessions are disposable.

## Sources

- OpenAI Developers — “Rethinking skills and prompts for GPT-6 Astra” (2026-09-11)
- OpenAI — “Harness engineering: leveraging Codex in an agent-first world” (2026)
- Thibaud Gloaguen et al. — “Evaluating AGENTS.md: Are Repository-Level Context Files Helpful for Coding Agents?” arXiv:2602.11988 (2026-02-12)
- OpenAI — “An open-source spec for Codex orchestration: Symphony” (2026-04-27)
