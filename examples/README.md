# Soplang examples

These programs demonstrate Soplang language features and are used for regression testing. Each `.sop` file (except `43_random_function.sop`) has a matching `.expected` file; tests run the program and compare stdout to `.expected`.

## Running examples

```bash
# Run one example
./target/release/soplang examples/hello.sop

# Run by number (1-based)
./target/release/soplang --example 1
```

## Regenerating .expected files

After changing the language or fixing bugs, regenerate expected output:

```bash
cd examples
for f in *.sop; do
  stem="${f%.sop}"
  [ "$stem" = "43_random_function" ] && continue   # non-deterministic
  ../target/release/soplang "$f" 2>/dev/null > "${stem}.expected"
done
```

**Note:** `43_random_function.sop` is non-deterministic (uses `xul()`); it has no `.expected` and is skipped in the test suite.

## Coverage index

| Example | Topic |
|---------|--------|
| **hello.sop** | Minimal: `qor()` |
| **01_dynamic_typing** | `door`, reassignment, multiple types |
| **02_static_typing** | `abn`, `qoraal`, `jajab`, `bool`, `teed`, `walax` |
| **03_type_checking** | `nooc()` |
| **04_arithmetic_operators** | `+`, `-`, `*`, `/`, `%` |
| **05_comparison_operators** | `==`, `!=`, `>`, `<`, `>=`, `<=` |
| **06_logical_operators** | `&&`, `\|\|`, `!` (iyo, ama, ma) |
| **07_conditional_statements** | `haddii`, `haddii_kale`, `ugudambeyn` |
| **08_for_loops** | `kuceli` (for), `ilaa` |
| **09_switch_case** | `dooro`, `xaalad`, `ugudambeyn` |
| **10_functions** | `hawl`, `celi`, parameters, return values |
| **11_list_operations** | `teed`, list literals, indexing, methods |
| **12_object_operations** | `walax`, properties, `fure()`, nested objects |
| **13_type_conversion** | `abn()`, `qoraal()`, `bool()` |
| **14_comparison_assignment** | Comparison in assignment |
| **15_switch_case** | Switch (extended: expressions, nested, function result) |
| **16_list_copy** | `nuqul()` |
| **17_list_reverse** | `rog()` |
| **18_list_clear** | `nadiifi()` |
| **19_list_filter** | `shaandhee()` |
| **20_constants** | `madoor` |
| **21_practical_constants** | Constants in expressions |
| **22_constant_reassignment_test** | Error: reassigning constant (negative test) |
| **23_constant_type_check** | Typed constants |
| **24_constant_type_error_test** | Type error (negative test) |
| **25_constant_type_error** | Constant type error (negative test) |
| **26_list_slice** | `jar()` slicing |
| **27_list_transform** | `aaddin()` |
| **28_list_raadso_and_negative_indexing** | `raadso()`, negative index |
| **29_object_copy** | `nuqul()` on walax |
| **30_object_clear** | `nadiifi()` on walax |
| **31_object_values** | `qiime()` |
| **32_object_entries** | `lamaane()` |
| **33_string_methods** | String methods overview |
| **34_string_contains** | `leeyahay()` |
| **35_math_floor** | `daji()` |
| **36_math_ceil** | `kor()` |
| **37_string_endswith** | `dhamaad()` |
| **38_string_startswith** | `bilow()` |
| **39_string_replace** | `beddel()` |
| **40_string_join** | `kudar()` (join) |
| **41_string_slice** | `jar()` on string |
| **42_universal_length** | `dherer()` |
| **43_random_function** | `xul()` (non-deterministic; no .expected) |
| **44_while_loops** | `intay` (while) |
| **45_user_input** | `gelin()` (simulated for tests) |
| **46_list_sort** | `habee()` |
| **47_list_filter_complex** | `shaandhee()` (complex predicates) |
| **return_test.sop** | `celi` with comparison expressions |

Negative tests (22, 24, 25) are intended to trigger errors; their `.expected` may be empty or contain error output depending on test setup.
