---
name: test-driven-development
version: "1.0.0"
description: Enforce Red-Green-Refactor TDD workflow for writing tests before implementation
---

# Test-Driven Development

A disciplined approach to development where tests drive the design and implementation of code.

## When to Use

Use this skill when:
- Implementing new features
- Fixing bugs (write test first to capture bug)
- Refactoring existing code
- Building new modules or components
- Writing library code

## The TDD Cycle

### The Three Laws of TDD

1. **Red**: Write a failing test before writing production code
2. **Green**: Write the minimal code to make the test pass
3. **Refactor**: Clean up the code while keeping tests green

### Visual Workflow

```
┌─────────┐
│   RED   │ ──► Write failing test
└────┬────┘
     │
     ▼
┌─────────┐
│  GREEN  │ ──► Write minimal code to pass
└────┬────┘
     │
     ▼
┌──────────┐
│ REFACTOR │ ──► Improve code quality
└─────┬────┘
      │
      └──► Repeat
```

## Phase 1: RED - Write Failing Test

**Goal**: Write a test that fails because the feature doesn't exist yet

### Steps

1. **Understand the requirement**
   - What should the code do?
   - What are the inputs and outputs?
   - What are the edge cases?

2. **Write a minimal test**
   - Test one thing only
   - Use descriptive test name
   - Focus on behavior, not implementation

3. **Run the test**
   - Verify it fails for the right reason
   - The test should fail, not crash
   - Error message should be informative

### Test Template

```rust
#[test]
fn test_<behavior_description>() {
    // Arrange - Set up test data
    let input = /* test input */;
    let expected = /* expected output */;
    
    // Act - Execute the behavior
    let result = function_under_test(input);
    
    // Assert - Verify the outcome
    assert_eq!(result, expected);
}
```

### Examples

```rust
// ✅ Good: Test behavior with descriptive name
#[test]
fn test_calculate_total_returns_sum_of_item_prices() {
    let items = vec![
        Item { name: "A", price: 10.0 },
        Item { name: "B", price: 20.0 },
    ];
    
    let result = calculate_total(&items);
    
    assert_eq!(result, 30.0);
}

// ✅ Good: Test edge case
#[test]
fn test_calculate_total_returns_zero_for_empty_items() {
    let items = vec![];
    
    let result = calculate_total(&items);
    
    assert_eq!(result, 0.0);
}

// ✅ Good: Test error case
#[test]
fn test_parse_config_returns_error_for_invalid_toml() {
    let invalid_toml = "not valid toml [[[";
    
    let result = parse_config(invalid_toml);
    
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ConfigError::InvalidToml(_)));
}

// ❌ Bad: Testing implementation details
#[test]
fn test_internal_parser_helper() {
    // Don't test private functions directly
}

// ❌ Bad: Multiple assertions in one test
#[test]
fn test_everything() {
    assert_eq!(func1(), 1);
    assert_eq!(func2(), 2);
    assert_eq!(func3(), 3);  // If func2 fails, we don't test func3
}
```

### Test Naming Convention

```rust
// Pattern: test_<unit>_<scenario>_<expected_result>
#[test]
fn test_parse_date_with_valid_iso_format_returns_date() { }

#[test]
fn test_parse_date_with_invalid_format_returns_error() { }

#[test]
fn test_parse_date_with_empty_string_returns_error() { }

#[test]
fn test_parse_date_with_future_date_returns_date() { }
```

## Phase 2: GREEN - Make It Pass

**Goal**: Write the minimal code to make the test pass

### Steps

1. **Write minimal implementation**
   - Focus on passing the test
   - Don't anticipate future requirements
   - Hardcode if necessary (for now)

2. **Run the test**
   - Verify it passes
   - Don't worry about elegance yet

3. **Keep it simple**
   - Avoid over-engineering
   - No need to handle cases not tested yet

### Examples

```rust
// Test
#[test]
fn test_add_returns_sum_of_two_numbers() {
    assert_eq!(add(2, 3), 5);
}

// Green (minimal implementation)
fn add(a: i32, b: i32) -> i32 {
    5  // Hardcoded! That's OK for now
}

// Later, after more tests...
#[test]
fn test_add_with_negative_numbers() {
    assert_eq!(add(-1, 1), 0);
}

// Now we need proper implementation
fn add(a: i32, b: i32) -> i32 {
    a + b  // Generalized solution
}
```

