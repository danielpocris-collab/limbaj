#include "ng.h"

#include <direct.h>
#include <process.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static void fail(const char *message) {
    fprintf(stderr, "test_main: %s\n", message);
    exit(1);
}

static void expect(bool condition, const char *message) {
    if (!condition) {
        fail(message);
    }
}

static void write_file_or_die(const char *path, const char *contents) {
    FILE *file = NULL;
    errno_t open_error = fopen_s(&file, path, "wb");
    if (open_error != 0 || file == NULL) {
        fail("failed to open temporary file");
    }

    size_t length = strlen(contents);
    size_t written = fwrite(contents, 1U, length, file);
    fclose(file);
    if (written != length) {
        fail("failed to write temporary file");
    }
}

static void read_file_or_die(const char *path, char *buffer, size_t buffer_size, size_t *out_length) {
    FILE *file = NULL;
    errno_t open_error = fopen_s(&file, path, "rb");
    if (open_error != 0 || file == NULL) {
        fail("failed to open temporary file for read");
    }

    size_t read = fread(buffer, 1U, buffer_size - 1U, file);
    fclose(file);
    buffer[read] = '\0';
    *out_length = read;
}

static void escape_ng_text_literal(const char *input, char *output, size_t output_size) {
    size_t out = 0U;
    for (size_t i = 0U; input[i] != '\0'; ++i) {
        unsigned char ch = (unsigned char)input[i];
        if (ch == '\\' || ch == '"') {
            if (out + 2U >= output_size) {
                fail("escaped ng string literal overflow");
            }
            output[out++] = '\\';
            output[out++] = (char)ch;
        } else {
            if (out + 1U >= output_size) {
                fail("escaped ng string literal overflow");
            }
            output[out++] = (char)ch;
        }
    }
    output[out] = '\0';
}

static void cleanup_temp_tree(const char *dir, const char *helper, const char *main_file, const char *exe) {
    remove(exe);
    remove(main_file);
    remove(helper);
    _rmdir(dir);
}

static void test_compile_with_modules(void) {
    char dir[256];
    char helper_path[320];
    char main_path[320];
    char exe_path[320];
    int pid = _getpid();

    snprintf(dir, sizeof(dir), "tests\\tmp_use_%d", pid);
    snprintf(helper_path, sizeof(helper_path), "%s\\helper.ng", dir);
    snprintf(main_path, sizeof(main_path), "%s\\main.ng", dir);
    snprintf(exe_path, sizeof(exe_path), "%s\\main.exe", dir);

    cleanup_temp_tree(dir, helper_path, main_path, exe_path);
    if (_mkdir(dir) != 0) {
        fail("failed to create temporary directory");
    }

    write_file_or_die(
        helper_path,
        "use text\n"
        "flow value() -> i64 {\n"
        "    check text.is_ng(\"ng\") {\n"
        "        yield 40\n"
        "    }\n"
        "    yield 0\n"
        "}\n");
    write_file_or_die(
        main_path,
        "use helper\n"
        "use array\n"
        "use slice\n"
        "use maybe\n"
        "flow main() -> i64 {\n"
        "    bind values :: [3]i64 = [7, 11, 24]\n"
        "    bind view :: []i64 = [1, 2, 40]\n"
        "    bind missing :: maybe[i64] = none\n"
        "    check maybe.is_none_i64(missing) {\n"
        "        yield helper.value() + array.sum3_i64(values) + slice.sum3_i64(view)\n"
        "    }\n"
        "    yield 0\n"
        "}\n");

    int compile_exit = ng_compile_source_file(main_path, exe_path, false);
    expect(compile_exit == 0, "expected module compile success");

    int run_exit = _spawnl(_P_WAIT, exe_path, exe_path, NULL);
    cleanup_temp_tree(dir, helper_path, main_path, exe_path);
    expect(run_exit == 125, "expected compiled program to return namespaced stdlib result");
}

