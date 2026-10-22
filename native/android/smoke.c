/* Runs exactly the same exported C API as Dart FFI, useful before installing Flutter. */
#include <dlfcn.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

static void *open_library(const char *name) {
    void *library = dlopen(name, RTLD_NOW | RTLD_GLOBAL);
    if (!library) {
        char directory[4096], path[8192];
        if (getcwd(directory, sizeof(directory))) {
            snprintf(path, sizeof(path), "%s/%s", directory, name);
            library = dlopen(path, RTLD_NOW | RTLD_GLOBAL);
        }
    }
    if (!library) { fprintf(stderr, "%s: %s\n", name, dlerror()); exit(2); }
    return library;
}

static void *symbol(void *library, const char *name) {
    void *value = dlsym(library, name);
    if (!value) { fprintf(stderr, "Missing export %s: %s\n", name, dlerror()); exit(2); }
    return value;
}

int main(int argc, char **argv) {
    if (argc != 2) { fprintf(stderr, "Usage: smoke writable-output-directory\n"); return 2; }
    void *skia = open_library("libquestpdf_skia.so");
    void *qpdf = open_library("libquestpdf_qpdf.so");
    int (*skia_version)(void) = symbol(skia, "questpdf_skia_get_compatibility_version");
    int (*qpdf_version)(void) = symbol(qpdf, "questpdf_get_compatibility_version");
    if (skia_version() != 26 || qpdf_version() != 4) { fprintf(stderr, "Native ABI mismatch\n"); return 2; }
    void *bridge = open_library("libquestpdf_android.so");
    int (*version)(void) = symbol(bridge, "questpdf_abi_version");
    int (*generate)(int32_t, const char *, uint8_t **, int32_t *) = symbol(bridge, "questpdf_generate");
    int (*tests)(const char *, uint8_t **, int32_t *) = symbol(bridge, "questpdf_run_tests");
    void (*release)(void *) = symbol(bridge, "questpdf_free");
    if (version() != 1) return 2;
    if (generate(0, argv[1], NULL, NULL) != 2) return 3;
    uint8_t *output = NULL;
    int32_t length = 0;
    if (generate(999, argv[1], &output, &length) != 1 || !output || length <= 0) return 3;
    release(output);
    release(NULL);
    output = NULL;
    length = 0;
    int result = tests(argv[1], &output, &length);
    if (output && length > 0) fwrite(output, 1, length, stdout);
    putchar('\n');
    release(output);
    return result;
}
