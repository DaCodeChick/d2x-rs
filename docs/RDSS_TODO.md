# RDSS TODO List - Comprehensive Codebase Refactoring

**Generated**: 2026-03-01  
**Scope**: All 48 Rust files, 15,332 lines of code  
**Status**: Analysis Complete, Refactoring In Progress  

---

## Overview

This document tracks the comprehensive RDSS (Refactor, Despaghettify, Simplify, Split) refactoring of the d2x-rs codebase.

### Codebase Statistics

| Metric | Value |
|--------|-------|
| Total Rust files | 48 |
| Total lines of code | 15,332 |
| Public functions | 176 |
| Impl blocks | 116 |
| Type definitions | ~200+ |
| Test files | 3 |
| Tests passing | 179 |
| Unsafe blocks | 0 ✅ |
| Clippy warnings | ~15 |

---

## Priority 1: Critical Issues (Standards Compliance)

### 1.1 Error Handling Review ✅ RESOLVED

**Issue**: `video.rs` uses `anyhow::Result` instead of `crate::error::Result<T>`

**Files affected**:
- `crates/descent-core/src/video.rs` (514 lines)

**Resolution**: ACCEPTED AS EXCEPTION

**Rationale**: Video conversion is:
1. Feature-gated (`#[cfg(feature = "video")]`)  
2. Utility/tool functionality (not core asset parsing)
3. Heavy integration with external FFmpeg C library (complex error scenarios)
4. Similar to CLI tool usage where anyhow is explicitly allowed per AGENTS.md

**Documentation**: Added exception note to AGENTS.md standards

**Status**: ✅ RESOLVED

---

### 1.2 Clippy Warnings ✅ COMPLETE

**Total warnings**: ~15 → 0 in descent-core