static void test_compile_with_imported_forms(void) {
    char dir[256];
    char helper_path[320];
    char main_path[320];
    char exe_path[320];
    int pid = _getpid();

    snprintf(dir, sizeof(dir), "tests\\tmp_form_use_%d", pid);
    snprintf(helper_path, sizeof(helper_path), "%s\\helper.ng", dir);
    snprintf(main_path, sizeof(main_path), "%s\\main.ng", dir);
    snprintf(exe_path, sizeof(exe_path), "%s\\main.exe", dir);

    cleanup_temp_tree(dir, helper_path, main_path, exe_path);
    if (_mkdir(dir) != 0) {
        fail("failed to create temporary directory");
    }

    write_file_or_die(
        helper_path,
        "form Pair {\n"
        "    left :: i64,\n"
        "    right :: i64,\n"
        "}\n"
        "flow make_pair(base :: i64) -> Pair {\n"
        "    yield Pair{\n"
        "        left = base,\n"
        "        right = base + 1,\n"
        "    }\n"
        "}\n"
        "flow sum_pair(value :: Pair) -> i64 {\n"
        "    yield value.left + value.right\n"
        "}\n");
    write_file_or_die(
        main_path,
        "use helper\n"
        "flow main() -> i64 {\n"
        "    bind direct :: Pair = Pair{\n"
        "        left = 20,\n"
        "        right = 22,\n"
        "    }\n"
        "    bind built :: Pair = helper.make_pair(40)\n"
        "    yield helper.sum_pair(direct) + helper.sum_pair(built)\n"
        "}\n");

    int compile_exit = ng_compile_source_file(main_path, exe_path, false);
    expect(compile_exit == 0, "expected imported form compile success");

    int run_exit = _spawnl(_P_WAIT, exe_path, exe_path, NULL);
    cleanup_temp_tree(dir, helper_path, main_path, exe_path);
    expect(run_exit == 123, "expected compiled program to return imported form result");
}

static void test_compile_with_indexed_form_fields(void) {
    char dir[256];
    char main_path[320];
    char exe_path[320];
    int pid = _getpid();

    snprintf(dir, sizeof(dir), "tests\\tmp_indexed_form_%d", pid);
    snprintf(main_path, sizeof(main_path), "%s\\main.ng", dir);
    snprintf(exe_path, sizeof(exe_path), "%s\\main.exe", dir);

    remove(exe_path);
    remove(main_path);
    _rmdir(dir);
    if (_mkdir(dir) != 0) {
        fail("failed to create indexed-form temp directory");
    }

    write_file_or_die(
        main_path,
        "form Pair {\n"
        "    left :: i64,\n"
        "    right :: i64,\n"
        "}\n"
        "flow main() -> i64 {\n"
        "    bind pairs :: [2]Pair = [Pair{ left = 1, right = 2 }, Pair{ left = 20, right = 22 }]\n"
        "    bind view :: []Pair = [Pair{ left = 3, right = 4 }, Pair{ left = 30, right = 12 }]\n"
        "    yield pairs[1].left + pairs[1].right + view[1].left + view[1].right\n"
        "}\n");

    int compile_exit = ng_compile_source_file(main_path, exe_path, false);
    expect(compile_exit == 0, "expected indexed form field compile success");

    int run_exit = _spawnl(_P_WAIT, exe_path, exe_path, NULL);
    remove(exe_path);
    remove(main_path);
    _rmdir(dir);
    expect(run_exit == 84, "expected compiled program to return indexed form field result");
}

static void test_compile_with_indexed_form_fields_from_calls(void) {
    char dir[256];
    char main_path[320];
    char exe_path[320];
    int pid = _getpid();

    snprintf(dir, sizeof(dir), "tests\\tmp_indexed_form_call_%d", pid);
    snprintf(main_path, sizeof(main_path), "%s\\main.ng", dir);
    snprintf(exe_path, sizeof(exe_path), "%s\\main.exe", dir);

    remove(exe_path);
    remove(main_path);
    _rmdir(dir);
    if (_mkdir(dir) != 0) {
        fail("failed to create indexed-form-call temp directory");
    }

    write_file_or_die(
        main_path,
        "form Pair {\n"
        "    left :: i64,\n"
        "    right :: i64,\n"
        "}\n"
        "flow make_pairs() -> [2]Pair {\n"
        "    yield [Pair{ left = 1, right = 2 }, Pair{ left = 20, right = 22 }]\n"
        "}\n"
        "flow make_view() -> []Pair {\n"
        "    yield [Pair{ left = 3, right = 4 }, Pair{ left = 30, right = 12 }]\n"
        "}\n"
        "flow main() -> i64 {\n"
        "    yield make_pairs()[1].left + make_pairs()[1].right + make_view()[1].left + make_view()[1].right\n"
        "}\n");

    int compile_exit = ng_compile_source_file(main_path, exe_path, false);
    expect(compile_exit == 0, "expected indexed form field from calls compile success");

    int run_exit = _spawnl(_P_WAIT, exe_path, exe_path, NULL);
    remove(exe_path);
    remove(main_path);
    _rmdir(dir);
    expect(run_exit == 84, "expected compiled program to return indexed form field from calls result");
}

