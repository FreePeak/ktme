#!/bin/bash

echo "Testing ktme MCP server with various inputs..."

# Test 1: Empty input
echo "Test 1: Empty input"
echo "" | timeout 1 /Users/linh.doan/work/harvey/ktme/target/release/ktme mcp start --stdio 2>&1 || echo "No output (expected)"
echo

# Test 2: Just newline
echo "Test 2: Just newline"
printf "\n" | timeout 1 /Users/linh.doan/work/harvey/ktme/target/release/ktme mcp start --stdio 2>&1 || echo "No output (expected)"
echo

# Test 3: Invalid JSON
echo "Test 3: Invalid JSON"
printf "invalid json\n" | /Users/linh.doan/work/harvey/ktme/target/release/ktme mcp start --stdio 2>&1
echo

# Test 4: JSON with missing method
echo "Test 4: JSON with missing method"
printf '{"jsonrpc":"2.0","id":1}\n' | /Users/linh.doan/work/harvey/ktme/target/release/ktme mcp start --stdio 2>&1
echo

# Test 5: Valid initialize
echo "Test 5: Valid initialize"
printf '{"jsonrpc":"2.0","method":"initialize","id":1}\n' | /Users/linh.doan/work/harvey/ktme/target/release/ktme mcp start --stdio 2>&1
echo