### Minimal Implementation Pattern

```rust
// Start with the simplest thing that could possibly work
pub fn format_name(first: &str, last: &str) -> String {
    format!("{} {}", first, last)  // Simple concatenation
}

// As tests reveal edge cases, refine
#[test]
fn test_format_name_handles_whitespace() {
    assert_eq!(format_name("  John  ", "  Doe  "), "John Doe");
}

// Refined implementation
pub fn format_name(first: &str, last: &str) -> String {
    format!("{} {}", first.trim(), last.trim())
}
```

## Phase 3: REFACTOR - Clean It Up

**Goal**: Improve code quality while keeping tests green

### Steps

1. **Identify improvements**
   - Remove duplication
   - Improve naming
   - Simplify logic
   - Extract functions
   - Optimize if needed

2. **Make one change at a time**
   - Small, focused refactorings
   - Run tests after each change
   - Commit frequently

3. **Verify tests still pass**
   - Run full test suite
   - Ensure no behavior changed

### Refactoring Patterns

```rust
// Before refactor: Duplicate logic
fn process_user(user: &User) -> String {
    let name = format!("{} {}", user.first_name.trim(), user.last_name.trim());
    name
}

fn process_admin(admin: &Admin) -> String {
    let name = format!("{} {}", admin.first_name.trim(), admin.last_name.trim());
    name
}

// After refactor: Extract common logic
fn format_name(first: &str, last: &str) -> String {
    format!("{} {}", first.trim(), last.trim())
}

fn process_user(user: &User) -> String {
    format_name(&user.first_name, &user.last_name)
}

fn process_admin(admin: &Admin) -> String {
    format_name(&admin.first_name, &admin.last_name)
}
```

### Refactoring Checklist

```markdown
REFACTORING:
- [ ] Code is readable
- [ ] Names are descriptive
- [ ] Functions are focused
- [ ] No duplication
- [ ] Proper abstraction
- [ ] Tests still pass
```

## TDD Workflow

### Feature Development

```markdown
1. Write acceptance criteria
2. Break down into small units
3. For each unit:
   a. RED: Write failing test
   b. GREEN: Make it pass
   c. REFACTOR: Clean up
4. Integration test
5. Manual verification
```

### Bug Fixing

```markdown
1. Write test that reproduces bug
   - Test should fail (demonstrating bug exists)
2. Fix the code
   - Test should now pass
3. Refactor if needed
4. Verify no regression
```

### Example: Bug Fix TDD

```rust
// Step 1: Write test that demonstrates bug
#[test]
fn test_parse_json_handles_empty_array() {
    let json = "[]";
    let result = parse_json(json);
    
    // This fails because current implementation panics on empty array
    assert_eq!(result.unwrap(), vec![]);
}

// Step 2: Run test - it fails (RED)
// thread 'test_parse_json_handles_empty_array' panicked at 'index out of bounds'

// Step 3: Fix the code (GREEN)
fn parse_json(json: &str) -> Result<Vec<Item>> {
    let items: Vec<Item> = serde_json::from_str(json)?;
    Ok(items)  // Now handles empty array correctly
}

// Step 4: Run test - it passes (GREEN)

// Step 5: Refactor if needed (REFACTOR)
// Code is clean, no refactor needed

// Step 6: Run all tests
cargo test  // All tests pass
```

## Test Types

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_single_unit_of_work() {
        // Fast, isolated, focused
        let result = parse_input("test");
        assert!(result.is_ok());
    }
}
```

### Integration Tests

```rust
// tests/integration_test.rs
use ktme::*;
use tempfile::TempDir;