static void test_compile_with_indexed_form_field_assignment(void) {
    char dir[256];
    char main_path[320];
    char exe_path[320];
    int pid = _getpid();

    snprintf(dir, sizeof(dir), "tests\\tmp_indexed_form_assign_%d", pid);
    snprintf(main_path, sizeof(main_path), "%s\\main.ng", dir);
    snprintf(exe_path, sizeof(exe_path), "%s\\main.exe", dir);

    remove(exe_path);
    remove(main_path);
    _rmdir(dir);
    if (_mkdir(dir) != 0) {
        fail("failed to create indexed-form-assign temp directory");
    }

    write_file_or_die(
        main_path,
        "form Inner {\n"
        "    value :: i64,\n"
        "}\n"
        "form Outer {\n"
        "    left :: i64,\n"
        "    inner :: Inner,\n"
        "}\n"
        "flow main() -> i64 {\n"
        "    vary items :: [2]Outer = [Outer{ left = 1, inner = Inner{ value = 2 } }, Outer{ left = 3, inner = Inner{ value = 4 } }]\n"
        "    items[1].left <- 20\n"
        "    items[1].inner.value <- 22\n"
        "    yield items[1].left + items[1].inner.value\n"
        "}\n");

    int compile_exit = ng_compile_source_file(main_path, exe_path, false);
    expect(compile_exit == 0, "expected indexed form field assignment compile success");

    int run_exit = _spawnl(_P_WAIT, exe_path, exe_path, NULL);
    remove(exe_path);
    remove(main_path);
    _rmdir(dir);
    expect(run_exit == 42, "expected compiled program to return indexed form field assignment result");
}

static void test_compile_with_form_init_field_chain(void) {
    char dir[256];
    char main_path[320];
    char exe_path[320];
    int pid = _getpid();

    snprintf(dir, sizeof(dir), "tests\\tmp_form_init_chain_%d", pid);
    snprintf(main_path, sizeof(main_path), "%s\\main.ng", dir);
    snprintf(exe_path, sizeof(exe_path), "%s\\main.exe", dir);

    remove(exe_path);
    remove(main_path);
    _rmdir(dir);
    if (_mkdir(dir) != 0) {
        fail("failed to create form-init-chain temp directory");
    }

    write_file_or_die(
        main_path,
        "form Inner {\n"
        "    val :: i64,\n"
        "}\n"
        "form Outer {\n"
        "    inner :: Inner,\n"
        "}\n"
        "flow main() -> i64 {\n"
        "    yield Outer{ inner = Inner{ val = 42 } }.inner.val\n"
        "}\n");

    int compile_exit = ng_compile_source_file(main_path, exe_path, false);
    expect(compile_exit == 0, "expected form init field chain compile success");

    int run_exit = _spawnl(_P_WAIT, exe_path, exe_path, NULL);
    remove(exe_path);
    remove(main_path);
    _rmdir(dir);
    expect(run_exit == 42, "expected compiled program to return form init field chain result");
}

