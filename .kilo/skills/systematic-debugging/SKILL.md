---
name: systematic-debugging
version: "1.0.0"
description: A systematic 4-phase debugging framework for reproducing, analyzing, fixing, and verifying bugs
---

# Systematic Debugging

A structured approach to debugging that ensures consistent, reliable bug fixes with verification.

## When to Use

Use this skill when:
- Encountering unexpected behavior or errors
- Investigating failing tests
- Debugging production issues
- Analyzing performance problems

## The 4-Phase Framework

### Phase 1: Reproduce the Issue

**Goal**: Create a minimal, reliable reproduction

**Steps**:
1. Identify the exact conditions that trigger the issue
2. Create a minimal test case or script
3. Verify you can reproduce it consistently
4. Document the reproduction steps

**Evidence Format**:
```
REPRODUCTION:
Environment: [OS, runtime version, dependencies]
Steps:
1. [step 1]
2. [step 2]
3. [step 3]
Expected: [what should happen]
Actual: [what actually happens]
```

**Best Practices**:
- Isolate the problem - remove unrelated code
- Use logging/debugging to narrow scope
- Check if issue is deterministic or intermittent
- Note any error messages, stack traces, or logs

### Phase 2: Analyze Root Cause

**Goal**: Identify why the issue occurs

**Steps**:
1. Trace execution flow from reproduction
2. Examine relevant code with `file:line` references
3. Identify the specific line/condition causing the issue
4. Determine the underlying cause

**Analysis Template**:
```
ANALYSIS:
file:line - [description of problematic code]
ACTUAL: [what the code does]
EXPECTED: [what it should do]
ROOT CAUSE: [wrong input | wrong order | missing dep | config | logic | external]

EVIDENCE:
- [evidence point 1 with file:line]
- [evidence point 2 with file:line]
- [evidence point 3 with file:line]
```

**Investigation Techniques**:
- Binary search (comment out code to isolate)
- Check recent changes (git blame, git log)
- Compare with working version
- Review documentation/API specs
- Check for race conditions/timing issues
- Verify assumptions with assertions

### Phase 3: Implement Fix

**Goal**: Apply minimal, targeted fix

**Steps**:
1. Design the simplest possible fix
2. Implement the fix with minimal changes
3. Consider edge cases
4. Add/update tests if applicable

**Fix Guidelines**:
- **MINIMAL**: Change only what's necessary
- **SAFE**: Don't break existing functionality
- **CLEAR**: Make the fix obvious in code
- **TESTED**: Verify with existing and new tests

**Implementation Template**:
```markdown
### Fix Implementation

**Approach**: [brief description of fix strategy]

**Changes**:
1. `file:line` - [what changed and why]
2. `file:line` - [what changed and why]

**Edge Cases Considered**:
- [edge case 1]: [how it's handled]
- [edge case 2]: [how it's handled]

**Tests Added/Modified**:
- [test name]: [what it verifies]
```

**Before/After Example**:
```markdown
BEFORE (file:line):
[code snippet showing the bug]

AFTER (file:line):
[code snippet showing the fix]

EXPLANATION:
[why this fixes the issue]
```

### Phase 4: Verify the Fix

**Goal**: Confirm the fix works and doesn't break anything

**Steps**:
1. Verify original reproduction case is fixed
2. Run existing tests
3. Run new tests (if added)
4. Check for regressions
5. Consider broader impact

**Verification Checklist**:
```markdown
VERIFICATION:
- [ ] Original reproduction case passes
- [ ] All existing tests pass
- [ ] New tests pass (if added)
- [ ] No lint/type errors introduced
- [ ] Edge cases handled
- [ ] No performance regression
- [ ] Documentation updated (if needed)
```

**Test Commands**:
```bash
# Run relevant tests
cargo test <test_name>

# Run all tests
cargo test

# Run linter
cargo clippy

# Format check
cargo fmt -- --check
```

## Common Debugging Patterns

### Pattern 1: Null/None Value Issues
```markdown
SYMPTOM: NullPointerException, None.unwrap() panic, undefined errors

INVESTIGATION:
1. Trace where value originates
2. Check initialization order
3. Verify conditional logic
4. Check for race conditions

FIX:
- Add null checks/validation
- Use Option/Result types properly
- Provide default values where appropriate
```

### Pattern 2: State Management Issues
```markdown
SYMPTOM: Wrong state, stale data, unexpected mutations

INVESTIGATION:
1. Trace state changes over time
2. Check for unintended mutations
3. Verify state transitions
4. Check async/await timing

FIX:
- Make state immutable where possible
- Add state validation
- Fix mutation logic
- Handle async properly
```

### Pattern 3: Configuration Issues
```markdown
SYMPTOM: Wrong behavior in specific environments

INVESTIGATION:
1. Check config files
2. Verify environment variables
3. Check feature flags
4. Compare environments

FIX:
- Add missing config
- Fix config precedence
- Add validation
- Update documentation
```

### Pattern 4: Integration Issues
```markdown
SYMPTOM: Failures when components interact

INVESTIGATION:
1. Check API contracts
2. Verify data formats
3. Check error handling
4. Trace integration points

FIX:
- Align API contracts
- Fix data transformation
- Improve error handling
- Add integration tests
```

## Debugging Tools Reference

### Logging
```rust
// Rust tracing
use tracing::{debug, info, warn, error};

debug!("Processing item: {:?}", item);
info!("Operation completed successfully");
warn!("Unexpected condition: {}", condition);
error!("Operation failed: {:?}", error);
```

### Assertions
```rust
// Runtime assertions for debugging
assert!(condition, "message");
assert_eq!(actual, expected, "message");
debug_assert!(condition);  // Only in debug builds
```

### Git Investigation
```bash
# Find when bug was introduced
git bisect start
git bisect bad [current_commit]
git bisect good [last_known_good]

# Check recent changes
git log --oneline -20

# See what changed in a file
git log -p -- path/to/file

# Find who changed a line
git blame path/to/file
```

### Test-Driven Debugging
```rust
#[test]
fn test_bug_reproduction() {
    // Write a failing test that demonstrates the bug
    let result = function_with_bug(input);
    assert_eq!(result, expected_output);
}

#[test]
fn test_fix_verification() {
    // After fix, test should pass
    let result = function_with_fix(input);
    assert_eq!(result, expected_output);
}
```

## Anti-Patterns to Avoid

❌ **Don't**:
- Add random print statements everywhere
- Change multiple things at once
- Skip the reproduction phase
- Assume you know the cause without evidence
- Fix symptoms instead of root cause
- Ignore failing tests

✅ **Do**:
- Create minimal reproduction first
- One change at a time with verification
- Use systematic investigation
- Gather evidence with `file:line` references
- Fix the underlying cause
- Ensure all tests pass

## Integration with Kilo Workflow

This skill integrates with Kilo's development workflow:

1. **Scope Definition**: Identify the bug and affected files
2. **Todo List**: Break down debugging phases
3. **Implementation**: Apply minimal fix
4. **Build/Lint**: Ensure code compiles and passes linting
5. **Test**: Verify fix with tests

Use with other skills:
- **code-review**: After fixing, review the change
- **test-driven-development**: Write test first to capture bug
- **git-worktree**: Create clean environment for debugging