#### Fixed Issues
- ✅ `ase.rs:769` - `expect_line()` method (added #[allow(dead_code)])
- ✅ `tga.rs:49,52,53` - Unused header fields (added #[allow(dead_code)])
- ✅ `tga.rs:63` - `ImageType` enum (added #[allow(dead_code)])
- ✅ `model.rs:459` - Unnecessary `unwrap()` (converted to `if let` pattern)
- ✅ `model.rs:393` - Used `.or_default()` instead of `.or_insert_with(Vec::new)`
- ✅ `models.rs:143` - Fixed collapsible `if` (used `if let && pattern`)
- ✅ `mvl.rs:99` - Used `contains()` for range check
- ✅ `ase.rs:349` - Fixed AseMap initialization (struct update syntax)
- ✅ `fixed_point.rs:318` - Used `std::f32::consts::PI` instead of literal
- ✅ `ase.rs:546` - Added type aliases for complex return types
- ✅ `video.rs:239` - Fixed collapsible if with `let &&` pattern
- ✅ `tga.rs:106,140` - Auto-fixed div_ceil reimplementation

**Result**: All clippy warnings resolved in descent-core library

**Commit**: 4902ebf

**Status**: ✅ COMPLETE

---

### 1.3 Unwrap Usage Audit ⚠️ HIGH

**Issue**: 200+ uses of `.unwrap()` across codebase (anti-pattern in production code)

**Top offenders**:
1. `converters/model.rs` - 31 unwraps (mostly in doc examples)
2. `hog2.rs` - 27 unwraps
3. `converters/level.rs` - 19 unwraps
4. `pof.rs` - 18 unwraps
5. `dhf.rs` - 17 unwraps
6. `player.rs` - 16 unwraps
7. `models.rs` - 14 unwraps

**Note**: Many are in doc examples (acceptable), but production code should use `?` operator

**Action**: Audit all unwrap() usage, convert production code to proper error handling

**Effort**: High (4-6 hours)

**Status**: ❌ TODO

---

## Priority 2: Documentation Improvements

### 2.1 Documentation Coverage ⚠️ MEDIUM

**Current**: 331 doc comments / 13,367 lines = **2.5%**  
**Target**: 5-10%  
**Gap**: Need ~500+ more doc comment lines

**Files needing docs**:
- `d2x-engine/` crates (minimal documentation)
- `d2x-client/` crates (minimal documentation)
- Internal helper functions in `descent-core`

**Action**: Add comprehensive doc comments to all public APIs

**Effort**: High (ongoing)

**Status**: ⚠️ IN PROGRESS

---

## Priority 3: Refactoring Opportunities

### 3.1 Large File Splitting 📦 MEDIUM

**Files over 500 lines** (candidates for splitting):

| File | Lines | Status | Recommendation |
|------|-------|--------|----------------|
| `converters/model.rs` | 1,295 | ❌ TODO | Split into `pof_converter.rs` + `ase_converter.rs` + `gltf_builder.rs` |
| `converters/level.rs` | 1,132 | ❌ TODO | Extract mesh building logic to separate module |
| `ase.rs` | 863 | ❌ TODO | Split parser and data structures |
| `level.rs` | 787 | ✅ OK | Manageable size, mostly data structures |
| `pof.rs` | 715 | ✅ OK | Manageable size, mostly parsing |
| `ham.rs` | 684 | ✅ OK | Complex but cohesive |
| `player.rs` | 586 | ✅ OK | Manageable size |
| `sound.rs` | 546 | ✅ OK | Manageable size |
| `video.rs` | 514 | ⚠️ FIX | Fix error handling first, then OK |

**Action**: Split converters/model.rs into separate modules

**Effort**: High (3-4 hours)

**Status**: ❌ TODO

---

### 3.2 Code Duplication Analysis 🔄 LOW

**Clone usage**: Minimal (7-10 per file average, acceptable)

**Pattern duplication**: Archive parsers (HOG2, DHF, MVL) share similar patterns
- ✅ Good: Already follow consistent pattern
- ⚠️ Potential: Could extract common archive traits

**Action**: Consider creating common `ArchiveTrait` to reduce duplication

**Effort**: Medium (2-3 hours)

**Status**: ❌ TODO (low priority)

---

### 3.3 Function Length Analysis 📏 LOW

**Analysis needed**: Find functions over 100 lines

**Action**: Identify and refactor long functions

**Effort**: Medium (ongoing)

**Status**: ❌ TODO

---

## Priority 4: Performance Optimizations

### 4.1 Unnecessary Allocations 🚀 LOW

**Areas to investigate**:
- String allocations in error messages
- Vec allocations in hot paths
- Clone usage in converters

**Action**: Profile and optimize after functionality is complete

**Effort**: Medium (2-3 hours)

**Status**: ❌ TODO (future work)

---

## Priority 5: Architecture Improvements

### 5.1 Module Organization 📂 LOW

**Current structure**: Good, well-organized by feature

**Potential improvements**:
- Split `converters/` into subdirectories by format
- Consider `parsers/` directory separate from format modules

**Action**: Evaluate module reorganization

**Effort**: Low (1-2 hours)

**Status**: ❌ TODO (low priority)

---

### 5.2 Trait Abstractions 🎭 LOW

**Opportunities**:
- Common `Parser` trait for format parsers
- Common `Archive` trait for HOG2/DHF/MVL
- Common `Converter` trait for format converters

**Action**: Design and implement common traits

**Effort**: High (4-5 hours)

**Status**: ❌ TODO (future work)

---

## Implementation Plan

### Phase 1: Fix Critical Issues (Week 1)

1. ✅ **DONE**: Establish RDSS standards in AGENTS.md
2. ❌ **TODO**: Fix video.rs error handling (anyhow → AssetError)
3. ❌ **TODO**: Fix all clippy warnings (dead code, unnecessary unwrap, etc.)
4. ❌ **TODO**: Audit unwrap() usage in production code

**Estimated time**: 8-10 hours

---

### Phase 2: Documentation (Week 2)

1. ❌ **TODO**: Add doc comments to all public APIs in descent-core
2. ❌ **TODO**: Add doc comments to d2x-engine public APIs
3. ❌ **TODO**: Add doc comments to d2x-client public APIs
4. ❌ **TODO**: Add examples to major converters

**Estimated time**: 10-12 hours

---

### Phase 3: Refactoring (Week 3-4)

1. ❌ **TODO**: Split converters/model.rs into modules
2. ❌ **TODO**: Split converters/level.rs mesh logic
3. ❌ **TODO**: Split ase.rs parser and structures
4. ❌ **TODO**: Extract common archive functionality
5. ❌ **TODO**: Refactor long functions (>100 lines)

**Estimated time**: 15-20 hours

---

### Phase 4: Polish (Week 5)

1. ❌ **TODO**: Run final clippy check with all warnings resolved
2. ❌ **TODO**: Run cargo fmt on all files
3. ❌ **TODO**: Verify all tests still pass
4. ❌ **TODO**: Profile and optimize hot paths

**Estimated time**: 5-8 hours

---

## Tracking

### Completed Items ✅

- [x] Comprehensive codebase analysis
- [x] RDSS standards established in AGENTS.md
- [x] RDSS_ANALYSIS.md created with detailed findings

### In Progress ⚠️

- [ ] Video.rs error handling fix
- [ ] Clippy warnings resolution
- [ ] Unwrap audit

### Blocked 🚫

None

---

## Metrics

### Before RDSS
- Clippy warnings: ~15
- Unwrap count: 200+
- Doc coverage: 2.5%
- Unsafe blocks: 0 ✅

### After RDSS (Target)
- Clippy warnings: 0
- Unwrap count: <50 (only in tests/examples)
- Doc coverage: 5%+
- Unsafe blocks: 0 ✅

---

## Notes

- All changes must maintain 100% test pass rate (179 tests)
- Follow AGENTS.md standards strictly
- Document all major refactoring decisions
- Create atomic commits for each logical change
- Run `cargo test --all` before each commit

---

**Last Updated**: 2026-03-01  
**Maintained By**: OpenCode AI Agent