static void test_compile_with_some_equality(void) {
    char dir[256];
    char main_path[320];
    char exe_path[320];
    int pid = _getpid();

    snprintf(dir, sizeof(dir), "tests\\tmp_some_eq_%d", pid);
    snprintf(main_path, sizeof(main_path), "%s\\main.ng", dir);
    snprintf(exe_path, sizeof(exe_path), "%s\\main.exe", dir);

    remove(exe_path);
    remove(main_path);
    _rmdir(dir);
    if (_mkdir(dir) != 0) {
        fail("failed to create some-equality temp directory");
    }

    write_file_or_die(
        main_path,
        "form Pair {\n"
        "    x :: i64,\n"
        "    y :: i64,\n"
        "}\n"
        "flow main() -> i64 {\n"
        "    check some(42) == some(42) {\n"
        "        check some(Pair{ x = 1, y = 2 }) == some(Pair{ x = 1, y = 2 }) {\n"
        "            yield 1\n"
        "        }\n"
        "    }\n"
        "    yield 0\n"
        "}\n");

    int compile_exit = ng_compile_source_file(main_path, exe_path, false);
    expect(compile_exit == 0, "expected some equality compile success");

    int run_exit = _spawnl(_P_WAIT, exe_path, exe_path, NULL);
    remove(exe_path);
    remove(main_path);
    _rmdir(dir);
    expect(run_exit == 1, "expected compiled program to return some equality result");
}

static void test_compile_with_integer_casts(void) {
    char dir[256];
    char main_path[320];
    char exe_path[320];
    int pid = _getpid();

    snprintf(dir, sizeof(dir), "tests\\tmp_cast_%d", pid);
    snprintf(main_path, sizeof(main_path), "%s\\main.ng", dir);
    snprintf(exe_path, sizeof(exe_path), "%s\\main.exe", dir);

    remove(exe_path);
    remove(main_path);
    _rmdir(dir);
    if (_mkdir(dir) != 0) {
        fail("failed to create integer-cast temp directory");
    }

    write_file_or_die(
        main_path,
        "flow main() -> i64 {\n"
        "    bind x :: i64 = 40\n"
        "    yield (x as i64) + (2 as i64)\n"
        "}\n");

    int compile_exit = ng_compile_source_file(main_path, exe_path, false);
    expect(compile_exit == 0, "expected integer cast compile success");

    int run_exit = _spawnl(_P_WAIT, exe_path, exe_path, NULL);
    remove(exe_path);
    remove(main_path);
    _rmdir(dir);
    expect(run_exit == 42, "expected compiled program to return integer cast result");
}

static void test_compile_with_array_literal_equality_context(void) {
    char dir[256];
    char main_path[320];
    char exe_path[320];
    int pid = _getpid();

    snprintf(dir, sizeof(dir), "tests\\tmp_array_eq_literal_%d", pid);
    snprintf(main_path, sizeof(main_path), "%s\\main.ng", dir);
    snprintf(exe_path, sizeof(exe_path), "%s\\main.exe", dir);

    remove(exe_path);
    remove(main_path);
    _rmdir(dir);
    if (_mkdir(dir) != 0) {
        fail("failed to create array-literal-equality temp directory");
    }

    write_file_or_die(
        main_path,
        "flow main() -> i64 {\n"
        "    bind values :: [2]i64 = [1, 2]\n"
        "    check values == [1, 2] {\n"
        "        yield 1\n"
        "    }\n"
        "    yield 0\n"
        "}\n");

    int compile_exit = ng_compile_source_file(main_path, exe_path, false);
    expect(compile_exit == 0, "expected array literal equality compile success");

    int run_exit = _spawnl(_P_WAIT, exe_path, exe_path, NULL);
    remove(exe_path);
    remove(main_path);
    _rmdir(dir);
    expect(run_exit == 1, "expected compiled program to return array literal equality result");
}

