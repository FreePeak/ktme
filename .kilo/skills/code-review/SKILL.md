---
name: code-review
version: "1.0.0"
description: Comprehensive code review framework for evaluating code quality, security, performance, and maintainability
---

# Code Review

A systematic approach to reviewing code changes with focus on quality, security, performance, and maintainability.

## When to Use

Use this skill when:
- Reviewing pull requests
- Evaluating proposed changes
- Conducting pre-commit reviews
- Performing security audits
- Analyzing code for refactoring opportunities

## Review Framework

### Phase 1: Understand Context

**Goal**: Understand what changed and why

**Steps**:
1. Read PR description or commit message
2. Identify related issues/tickets
3. Understand the business requirement
4. Note the scope of changes

**Context Checklist**:
```markdown
CONTEXT:
- Issue/PR: [link or number]
- Author: [who made the changes]
- Scope: [files/directories affected]
- Purpose: [what problem it solves]
- Dependencies: [any related PRs or changes]
```

**Questions to Answer**:
- What problem does this solve?
- Is this the right approach?
- Are there alternative solutions?
- What's the impact on existing code?

### Phase 2: Review the Diff

**Goal**: Examine actual code changes

**Steps**:
1. Review files changed (start with most important)
2. Check for unexpected changes
3. Examine each diff hunk carefully
4. Note questions and concerns

**Diff Review Template**:
```markdown
## Files Changed

### Critical Files
- `file1:line` - [concern or observation]
- `file2:line` - [concern or observation]

### Supporting Files
- `file3:line` - [concern or observation]

### Test Files
- `test_file:line` - [concern or observation]

## Observations
1. [observation with file:line reference]
2. [observation with file:line reference]
```

### Phase 3: Quality Checks

**Categories to Evaluate**:

#### 3.1 Correctness
```markdown
CORRECTNESS:
- [ ] Logic is correct
- [ ] Handles edge cases
- [ ] Error handling is appropriate
- [ ] No off-by-one errors
- [ ] Null/None handling is proper
- [ ] Type conversions are safe

ISSUES:
- `file:line` - [description of issue]
```

#### 3.2 Security
```markdown
SECURITY:
- [ ] No SQL injection vulnerabilities
- [ ] Input validation is proper
- [ ] No hardcoded secrets
- [ ] Authentication/authorization checked
- [ ] No XSS vulnerabilities (if web)
- [ ] Sensitive data is protected
- [ ] Rate limiting considered
- [ ] Dependencies are safe

ISSUES:
- `file:line` - [description of security concern]
```

**Security Patterns to Check**:
```rust
// ❌ Bad: SQL injection risk
let query = format!("SELECT * FROM users WHERE id = {}", user_input);

// ✅ Good: Parameterized query
let query = "SELECT * FROM users WHERE id = ?1";
conn.query_row(query, params![user_input], |row| { ... });

// ❌ Bad: Hardcoded secret
let api_key = "sk-abc123...";

// ✅ Good: Environment variable
let api_key = std::env::var("API_KEY")?;

// ❌ Bad: No input validation
fn process_input(input: &str) { ... }

// ✅ Good: Validated input
fn process_input(input: &str) -> Result<()> {
    if input.len() > MAX_LENGTH {
        return Err(Error::InputTooLong);
    }
    // Additional validation...
    Ok(())
}
```

#### 3.3 Performance
```markdown
PERFORMANCE:
- [ ] No N+1 queries
- [ ] Efficient algorithms
- [ ] Proper use of caching
- [ ] No unnecessary allocations
- [ ] Async operations used correctly
- [ ] Database queries optimized

ISSUES:
- `file:line` - [description of performance concern]
```

**Performance Patterns**:
```rust
// ❌ Bad: N+1 query problem
for user in users {
    let posts = get_posts_for_user(user.id).await?;  // N queries!
}

// ✅ Good: Batch query
let user_ids: Vec<i64> = users.iter().map(|u| u.id).collect();
let posts = get_posts_for_users(&user_ids).await?;  // 1 query

// ❌ Bad: Unnecessary clone
let data = large_vec.clone();
process_data(&data);

// ✅ Good: Reference when possible
process_data(&large_vec);

// ❌ Bad: Inefficient string concatenation
let mut s = String::new();
for part in parts {
    s += part;  // O(n²) complexity
}

// ✅ Good: Use join or with_capacity
let s = parts.join("");
// or
let mut s = String::with_capacity(estimated_size);
for part in parts {
    s.push_str(part);
}
```

