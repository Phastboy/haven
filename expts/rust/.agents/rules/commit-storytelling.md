# Commit Philosophy: Narrative Storytelling

Every feature branch and pull request must tell a coherent, elegant story through its commit history. Commits are chapters in an unfolding narrative, progressing strictly from the quietest foundational ground to the most impactful user-facing changes.

## The Narrative Arc (Least Impactful to Most Impactful)

When planning and sequencing commits, arrange them strictly in ascending order of impact:

1. **Foundations & Environment (`docs`, `chore`)**:
   - Environment variable declarations (e.g., `.env.example`).
   - Architectural decision records, documentation, and baseline benchmarks.
2. **Domain Contracts & Types (`feat(domain)`)**:
   - Pure domain models, strongly-typed newtypes, error enums, and port traits.
   - Zero side-effects; defines the cast and laws of the system.
3. **Infrastructure & Adapters (`feat(db)`, `feat(infra)`)**:
   - Database migrations, SQL queries, repository implementations, external client wrappers.
4. **Application & Routing (`feat(web)`, `feat(auth)`)**:
   - Application state wiring, router extensions, and HTTP request handlers.
5. **Presentation & UI (`feat(ui)`)**:
   - HTML templates, views, buttons, and user interaction entry points.
6. **Verification & Hardening (`test`)**:
   - Automated integration tests, unit tests, and validation asserting the new behavior.
7. **Milestone Closure & Epilogue (`docs(roadmap)`)**:
   - Roadmap updates, removing temporary spikes, and release tags.

## Craftsmanship Guidelines

- **Linear History**: Rebase feature branches cleanly onto `main`.
- **Atomic & Buildable**: Every intermediate commit must compile cleanly and pass tests on its own.
- **Conventional Commits**: Use descriptive types (`feat`, `fix`, `docs`, `test`, `perf`, `refactor`) with a concise summary.