static void test_compile_with_bootstrap_subset_smoke(void) {
    char dir[256];
    char main_path[320];
    char exe_path[320];
    int pid = _getpid();

    snprintf(dir, sizeof(dir), "tests\\tmp_bootstrap_subset_%d", pid);
    snprintf(main_path, sizeof(main_path), "%s\\main.ng", dir);
    snprintf(exe_path, sizeof(exe_path), "%s\\main.exe", dir);

    remove(exe_path);
    remove(main_path);
    _rmdir(dir);
    if (_mkdir(dir) != 0) {
        fail("failed to create bootstrap-subset temp directory");
    }

    write_file_or_die(
        main_path,
        "form Pair {\n"
        "    x :: i64,\n"
        "    y :: i64,\n"
        "}\n"
        "flow make_pair(base :: i64) -> Pair {\n"
        "    yield Pair{ x = base, y = base + 1 }\n"
        "}\n"
        "flow echo_text(value :: text) -> text {\n"
        "    yield value\n"
        "}\n"
        "flow main() -> i64 {\n"
        "    bind values :: [2]i64 = [1, 2]\n"
        "    bind view :: []i64 = [3, 4]\n"
        "    vary pairs :: [2]Pair = [make_pair(10), make_pair(20)]\n"
        "    bind maybe_pair :: maybe[Pair] = some(make_pair(5))\n"
        "    vary total :: i64 = 0\n"
        "    pairs[1].x <- pairs[1].x + 1\n"
        "    check values == [1, 2] {\n"
        "        total <- total + 1\n"
        "    }\n"
        "    check some(42) == some(42) {\n"
        "        total <- total + 2\n"
        "    }\n"
        "    check maybe_pair != none {\n"
        "        total <- total + 4\n"
        "    }\n"
        "    check echo_text(\"ng\") == \"ng\" {\n"
        "        total <- total + 8\n"
        "    }\n"
        "    yield total + pairs[1].x + pairs[1].y + view[0] + view[1] + (14 as i64)\n"
        "}\n");

    int compile_exit = ng_compile_source_file(main_path, exe_path, false);
    expect(compile_exit == 0, "expected bootstrap subset smoke compile success");

    int run_exit = _spawnl(_P_WAIT, exe_path, exe_path, NULL);
    remove(exe_path);
    remove(main_path);
    _rmdir(dir);
    expect(run_exit == 78, "expected compiled program to return bootstrap subset smoke result");
}

static void test_compile_with_text_index_and_each(void) {
    char dir[256];
    char main_path[320];
    char exe_path[320];
    int pid = _getpid();

    snprintf(dir, sizeof(dir), "tests\\tmp_text_each_%d", pid);
    snprintf(main_path, sizeof(main_path), "%s\\main.ng", dir);
    snprintf(exe_path, sizeof(exe_path), "%s\\main.exe", dir);

    remove(exe_path);
    remove(main_path);
    _rmdir(dir);
    if (_mkdir(dir) != 0) {
        fail("failed to create text-index-and-each temp directory");
    }

    write_file_or_die(
        main_path,
        "use text\n"
        "flow main() -> i64 {\n"
        "    bind value :: text = \"ABC\"\n"
        "    yield (text.byte_at(value, 0) as i64) + text.ascii_sum(value)\n"
        "}\n");

    int compile_exit = ng_compile_source_file(main_path, exe_path, false);
    expect(compile_exit == 0, "expected text index and each compile success");

    int run_exit = _spawnl(_P_WAIT, exe_path, exe_path, NULL);
    remove(exe_path);
    remove(main_path);
    _rmdir(dir);
    expect(run_exit == 263, "expected compiled program to return text index and each result");
}

