# RFC: tudu - Two-way TODO ↔ Issue Tracker Sync

## Executive Summary

This RFC outlines the implementation plan for `tudu`, a Rust-based tool that maintains bidirectional synchronization between TODO comments in code and issue trackers (Notion-first). The system is designed to be non-invasive, opt-in, and safe by default, producing patches rather than direct edits unless explicitly requested.

## Core Principles

1. **Safety First**: Read-only by default, explicit opt-in for writes
2. **Incremental Adoption**: Works with existing codebases without requiring migration
3. **Patch-based**: All changes proposed as diffs before application
4. **Provider Agnostic**: Extensible architecture supporting multiple issue trackers
5. **Performance**: Sub-60s cold scans on 100k LOC, sub-10s incremental

## Architecture Overview

### Module Structure

```
tudu/
├── src/
│   ├── main.rs              # CLI entry point
│   ├── cli/                 # Command-line interface
│   ├── scanner/             # File traversal and comment extraction
│   ├── parser/              # TODO comment parsing
│   ├── anchor/              # Context-based anchoring system
│   ├── providers/           # Issue tracker integrations
│   │   ├── mod.rs          # Provider trait definition
│   │   ├── notion.rs       # Notion API integration
│   │   ├── github.rs       # GitHub API integration
│   │   └── ...
│   ├── reconciler/          # Sync logic and conflict resolution
│   ├── writer/              # Patch generation and file editing
│   ├── state/               # Persistent state management
│   └── config/              # Configuration handling
├── tests/
└── Cargo.toml
```

## Implementation Milestones

### Milestone 0: Project Setup (Week 1)

**Goal**: Establish project foundation and development environment

#### Tasks:

- [ ] Initialize Rust project structure
- [ ] Set up Cargo.toml with core dependencies
- [ ] Create basic CI/CD pipeline (GitHub Actions)
- [ ] Set up testing framework (proptest, insta, assert_cmd)
- [ ] Create initial README and development documentation

### Milestone 1: Core Infrastructure (Weeks 1-2)

**Goal**: Build foundational components for file processing

#### Tasks:

- [ ] Implement configuration system (.tudu.yaml parser)
- [ ] Create CLI structure with clap
- [ ] Build logging/tracing infrastructure
- [ ] Implement error handling framework (thiserror)
- [ ] Set up state management (.tudu/state.json)

### Milestone 2: Scanner Module (Week 2)

**Goal**: Efficient file traversal respecting ignore patterns

#### Tasks:

- [ ] Integrate `ignore` crate for .gitignore support
- [ ] Implement parallel file walker
- [ ] Create language-agnostic comment extractors:
  - [ ] Line comments (// and #)
  - [ ] Block comments (/\* \*/ and """)
  - [ ] HTML comments (<!-- -->)
- [ ] Add configurable ignore patterns
- [ ] Implement incremental scanning logic

### Milestone 3: Parser Module (Week 3)

**Goal**: Robust TODO comment parsing with helpful errors

#### Tasks:

- [ ] Define formal grammar for TODO syntax
- [ ] Implement TODO(...) parser with attribute support
- [ ] Support legacy TODO <ID>: format
- [ ] Handle untracked TODOs (no ID)
- [ ] Create comprehensive error messages
- [ ] Add parser validation tests with fuzzing

### Milestone 4: Anchor System (Week 3)

**Goal**: Stable comment tracking across file changes

#### Tasks:

- [ ] Implement context hashing (xxhash)
- [ ] Create rolling window capture (±N lines)
- [ ] Build relocation algorithm for moved TODOs
- [ ] Implement fallback strategies for lost anchors
- [ ] Test anchoring stability across refactors

### Milestone 5: Provider Trait & Notion Integration (Week 4)

**Goal**: Core provider abstraction and Notion implementation

#### Tasks:

- [ ] Define IssueProvider trait
- [ ] Implement Notion provider:
  - [ ] Authentication and API client
  - [ ] Multi-database support
  - [ ] Property mapping (status, labels, assignee, etc.)
  - [ ] Create/Read/Update operations
  - [ ] Status-based closing
- [ ] Add provider caching (ETags/timestamps)
- [ ] Implement rate limiting

### Milestone 6: Reconciliation Engine (Week 5)

**Goal**: Core sync logic and conflict detection

#### Tasks:

- [ ] Build reconciliation state machine
- [ ] Implement drift detection (comment ↔ issue)
- [ ] Create conflict resolution strategies
- [ ] Handle TODO lifecycle (create/update/close)
- [ ] Implement policy enforcement (one-way vs bidirectional)

### Milestone 7: Writer Module (Week 6)

**Goal**: Safe file modification with patch generation

#### Tasks:

- [ ] Integrate diff library (similar/diffy)
- [ ] Create unified diff generator
- [ ] Implement in-place editor (--apply mode)
- [ ] Preserve formatting and indentation
- [ ] Build patch file management (.tudu/patches/)

### Milestone 8: Core Commands (Week 6)

**Goal**: Complete CLI with all primary commands

#### Tasks:

- [ ] Implement `tudu scan` (list and validate)
- [ ] Implement `tudu file` (create issues for untracked)
- [ ] Implement `tudu sync` (with --dry-run and --apply)
- [ ] Implement `tudu status` (drift reporting)
- [ ] Add JSON output mode for CI

### Milestone 9: Testing & Validation (Week 7)

**Goal**: Comprehensive test coverage and validation

#### Tasks:

- [ ] Create integration test suite
- [ ] Build mock providers for testing
- [ ] Add snapshot tests for patches
- [ ] Test anchoring stability with pathological cases
- [ ] Performance benchmarks (100k LOC target)
- [ ] End-to-end workflow tests

### Milestone 10: Documentation & Polish (Week 8)

**Goal**: Production-ready release

#### Tasks:

- [ ] Write comprehensive user documentation
- [ ] Create CI/CD examples
- [ ] Add configuration examples
- [ ] Build demo repository
- [ ] Create migration guide for existing repos
- [ ] Performance optimization pass

## Technical Decisions

### Dependencies

- **Core**: tokio, reqwest, serde, clap, anyhow, thiserror
- **File handling**: ignore, regex
- **Diffing**: similar or diffy
- **Hashing**: xxhash-rust
- **Testing**: proptest, insta, wiremock

### Key Design Choices

1. **Comment Detection**: Language-agnostic regex patterns rather than AST parsing (initially)
2. **Anchoring**: Context-based hashing for resilience to code changes
3. **State Storage**: Local JSON file for simplicity and transparency
4. **Patch Generation**: Unified diff format for compatibility
5. **Provider Architecture**: Trait-based for extensibility

## Testing Strategy

### Unit Tests

- Parser edge cases and fuzzing
- Anchor stability tests
- Provider API mocking

### Integration Tests

- End-to-end workflows
- Multi-provider scenarios
- Conflict resolution cases

### Performance Tests

- Large repository scanning
- Incremental update efficiency
- Provider API batching

## Risk Mitigation

### Technical Risks

1. **Anchor instability**: Mitigated by fallback strategies and manual review
2. **API rate limits**: Batching and caching with exponential backoff
3. **Destructive edits**: Patch-first approach, explicit --apply flag

### Adoption Risks

1. **Legacy TODO friction**: Support common formats, gradual migration path
2. **Performance concerns**: Incremental scanning, parallel processing
3. **Trust issues**: Read-only default, comprehensive dry-run mode

## Success Metrics

### MVP (v0.1)

- [ ] Cold scan < 60s on 100k LOC
- [ ] Incremental scan < 10s
- [ ] 95%+ anchor stability
- [ ] Zero destructive edits without --apply
- [ ] Full Notion provider support
- [ ] Basic GitHub/GitLab/Jira validation

### Future Iterations

- Full write support for all providers
- IDE plugins
- PR bot mode
- AST-aware anchoring
- Package distribution (Homebrew, Docker)

## Implementation Order

1. **Foundation** (Milestones 0-1): Setup and core infrastructure
2. **Input Pipeline** (Milestones 2-4): Scanner → Parser → Anchor
3. **Provider Layer** (Milestones 5-6): Notion-first, then others
4. **Processing** (Milestones 7-8): Reconciliation and writing
5. **Interface** (Milestone 9): Complete CLI
6. **Quality** (Milestones 10-11): Testing and documentation

## Timeline

- **Week 1**: Project setup and core infrastructure
- **Week 2**: Scanner implementation
- **Week 3**: Parser and anchor system
- **Week 4**: Notion provider
- **Week 5**: Additional providers and reconciliation
- **Week 6**: Writer and CLI commands
- **Week 7**: Testing and validation
- **Week 8**: Documentation and release preparation

Total estimated time: 8 weeks for MVP

## Open Questions

1. Should we support custom comment markers beyond standard syntax?
2. How should we handle TODO comments in generated code?
3. What's the optimal context window size for anchoring?
4. Should provider credentials support OS keychain from day one?
5. How to handle TODOs in binary files or non-text formats?

## Conclusion

This RFC provides a clear path from zero to a functional `tudu` tool that safely synchronizes TODO comments with issue trackers. The implementation prioritizes safety, performance, and gradual adoption, making it suitable for both new projects and large existing codebases.