#[test]
fn test_full_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    
    // Initialize
    let db = Database::new(&db_path).unwrap();
    
    // Create
    let service = db.create_service("test-service").unwrap();
    
    // Read
    let retrieved = db.get_service("test-service").unwrap();
    assert_eq!(service.name, retrieved.unwrap().name);
    
    // Update
    db.update_service_description("test-service", "Updated").unwrap();
    
    // Delete
    db.delete_service("test-service").unwrap();
}
```

### Property-Based Tests

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_parse_format_roundtrip(s in "\\PC*") {
        // For any string, parsing and formatting should be reversible
        let parsed = parse_input(&s);
        if let Ok(value) = parsed {
            let formatted = format_output(&value);
            let reparsed = parse_input(&formatted).unwrap();
            assert_eq!(value, reparsed);
        }
    }
}
```

## Test Organization

### File Structure

```
src/
├── lib.rs
├── parser.rs
│   └── #[cfg(test)] mod tests { }
└── database.rs
    └── #[cfg(test)] mod tests { }

tests/
├── integration_test.rs
├── common/
│   └── mod.rs
└── fixtures/
    └── test_data.json
```

### Test Module Structure

```rust
// src/parser.rs
pub fn parse_input(input: &str) -> Result<ParsedData> {
    // Implementation
}

#[cfg(test)]
mod tests {
    use super::*;
    
    mod parse_input {
        use super::*;
        
        #[test]
        fn with_valid_input_returns_parsed_data() { }
        
        #[test]
        fn with_empty_input_returns_error() { }
        
        #[test]
        fn with_invalid_format_returns_error() { }
    }
    
    mod edge_cases {
        use super::*;
        
        #[test]
        fn handles_unicode_correctly() { }
        
        #[test]
        fn handles_very_long_input() { }
    }
}
```

## Best Practices

### Test Independence

```rust
// ❌ Bad: Tests depend on each other
static mut COUNTER: i32 = 0;

#[test]
fn test_increment() {
    unsafe { COUNTER += 1; }
    assert_eq!(unsafe { COUNTER }, 1);
}

#[test]
fn test_increment_again() {
    unsafe { COUNTER += 1; }
    assert_eq!(unsafe { COUNTER }, 2);  // Depends on test order!
}

// ✅ Good: Each test is independent
#[test]
fn test_increment() {
    let mut counter = 0;
    counter += 1;
    assert_eq!(counter, 1);
}

#[test]
fn test_increment_again() {
    let mut counter = 0;
    counter += 1;
    assert_eq!(counter, 1);  // Independent of other tests
}
```

### Test Readability

```rust
// ❌ Bad: Unclear test
#[test]
fn test1() {
    let x = func("a", "b");
    assert!(x);
}

// ✅ Good: Clear test
#[test]
fn test_validate_credentials_returns_true_for_valid_user() {
    let username = "valid_user";
    let password = "valid_pass123";
    
    let is_valid = validate_credentials(username, password);
    
    assert!(is_valid);
}
```

### Arrange-Act-Assert Pattern

```rust
#[test]
fn test_user_can_be_created_with_valid_data() {
    // Arrange - Set up test data
    let name = "John Doe";
    let email = "john@example.com";
    
    // Act - Perform the action
    let user = User::new(name, email);
    
    // Assert - Verify the result
    assert_eq!(user.name, name);
    assert_eq!(user.email, email);
}
```

## Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_parse_input

# Run tests in specific module
cargo test parser::tests

# Run tests matching pattern
cargo test "test_parse"

# Show println! output
cargo test -- --nocapture

# Run single test by name
cargo test test_specific_function --exact

# Run tests in parallel
cargo test -- --test-threads=4

# Run ignored tests
cargo test -- --ignored
```

## Anti-Patterns to Avoid

❌ **Don't**:
- Write tests after implementation
- Skip the RED phase
- Write tests that always pass
- Test implementation details
- Write large, complex tests
- Ignore failing tests

✅ **Do**:
- Write test first (RED)
- Keep tests simple and focused
- Test behavior, not implementation
- Refactor tests too
- Run tests frequently
- Keep tests fast

## Integration with Kilo

This skill integrates with Kilo's workflow:

1. **Evidence**: Tests provide evidence that code works
2. **Minimal Changes**: TDD promotes minimal implementation
3. **Refactoring**: Safe refactoring with test safety net
4. **Documentation**: Tests serve as living documentation

Use with other skills:
- **systematic-debugging**: When tests reveal bugs
- **code-review**: Tests are part of review
- **documentation**: Tests document behavior