static void test_compile_with_sequence_lengths_and_text_helpers(void) {
    char dir[256];
    char main_path[320];
    char exe_path[320];
    int pid = _getpid();

    snprintf(dir, sizeof(dir), "tests\\tmp_sequence_len_%d", pid);
    snprintf(main_path, sizeof(main_path), "%s\\main.ng", dir);
    snprintf(exe_path, sizeof(exe_path), "%s\\main.exe", dir);

    remove(exe_path);
    remove(main_path);
    _rmdir(dir);
    if (_mkdir(dir) != 0) {
        fail("failed to create sequence-length temp directory");
    }

    write_file_or_die(
        main_path,
        "use text\n"
        "use slice\n"
        "flow main() -> i64 {\n"
        "    bind digit :: text = \"7\"\n"
        "    bind space :: text = \" \"\n"
        "    bind head :: text = \"_\"\n"
        "    bind values :: [3]i64 = [7, 11, 24]\n"
        "    bind view :: []i64 = [1, 2, 39]\n"
        "    vary total :: i64 = text.len(\"abc\") + values.len + view.len\n"
        "    total <- total + slice.sum3_i64(view) + slice.third_i64(view)\n"
        "    check text.is_ascii_digit(digit[0]) {\n"
        "        total <- total + 10\n"
        "    } else {\n"
        "        total <- total + 1\n"
        "    }\n"
        "    check text.is_ascii_space(space[0]) {\n"
        "        total <- total + 20\n"
        "    } else {\n"
        "        total <- total + 2\n"
        "    }\n"
        "    check text.is_ident_head(head[0]) {\n"
        "        total <- total + 30\n"
        "    } else {\n"
        "        total <- total + 3\n"
        "    }\n"
        "    check text.is_ident_tail(digit[0]) {\n"
        "        total <- total + 40\n"
        "    } else {\n"
        "        total <- total + 4\n"
        "    }\n"
        "    yield total\n"
        "}\n");

    int compile_exit = ng_compile_source_file(main_path, exe_path, false);
    expect(compile_exit == 0, "expected sequence length/text helper compile success");

    int run_exit = _spawnl(_P_WAIT, exe_path, exe_path, NULL);
    remove(exe_path);
    remove(main_path);
    _rmdir(dir);
    expect(run_exit == 190, "expected compiled program to return sequence length/text helper result");
}

static void test_compile_bootstrap_lexer_example(void) {
    const char *input_path = "examples\\bootstrap_lexer_demo.ng";
    const char *exe_path = "tests\\generated_bootstrap_lexer_demo.exe";

    remove(exe_path);
    int compile_exit = ng_compile_source_file(input_path, exe_path, false);
    expect(compile_exit == 0, "expected bootstrap lexer example compile success");

    int run_exit = _spawnl(_P_WAIT, exe_path, exe_path, NULL);
    remove(exe_path);
    expect(run_exit == 1777, "expected compiled bootstrap lexer example result");
}

static void test_compile_bootstrap_frontend_example(void) {
    const char *input_path = "examples\\bootstrap_frontend_demo.ng";
    const char *exe_path = "tests\\generated_bootstrap_frontend_demo.exe";

    remove(exe_path);
    int compile_exit = ng_compile_source_file(input_path, exe_path, false);
    expect(compile_exit == 0, "expected bootstrap frontend example compile success");

    int run_exit = _spawnl(_P_WAIT, exe_path, exe_path, NULL);
    remove(exe_path);
    expect(run_exit == 337, "expected compiled bootstrap frontend example result");
}

static void test_compile_bootstrap_compiler_example(void) {
    const char *input_path = "examples\\bootstrap_compiler_demo.ng";
    const char *exe_path = "tests\\generated_bootstrap_compiler_demo.exe";

    remove(exe_path);
    int compile_exit = ng_compile_source_file(input_path, exe_path, false);
    expect(compile_exit == 0, "expected bootstrap compiler example compile success");

    int run_exit = _spawnl(_P_WAIT, exe_path, exe_path, NULL);
    remove(exe_path);
    expect(run_exit == 6145, "expected compiled bootstrap compiler example result");
}

static void test_compile_bootstrap_codegen_example(void) {
    const char *input_path = "examples\\bootstrap_codegen_demo.ng";
    const char *exe_path = "tests\\generated_bootstrap_codegen_demo.exe";

    remove(exe_path);
    int compile_exit = ng_compile_source_file(input_path, exe_path, false);
    expect(compile_exit == 0, "expected bootstrap codegen example compile success");

    int run_exit = _spawnl(_P_WAIT, exe_path, exe_path, NULL);
    remove(exe_path);
    expect(run_exit == 786, "expected compiled bootstrap codegen example result");
}

