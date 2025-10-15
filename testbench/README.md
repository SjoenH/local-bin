# epcheck Testbench

A comprehensive test suite for the `epcheck` OpenAPI endpoint usage checker.

## Overview

The testbench provides versioned test cases to ensure `epcheck` works correctly and to track performance over time. It includes various scenarios from basic functionality to complex edge cases.

## Structure

```
testbench/
├── v1.0/                          # Current test version
│   ├── run-tests.sh              # Test runner script
│   ├── results/                  # Test results (generated)
│   ├── test-basic/               # Basic functionality tests
│   ├── test-complex/             # Complex scenarios
│   ├── test-edge-cases/          # Edge cases and error conditions
│   ├── test-performance/         # Performance benchmarks
│   └── test-options/             # CLI options testing
├── v1.1/                         # Future test versions
└── README.md                     # This file
```

## Test Categories

### Basic Tests (`test-basic`)
- Simple OpenAPI spec with mixed used/unused endpoints
- Tests auto-detection of spec files
- Validates basic parsing and reporting

### Complex Tests (`test-complex`)
- Nested paths with multiple parameters
- YAML format specifications
- Mixed client library patterns

### Edge Cases (`test-edge-cases`)
- Empty specifications
- Specifications with no endpoints
- Malformed or invalid specs
- Codebases with no endpoint usage

### Performance Tests (`test-performance`)
- Large specifications (10+ endpoints)
- Multiple source files
- Execution time and memory monitoring

### Options Tests (`test-options`)
- Different output formats (table, CSV, JSON)
- CLI flags (--unused-only, --verbose, --quick, etc.)
- Filtering options (--pattern)

## Running Tests

### Run All Tests
```bash
cd testbench/v1.0
./run-tests.sh
```

### Run Specific Test
```bash
cd testbench/v1.0/test-basic
../../../epcheck -d src --no-colors
```

## Test Configuration

Each test case includes a `config.json` file:

```json
{
  "name": "test-name",
  "description": "Test description",
  "args": ["-d", "src", "--no-colors"],
  "expected_exit_code": 0,
  "expected_files": {
    "table": "expected-table.txt"
  },
  "performance": {
    "max_time_seconds": 10,
    "max_memory_mb": 50
  },
  "skip_tools": ["fzf"]
}
```

## Expected Output Files

Test cases include expected output files for validation:
- `expected-table.txt` - Expected table format output
- `expected-json.json` - Expected JSON format output
- `expected-csv.txt` - Expected CSV format output

## Performance Monitoring

The test runner tracks:
- **Execution time**: Wall clock time for each test
- **Memory usage**: Peak memory consumption (future enhancement)
- **Regression detection**: Alerts on performance degradation

## Adding New Tests

1. Create a new test directory: `testbench/v1.0/test-new-feature/`
2. Add OpenAPI spec file(s)
3. Create source code that uses some endpoints
4. Run the test to generate actual output
5. Create expected output files
6. Add `config.json` with test configuration

## Versioning

Tests are versioned to track changes over time:
- **v1.0**: Initial comprehensive test suite
- **v1.1**: Additional edge cases and performance tests
- **v2.0**: Major epcheck feature changes

## CI/CD Integration

The testbench is designed for automated testing:

```yaml
# Example GitHub Actions
- name: Run epcheck tests
  run: |
    cd testbench/v1.0
    ./run-tests.sh
```

## Contributing

When making changes to epcheck:
1. Run the full test suite
2. Add new test cases for new features
3. Update expected outputs if behavior changes intentionally
4. Ensure performance doesn't regress significantly

## Troubleshooting

### Tests Failing
- Check that `epcheck` script is executable and in the correct path
- Verify OpenAPI specs are valid JSON/YAML
- Ensure source files contain expected endpoint usage patterns

### Performance Issues
- Tests taking too long may indicate performance regressions
- Check system resources and tool availability
- Review epcheck implementation for optimization opportunities

## Future Enhancements

- [ ] Memory usage monitoring
- [ ] CPU profiling
- [ ] More comprehensive output validation (diff-based)
- [ ] Integration with coverage tools
- [ ] Automated test case generation
- [ ] Cross-platform testing (Linux, macOS, Windows)</content>
</xai:function_call">/api/projects/{projectId}/tasks