# Agent Policies

This document defines coding policies and guidelines for AI agents working on this project.

## RDSS Policy

**RDSS** = **Refactor, Despaghettify, Simplify, Split**

When working on code in this project, always apply the RDSS policy:

### R - Refactor
- Improve code structure and organization
- Extract reusable functions and modules
- Apply language-specific best practices and idioms
- Eliminate code duplication
- Improve naming for clarity and consistency

### D - Despaghettify
- Remove complex nested logic and callback chains
- Flatten deeply nested code structures
- Break up circular dependencies
- Eliminate global state where possible
- Replace complex control flow with clearer alternatives

### S - Simplify
- Reduce cognitive complexity
- Remove unnecessary abstractions
- Use clearer, more direct approaches
- Eliminate dead code and unused features
- Prefer explicit over implicit behavior

### S - Split
- Break up monolithic files into focused modules
- Separate concerns into distinct components
- Keep functions and methods small and focused
- Split large data structures into logical pieces
- Organize code by feature or domain, not by type

## Language-Specific Guidelines

### Rust (Primary Language)

- **Edition**: Use Rust 2024 edition
- **Idioms**: Leverage modern Rust patterns (async/await, const generics, trait bounds)
- **Error Handling**: Use `Result<T, E>` and `?` operator, never `unwrap()` in production code
- **Memory Safety**: Prefer owned types, use references judiciously, avoid `unsafe` unless absolutely necessary
- **Zero-cost Abstractions**: Use iterators, closures, and trait objects appropriately
- **Compile-Time Evaluation**: Always check if functions can be `const fn`
  - Maximize compile-time computation
  - Use `const fn` for functions that can be evaluated at compile time
  - Use `const` for constant values and static data
- **Documentation**: Write comprehensive doc comments with examples
- **Testing**: Include unit tests for all public APIs

### C++ (Level Editor Only - Phase 2)

- **Standard**: Use C++23 exclusively
- **Modern Idioms**: 
  - Use `<format>` and `<print>` for formatted output (no `printf`, no `iostream` operators)
  - Use `std::expected` for error handling
  - Use concepts for template constraints
  - Use ranges and views from `<ranges>`
  - Use `std::span` for array views

- **STL Containers**: *Always* use modern STL containers
  - `std::vector` for dynamic arrays
  - `std::array` for fixed-size arrays
  - `std::string` and `std::string_view` for strings
  - `std::unordered_map` / `std::map` for associative containers
  - `std::optional` for nullable values
  - `std::variant` for sum types

- **Smart Pointers**: *Always* use smart pointers, *NEVER* raw pointers
  - `std::unique_ptr` for exclusive ownership
  - `std::shared_ptr` for shared ownership (use sparingly)
  - `std::weak_ptr` to break cycles
  - Raw pointers are **ONLY** allowed for non-owning references (prefer references instead)

- **C Library**: *NEVER* use C library functions unless ABSOLUTELY NECESSARY
  - No `malloc/free` (use smart pointers and containers)
  - No `printf/scanf` (use `<print>` and `<format>`)
  - No `strcpy/strcat` (use `std::string`)
  - No `memcpy` (use `std::copy` or container methods)
  - Exception: Low-level system calls or third-party C APIs only

- **Compile-Time Evaluation**: Always check if functions can be `constexpr` or `consteval`
  - Maximize compile-time computation
  - Use `constexpr` variables and functions where possible
  - Use `consteval` for functions that must be compile-time only

### Reference Code Guidelines

When working with D2X-XL reference code (`/tmp/d2x-xl-src/`):

- **DO NOT** document corresponding C/C++ source files in our codebase
- **DO** extract algorithms, data structures, and format specifications
- **DO** document file formats, protocols, and data layouts
- **DO** reference original file paths in comments for traceability (e.g., `// Corresponds to: include/piggy.h`)
- **DO NOT** port code line-by-line; rewrite in idiomatic Rust/C++23
- **DO NOT** preserve original code style or structure

## Code Quality Standards

### General Principles