static void test_compile_bootstrap_codegen_driver_example(void) {
    const char *driver_path = "examples\\bootstrap_codegen_driver.ng";
    const char *exe_path = "tests\\generated_bootstrap_codegen_driver.exe";
    char dir[256];
    char input_path[320];
    char output_path[320];
    char rendered[256];
    size_t rendered_length = 0U;
    int pid = _getpid();

    snprintf(dir, sizeof(dir), "tests\\tmp_bootstrap_codegen_driver_%d", pid);
    snprintf(input_path, sizeof(input_path), "%s\\input.ng", dir);
    snprintf(output_path, sizeof(output_path), "%s\\output.txt", dir);

    remove(output_path);
    remove(input_path);
    remove(exe_path);
    _rmdir(dir);
    if (_mkdir(dir) != 0) {
        fail("failed to create bootstrap-codegen-driver temp directory");
    }

    write_file_or_die(
        input_path,
        "use text\n"
        "flow main() -> i64 {\n"
        "    yield 42\n"
        "}\n");

    expect(ng_compile_source_file(driver_path, exe_path, false) == 0, "expected bootstrap codegen driver compile success");
    expect(_spawnl(_P_WAIT, exe_path, exe_path, input_path, output_path, NULL) == 0,
           "expected bootstrap codegen driver program result");

    read_file_or_die(output_path, rendered, sizeof(rendered), &rendered_length);
    expect(rendered_length == strlen("ok=true\nflows=1\ncode_bytes=104\nrdata_bytes=48\nstack_slots=8\nexit_code=42\n"),
           "expected bootstrap codegen driver output length");
    expect(strcmp(rendered, "ok=true\nflows=1\ncode_bytes=104\nrdata_bytes=48\nstack_slots=8\nexit_code=42\n") == 0,
           "expected bootstrap codegen driver output contents");

    remove(output_path);
    remove(input_path);
    remove(exe_path);
    _rmdir(dir);
}

static void test_compile_with_runtime_file_io(void) {
    char dir[256];
    char main_path[320];
    char exe_path[320];
    char input_path[320];
    char output_path[320];
    char escaped_input[640];
    char escaped_output[640];
    char source[2048];
    char copied[128];
    size_t copied_length = 0U;
    int pid = _getpid();

    snprintf(dir, sizeof(dir), "tests\\tmp_runtime_io_%d", pid);
    snprintf(main_path, sizeof(main_path), "%s\\main.ng", dir);
    snprintf(exe_path, sizeof(exe_path), "%s\\main.exe", dir);
    snprintf(input_path, sizeof(input_path), "%s\\input.txt", dir);
    snprintf(output_path, sizeof(output_path), "%s\\output.txt", dir);

    remove(output_path);
    remove(input_path);
    remove(exe_path);
    remove(main_path);
    _rmdir(dir);
    if (_mkdir(dir) != 0) {
        fail("failed to create runtime-io temp directory");
    }

    write_file_or_die(input_path, "runtime-bridge");
    escape_ng_text_literal(input_path, escaped_input, sizeof(escaped_input));
    escape_ng_text_literal(output_path, escaped_output, sizeof(escaped_output));
    snprintf(
        source,
        sizeof(source),
        "use io\n"
        "flow main() -> i64 {\n"
        "    bind loaded :: ReadTextResult = io.read_text(\"%s\")\n"
        "    check loaded.ok and io.write_text(\"%s\", loaded.value) {\n"
        "        check loaded.value.len == 14 and (loaded.value[0] as i64) == 114 {\n"
        "            yield 42\n"
        "        }\n"
        "    }\n"
        "    yield 0\n"
        "}\n",
        escaped_input,
        escaped_output);
    write_file_or_die(main_path, source);

    expect(ng_compile_source_file(main_path, exe_path, false) == 0, "expected runtime io compile success");
    expect(_spawnl(_P_WAIT, exe_path, exe_path, NULL) == 42, "expected runtime io program result");

    read_file_or_die(output_path, copied, sizeof(copied), &copied_length);
    expect(copied_length == 14U, "expected copied runtime text length");
    expect(strcmp(copied, "runtime-bridge") == 0, "expected runtime text copy contents");

    remove(output_path);
    remove(input_path);
    remove(exe_path);
    remove(main_path);
    _rmdir(dir);
}

