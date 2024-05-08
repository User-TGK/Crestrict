""" Auto execution and result comparison of the provided examples

This script assumes all example binaries have been build by an invocation of `make`.

"""

import argparse
import os
import subprocess

GCC_SUFFIX = "_gcc"
CLANG_SUFFIX = "_clang"

ERR_UB = '"An object which has been modified is accessed through an expression based on a restrict-qualified pointer and another lvalue not also based on said pointer."'


# Examples to skip for GCC (e.g. because of non-termination)
gcc_exceptions = {
    "thesis/thesis-strange-delay-check-inlined",
    "thesis/thesis-strange-delay-check",
    "other/strange-delay-check",
}

# Examples to skip for Clang (e.g. because of non-termination)
clang_exceptions = {
    "thesis/thesis-strange-delay-check-inlined",
    "thesis/thesis-strange-delay-check",
    "other/strange-delay-check",
}

# Map of examples that have a defined behavior and their expected output.
# The key represents the file basename (so the .c extension is omitted).
defined_behavior_examples = {
    # Array of restrict pointers
    "aggregate/aliasing-array-readonly-array": "02",
    "aggregate/aliasing-array-readonly-value": "",

    # Global restrict pointers
    "global/global-based-on-local-1-q-qualified": "",
    "global/global-based-on-local-2-both-qualified": "",
    "global/global-based-on-local-2-p-qualified": "",
    "global/global-based-on-local-2-q-qualified": "",
    "global/local-based-on-global-p-qualified": "",

    # KCC unit tests
    "kcc/kcc-unit-fail-j068c": "",
    # "kcc/kcc-unit-pass-j068": "",
    # "kcc/kcc-unit-pass-j069": "",

    # Nested pointers
    "nested/based-on-nested": "",
    "nested/inner-based-on": "",
    "nested/inner-qualified-both-additional-child-ptr-main-2": "",
    "nested/inner-qualified-both-additional-child-ptr-main-4": "",
    "nested/inner-qualified-both-main-2": "",
    "nested/inner-qualified-both-main-4": "",
    "nested/inner-qualified-invoke-twice": "",
    "nested/read-via-inner-twice": "",
    "nested/write-via-inner-twice": "",

    # Examples from the ISO/IEC C11 standard
    "standard/c11-example-2-no-overlap": "",
    "standard/c11-example-3-double-read": "0",

    # Examples included in the thesis
    "thesis/thesis-c-in-k-invalid-merge": "",
    "thesis/thesis-example-as-mut-ptr-2": "",
    "thesis/thesis-filtering-bases-global": "",

    # Other
    "other/based-on-wording-defect": "13",
    "other/clarifying-restrict-article": "4",
    "other/copy-index-loop-restrict-p": "0123401234",
    "other/copy-index-loop-restrict-q": "0123401234",
    "other/copy-index-loop-restrict": "0123401234",
    "other/copy-index-loop": "0123401234",
    "other/copy-recursive-restrict-p": "0123401234",
    "other/copy-recursive-restrict-q": "0123401234",
    "other/copy-recursive-restrict": "0123401234",
    "other/copy-recursive": "0123401234",
    "other/copy-while-increment-restrict-p": "0123401234",
    "other/copy-while-increment-restrict-q": "0123401234",
    "other/copy-while-increment-restrict": "0123401234",
    "other/copy-while-increment": "0123401234",
    "other/drop-restrict": "",
    "other/read-through-aliasing-restrict": "",
    "other/retain-bases-assignment": "",
    "other/unrelated-restrict-block": "",
    "other/write-twice": "",
}