1. **Clarity over cleverness**: Readable code is more important than clever code
2. **Explicit over implicit**: Make intentions clear, avoid magic
3. **Local over global**: Prefer local variables and parameters over global state
4. **Composition over inheritance**: Favor composition and traits/interfaces
5. **Fail fast**: Validate input early, return errors immediately
6. **Test coverage**: Every public API should have tests
7. **Documentation**: All public APIs must have doc comments with examples

### Performance Considerations

- Profile before optimizing
- Optimize hot paths identified by profiling data
- Document performance characteristics in comments
- Use appropriate data structures for access patterns
- Consider cache locality and memory layout

### Code Review Checklist

Before committing code, verify:

- [ ] Follows RDSS policy (Refactor, Despaghettify, Simplify, Split)
- [ ] Uses language-appropriate idioms (Rust 2024 / C++23)
- [ ] No raw pointers or C library usage in C++ (unless justified)
- [ ] Functions checked for `const` / `constexpr` where applicable
- [ ] Comprehensive error handling with proper error types
- [ ] All public APIs have documentation
- [ ] Unit tests cover functionality
- [ ] No compiler warnings
- [ ] Code is formatted (rustfmt / clang-format)
- [ ] Commit message is clear and descriptive

## Architecture Guidelines

### Bevy ECS Integration (Rust)

- Implement game systems as Bevy plugins
- Use ECS components for game state
- Leverage Bevy's scheduling and parallelism
- Keep systems small and focused
- Use events for cross-system communication

### Asset Loading Pipeline

- Parse assets in `d2x-assets` crate (pure data transformation)
- Load assets into Bevy in `d2x-engine` crate (game integration)
- Cache and stream assets efficiently
- Support hot-reloading for development

### Networking Architecture

- Client-server model with authoritative server
- Client-side prediction with server reconciliation
- Deterministic simulation for replay/demo support
- Modern protocols (TCP/IP, WebRTC) - no legacy IPX/serial

## Commit Guidelines

### Commit Message Format

```
Short summary (50 chars or less)

- Bullet point list of changes
- Each point describes one logical change
- Use present tense ("Add feature" not "Added feature")
- Include relevant file paths and function names

Closes #issue-number (if applicable)
```

### Commit Scope

- Each commit should be atomic and self-contained
- Related changes should be in the same commit
- Unrelated changes should be separate commits
- All commits should pass tests and build successfully

## Documentation Standards

### Code Documentation

- **Rust**: Use `///` for doc comments, include examples in doc tests
- **C++**: Use Doxygen-style comments with `@brief`, `@param`, `@return`
- Document **why**, not just **what**
- Include complexity analysis for algorithms (Big-O notation)
- Document thread-safety and const-correctness

### Format Documentation

When documenting file formats (HOG, PIG, HAM, RDL, etc.):

- Include byte-level layout tables
- Show example data and parsing code
- Document all known versions and variations
- Reference original source code locations
- Include known file sizes and checksums
- Explain endianness and alignment
- Document error conditions and edge cases

## Testing Strategy

### Test Pyramid

1. **Unit Tests** (most): Test individual functions and modules
2. **Integration Tests** (moderate): Test crate interfaces and subsystems
3. **End-to-End Tests** (few): Test complete features with real data

### Test Organization

- Unit tests in same file as implementation (`#[cfg(test)]` module)
- Integration tests in `tests/` directory
- Test fixtures in `test_data/` (gitignored)
- Use `#[should_panic]` for expected failures
- Use property-based testing for parsers (consider `proptest` crate)

## Development Workflow

1. **Plan**: Understand requirements and design approach
2. **Document**: Write or update format/architecture docs first
3. **Implement**: Write code following RDSS policy
4. **Test**: Write tests and verify functionality
5. **Review**: Self-review against checklist
6. **Commit**: Create atomic commit with clear message
7. **Verify**: Ensure `cargo test` and `cargo build` pass

## Questions or Clarifications

When uncertain about how to proceed:

1. Check this document first
2. Review existing code for patterns
3. Consult project documentation (README, ARCHITECTURE, FEATURES)
4. Ask the user for clarification
5. Document the decision for future reference

---

**Remember**: The goal is clean, maintainable, idiomatic code that will last for years. Take time to do it right the first time.