static void test_compile_with_runtime_command_line_and_heap(void) {
    char dir[256];
    char main_path[320];
    char exe_path[320];
    char source[2048];
    int pid = _getpid();

    snprintf(dir, sizeof(dir), "tests\\tmp_runtime_cli_%d", pid);
    snprintf(main_path, sizeof(main_path), "%s\\main.ng", dir);
    snprintf(exe_path, sizeof(exe_path), "%s\\main.exe", dir);

    remove(exe_path);
    remove(main_path);
    _rmdir(dir);
    if (_mkdir(dir) != 0) {
        fail("failed to create runtime-cli temp directory");
    }

    snprintf(
        source,
        sizeof(source),
        "use sys\n"
        "use mem\n"
        "flow main() -> i64 !unsafe {\n"
        "    bind command :: text = sys.raw_command_line()\n"
        "    unsafe {\n"
        "        bind ptr :: *u8 = mem.alloc(3)\n"
        "        bind letters :: text = \"ABC\"\n"
        "        ptr[0] <- letters[0]\n"
        "        ptr[1] <- letters[1]\n"
        "        ptr[2] <- letters[2]\n"
        "        bind ready :: bool = command.len > 0 and (ptr[0] as i64) == 65 and (ptr[1] as i64) == 66 and (ptr[2] as i64) == 67\n"
        "        bind freed :: bool = mem.free(ptr)\n"
        "        check ready and freed {\n"
        "            yield 42\n"
        "        }\n"
        "    }\n"
        "    yield 0\n"
        "}\n");
    write_file_or_die(main_path, source);

    expect(ng_compile_source_file(main_path, exe_path, false) == 0, "expected runtime cli compile success");
    expect(_spawnl(_P_WAIT, exe_path, exe_path, "marker", NULL) == 42, "expected runtime cli program result");

    remove(exe_path);
    remove(main_path);
    _rmdir(dir);
}

static void test_compile_with_runtime_text_from_ptr_len(void) {
    char dir[256];
    char main_path[320];
    char exe_path[320];
    char source[2048];
    int pid = _getpid();

    snprintf(dir, sizeof(dir), "tests\\tmp_runtime_text_view_%d", pid);
    snprintf(main_path, sizeof(main_path), "%s\\main.ng", dir);
    snprintf(exe_path, sizeof(exe_path), "%s\\main.exe", dir);

    remove(exe_path);
    remove(main_path);
    _rmdir(dir);
    if (_mkdir(dir) != 0) {
        fail("failed to create runtime-text-view temp directory");
    }

    snprintf(
        source,
        sizeof(source),
        "use mem\n"
        "use sys\n"
        "flow main() -> i64 !unsafe {\n"
        "    unsafe {\n"
        "        bind ptr :: *u8 = mem.alloc(3)\n"
        "        ptr[0] <- \"A\"[0]\n"
        "        ptr[1] <- \"B\"[0]\n"
        "        ptr[2] <- \"C\"[0]\n"
        "        bind rendered :: text = sys.text_from_ptr_len(ptr, 3)\n"
        "        yield (rendered[0] as i64) + (rendered[1] as i64) + (rendered[2] as i64)\n"
        "    }\n"
        "}\n");
    write_file_or_die(main_path, source);

    expect(ng_compile_source_file(main_path, exe_path, false) == 0, "expected runtime text-view compile success");
    expect(_spawnl(_P_WAIT, exe_path, exe_path, NULL) == 198, "expected runtime text-view program result");

    remove(exe_path);
    remove(main_path);
    _rmdir(dir);
}

int main(void) {
    test_compile_with_modules();
    test_compile_with_imported_forms();
    test_compile_with_indexed_form_fields();
    test_compile_with_indexed_form_fields_from_calls();
    test_compile_with_indexed_form_field_assignment();
    test_compile_with_form_init_field_chain();
    test_compile_with_some_equality();
    test_compile_with_integer_casts();
    test_compile_with_array_literal_equality_context();
    test_compile_with_bootstrap_subset_smoke();
    test_compile_with_text_index_and_each();
    test_compile_with_sequence_lengths_and_text_helpers();
    test_compile_bootstrap_lexer_example();
    test_compile_bootstrap_frontend_example();
    test_compile_bootstrap_compiler_example();
    test_compile_bootstrap_codegen_example();
    test_compile_bootstrap_codegen_driver_example();
    test_compile_with_runtime_file_io();
    test_compile_with_runtime_command_line_and_heap();
    test_compile_with_runtime_text_from_ptr_len();
    puts("test_main: ok");
    return 0;
}