# Map of examples that have an undefined behavior according to the strict C standard,
# and the expected error by the interpreter.
# The key represents the file basename (so the .c extension is omitted).
undefined_behavior_examples = {
    # Array of restrict pointers
    "aggregate/aliasing-array-child-ptr": ERR_UB,
    "aggregate/aliasing-array-modify-1": ERR_UB,
    "aggregate/aliasing-array-modify-2": ERR_UB,

    # Global restrict pointers
    "global/global-based-on-local-1-both-qualified": ERR_UB,
    "global/global-based-on-local-1-p-qualified": ERR_UB,
    "global/local-based-on-global-both-qualified": ERR_UB,
    "global/local-based-on-global-q-qualified": ERR_UB,
    "global/restrict-block": ERR_UB,

    # KCC unit tests
    "kcc/kcc-unit-fail-j068a": ERR_UB,
    "kcc/kcc-unit-fail-j068b": ERR_UB,
    # "kcc/kcc-unit-fail-j069a": ERR_UB,
    # "kcc/kcc-unit-fail-j069b": ERR_UB,
    # "kcc/kcc-unit-fail-j069c": ERR_UB,

    # Nested pointers
    "nested/double-qualified-both-additional-child-ptr-main-1": ERR_UB,
    "nested/double-qualified-both-additional-child-ptr-main-2": ERR_UB,
    "nested/double-qualified-both-additional-child-ptr-main-3": ERR_UB,
    "nested/double-qualified-both-additional-child-ptr-main-4": ERR_UB,
    "nested/double-qualified-both-main-1": ERR_UB,
    "nested/double-qualified-both-main-2": ERR_UB,
    "nested/double-qualified-both-main-3": ERR_UB,
    "nested/double-qualified-both-main-4": ERR_UB,
    "nested/inner-qualified-both-additional-child-ptr-main-1": ERR_UB,
    "nested/inner-qualified-both-additional-child-ptr-main-3": ERR_UB,
    "nested/inner-qualified-both-main-1": ERR_UB,
    "nested/inner-qualified-both-main-3": ERR_UB,
    "nested/p-qualified-both-additional-child-ptr-main-1": ERR_UB,
    "nested/p-qualified-both-additional-child-ptr-main-2": ERR_UB,
    "nested/p-qualified-both-additional-child-ptr-main-3": ERR_UB,
    "nested/p-qualified-both-additional-child-ptr-main-4": ERR_UB,
    "nested/p-qualified-both-main-1": ERR_UB,
    "nested/p-qualified-both-main-2": ERR_UB,
    "nested/p-qualified-both-main-3": ERR_UB,
    "nested/p-qualified-both-main-4": ERR_UB,
    "nested/simple-loc-distinction": ERR_UB,
    "nested/triple-qualified-both-additional-child-ptr-main-1": ERR_UB,
    "nested/triple-qualified-both-additional-child-ptr-main-2": ERR_UB,
    "nested/triple-qualified-both-additional-child-ptr-main-3": ERR_UB,
    "nested/triple-qualified-both-additional-child-ptr-main-4": ERR_UB,
    "nested/triple-qualified-both-main-1": ERR_UB,
    "nested/triple-qualified-both-main-2": ERR_UB,
    "nested/triple-qualified-both-main-3": ERR_UB,
    "nested/triple-qualified-both-main-4": ERR_UB,

    # Examples from the ISO/IEC C11 standard
    "standard/c11-example-2-overlap": ERR_UB,
    # "standard/c11-example-4-assign-between-restrict": "", # Unimplemented

    # Examples included in the MSc thesis
    "thesis/thesis-arr-same-base": ERR_UB,
    "thesis/thesis-manual-free": ERR_UB,
    "thesis/thesis-nested-restrict-ptrs": ERR_UB,
    "thesis/thesis-strange-delay-check-inlined": ERR_UB,  # GCC/Clang simply optimize to infinite loop.
    "thesis/thesis-strange-delay-check": ERR_UB, # Same as above.

    # Other
    "other/aliasing-restrict-construction": ERR_UB,
    "other/assign-no-restrict-overlap": ERR_UB,
    "other/assign-restrict-overlap": ERR_UB,
    "other/assign-to-self": ERR_UB,
    "other/c-in-k-free-invalid": ERR_UB,
    "other/copy-index-loop-restrict-overlap": ERR_UB,
    "other/copy-while-increment-restrict-overlap": ERR_UB,
    "other/counter-example-based-on-outmost-unique": ERR_UB,
    "other/f": ERR_UB,
    "other/f2": ERR_UB,
    "other/f3": ERR_UB,
    "other/f4": ERR_UB,
    "other/f5": ERR_UB,
    "other/funcall": ERR_UB,
    "other/funcall2": ERR_UB,
    "other/indirections": ERR_UB,
    "other/read-after-write": ERR_UB,
    "other/share_until_write": ERR_UB,
    "other/strange-delay-check": ERR_UB,
}


def main():
    parser = argparse.ArgumentParser(description='Auto execute all examples')
    parser.add_argument('-interpreter-log', type=str,
                        default='error', required=False, help='The Rust log level')

    args = parser.parse_args()

    db_path_prefix = os.path.dirname(__file__) + "/DB/"
    ub_path_prefix = os.path.dirname(__file__) + "/UB/"

    os.environ["RUST_LOG"] = args.interpreter_log + ",stdout,stderr"
    interp_path = "../interp/target/debug/interp"

    # Run the test cases for the defined behavior examples
    for example, expected_output in defined_behavior_examples.items():
        result_gcc = "skipped" if example in gcc_exceptions else subprocess.run(
            [db_path_prefix + example + GCC_SUFFIX], stdout=subprocess.PIPE).stdout.decode('utf-8').strip()
        result_clang = "skipped" if example in clang_exceptions else subprocess.run(
            [db_path_prefix + example + CLANG_SUFFIX], stdout=subprocess.PIPE).stdout.decode('utf-8').strip()
        result_interp = subprocess.run(
            [interp_path, "--source-file", db_path_prefix + example + '.c'], stdout=subprocess.PIPE, stderr=subprocess.STDOUT).stdout.decode('utf-8').strip()

        assert (result_gcc == expected_output), \
            f"GCC DB test for {example} failed.\nOutput was '{result_gcc}'.Expected output is '{expected_output}'."
        assert (result_clang == expected_output), \
            f"CLANG DB test for {example} failed.\nOutput was '{result_clang}'.Expected output is '{expected_output}'."
        assert (result_interp == expected_output), \
            f"Interpreter DB test for {example} failed.\nOutput was '{result_interp}'.Expected output is '{expected_output}'."

        print(f"Successfully passed DB test '{example}'")

    print("\n*************************************************\n")

    # Run the undefined behavior tests and print the output of the different compilers
    for example, expected_ub_error in undefined_behavior_examples.items():
        result_gcc = "skipped" if example in gcc_exceptions else subprocess.run(
            [ub_path_prefix + example + GCC_SUFFIX], stdout=subprocess.PIPE).stdout.decode('utf-8').strip()
        result_clang = "skipped" if example in clang_exceptions else subprocess.run(
            [ub_path_prefix + example + CLANG_SUFFIX], stdout=subprocess.PIPE).stdout.decode('utf-8').strip()
        result_interp = subprocess.run(
            [interp_path, "--source-file", ub_path_prefix + example + '.c'], stdout=subprocess.PIPE, stderr=subprocess.STDOUT).stdout.decode('utf-8').strip()

        assert (result_interp == expected_ub_error), \
            f"Interpreter UB test for {example} failed.\nOutput was '{result_interp}'.Expected output is '{expected_ub_error}'."

        print(f"""UB Test case {example}
                    GCC output:   '{result_gcc}'
                    CLANG output: '{result_clang}'
                    Equivalent: {'yes' if result_gcc == result_clang else 'no'}
                    Interpreter passed successfully""")

if __name__ == "__main__":
    main()