#### 3.4 Maintainability
```markdown
MAINTAINABILITY:
- [ ] Code is readable
- [ ] Functions are focused
- [ ] Names are descriptive
- [ ] No code duplication
- [ ] Proper abstraction levels
- [ ] Easy to modify
- [ ] Follows project conventions

ISSUES:
- `file:line` - [description of maintainability concern]
```

**Maintainability Patterns**:
```rust
// ❌ Bad: Magic numbers
if status == 200 { ... }

// ✅ Good: Named constants
const HTTP_OK: u16 = 200;
if status == HTTP_OK { ... }

// ❌ Bad: Long function doing multiple things
fn process_order(order: Order) -> Result<()> {
    // Validate order (20 lines)
    // Calculate pricing (30 lines)
    // Update inventory (25 lines)
    // Send notifications (15 lines)
    // Log everything (10 lines)
}

// ✅ Good: Focused functions
fn process_order(order: Order) -> Result<()> {
    validate_order(&order)?;
    let total = calculate_total(&order)?;
    update_inventory(&order)?;
    send_notifications(&order)?;
    log_order(&order, total)?;
    Ok(())
}

// ❌ Bad: Duplicated logic
fn process_user(u: &User) { /* duplicate validation */ }
fn process_admin(u: &Admin) { /* duplicate validation */ }

// ✅ Good: Extracted common logic
trait Validatable {
    fn validate(&self) -> Result<()>;
}

fn process_entity<T: Validatable>(entity: &T) -> Result<()> {
    entity.validate()?;
    // ...
}
```

#### 3.5 Testing
```markdown
TESTING:
- [ ] New code is tested
- [ ] Tests are meaningful
- [ ] Edge cases covered
- [ ] Tests are maintainable
- [ ] Mocks used appropriately
- [ ] Test names are descriptive

ISSUES:
- `file:line` - [description of testing concern]
```

**Testing Patterns**:
```rust
// ❌ Bad: Vague test name
#[test]
fn test_process() { ... }

// ✅ Good: Descriptive test name
#[test]
fn test_process_returns_error_when_input_is_empty() { ... }

// ❌ Bad: Testing implementation details
#[test]
fn test_internal_helper_function() { ... }

// ✅ Good: Testing public behavior
#[test]
fn test_public_api_returns_expected_result() { ... }

// ❌ Bad: Not testing edge cases
#[test]
fn test_happy_path() { ... }

// ✅ Good: Comprehensive coverage
#[test]
fn test_happy_path() { ... }

#[test]
fn test_empty_input() { ... }

#[test]
fn test_max_length_input() { ... }

#[test]
fn test_invalid_input_returns_error() { ... }
```

### Phase 4: Provide Feedback

**Goal**: Communicate findings clearly and constructively

**Feedback Categories**:
- 🔴 **MUST FIX**: Critical issues that block merge
- 🟡 **SHOULD FIX**: Important improvements recommended
- 🟢 **SUGGESTION**: Nice-to-have improvements
- 💬 **QUESTION**: Clarification needed
- ℹ️ **NOTE**: Information for awareness

**Feedback Template**:
```markdown
## Review Summary

**Overall**: [Approve | Request Changes | Comment]
**Confidence**: [High | Medium | Low]

### 🔴 Must Fix
1. `file:line` - [issue description]
   **Why**: [explanation]
   **Suggestion**: [how to fix]

### 🟡 Should Fix
1. `file:line` - [issue description]
   **Why**: [explanation]
   **Suggestion**: [how to fix]

### 🟢 Suggestions
1. `file:line` - [improvement suggestion]
   **Benefit**: [what it improves]

### 💬 Questions
1. `file:line` - [question]
   **Context**: [why you're asking]

### ✅ What's Good
- [positive observation 1]
- [positive observation 2]
```

**Writing Good Feedback**:
```markdown
# ❌ Bad: Vague and unhelpful
"This code is wrong."

# ✅ Good: Specific and actionable
"file:42 - This function doesn't handle the case where `users` is empty.
ACTUAL: Panics with index out of bounds
EXPECTED: Should return an empty vec or error
SUGGESTION: Add check: `if users.is_empty() { return Ok(vec![]); }`"
```

## Specialized Review Checklists

### Security Review Checklist
```markdown
## Security Review

### Input Validation
- [ ] All user inputs are validated
- [ ] Input length limits enforced
- [ ] Input type checking
- [ ] Input sanitization for display

### Authentication & Authorization
- [ ] Auth checks are present
- [ ] Auth checks are correct
- [ ] Role-based access control verified
- [ ] Session management is secure

### Data Protection
- [ ] Sensitive data encrypted at rest
- [ ] Sensitive data encrypted in transit
- [ ] No secrets in logs
- [ ] No secrets in error messages

### Injection Prevention
- [ ] SQL queries use parameterized statements
- [ ] No command injection
- [ ] No path traversal vulnerabilities
- [ ] Proper output encoding

### Dependencies
- [ ] New dependencies are necessary
- [ ] New dependencies are trusted
- [ ] No known vulnerabilities in dependencies
```

### Performance Review Checklist
```markdown
## Performance Review

### Database
- [ ] Queries are optimized
- [ ] Indexes used appropriately
- [ ] N+1 queries avoided
- [ ] Transactions used correctly

### Memory
- [ ] No memory leaks
- [ ] Large allocations justified
- [ ] References used instead of clones
- [ ] Buffers sized appropriately

### Concurrency
- [ ] Async used correctly
- [ ] No race conditions
- [ ] Locks are necessary and minimal
- [ ] Deadlock potential analyzed

### Caching
- [ ] Caching strategy is appropriate
- [ ] Cache invalidation is correct
- [ ] Cache keys are well-designed
```

### API Design Review Checklist
```markdown
## API Design Review

### Consistency
- [ ] Naming follows conventions
- [ ] Parameter ordering is consistent
- [ ] Response format is consistent
- [ ] Error format is consistent

### Documentation
- [ ] All endpoints documented
- [ ] Parameters documented
- [ ] Response schema documented
- [ ] Error codes documented

### Versioning
- [ ] Breaking changes handled correctly
- [ ] Version in URL or header
- [ ] Backwards compatibility maintained

### Error Handling
- [ ] Errors are informative
- [ ] Errors are actionable
- [ ] Error codes are consistent
- [ ] HTTP status codes are correct
```

## Review Workflow Integration

### Pre-Commit Review
```bash
# Run before committing
cargo fmt -- --check
cargo clippy -- -D warnings
cargo test

# Review your own changes
git diff
```

### PR Review Process
```bash
# Fetch PR branch
git fetch origin pull/123/head:pr-123
git checkout pr-123

# Run checks
cargo test
cargo clippy

# Review specific files
git diff main...HEAD -- path/to/file
```

### Continuous Integration
Ensure CI runs:
- Lint checks
- Type checks
- Test suite
- Security scans
- Performance benchmarks (if applicable)

## Anti-Patterns to Avoid

❌ **Don't**:
- Nitpick style preferences
- Block PRs for minor improvements
- Be vague or unhelpful
- Focus only on finding faults
- Review without understanding context

✅ **Do**:
- Focus on correctness and maintainability
- Balance thoroughness with pragmatism
- Provide specific, actionable feedback
- Acknowledge good practices
- Understand the problem being solved
- Consider the author's perspective

## Integration with Kilo

This skill integrates with Kilo's workflow:

1. **Evidence-Based**: All feedback includes `file:line` references
2. **Minimal Changes**: Focus on what matters most
3. **Constructive**: Provide solutions, not just problems
4. **Systematic**: Follow structured review process

Use with other skills:
- **systematic-debugging**: When review reveals bugs
- **test-driven-development**: When suggesting new tests
- **documentation**: When docs need updates