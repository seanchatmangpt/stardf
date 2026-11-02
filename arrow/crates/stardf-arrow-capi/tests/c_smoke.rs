// SPDX-License-Identifier: MIT OR Apache-2.0

//! C smoke court for the StarDF Arrow C ABI (`include/stardf_arrow.h`).
//!
//! Chicago-school: a real C program, really compiled with `cc`, really linked
//! against the freshly built `libstardf_arrow_capi` shared library, really run
//! — no mocks, no stubs. It dlopens nothing it cannot reach: linkage is at
//! link time, load time via `DYLD_LIBRARY_PATH`/`LD_LIBRARY_PATH`. The C
//! source lives in a scratch file under `/tmp`, written by this test and
//! deleted again when the test finishes — the only durable artifact of this
//! court is this Rust file.
//!
//! Drives: `stardf_arrow_version`, `stardf_arena_new`, `stardf_arena_intern`
//! (2 terms), `stardf_batch_append_quad` (3 quads), `stardf_avec_export_stream`,
//! then the `ArrowArrayStream` vtable (`get_schema` / `get_next` / `release`),
//! asserting: schema format non-null, exactly one batch of 3 rows, then EOF,
//! and a clean release.

#![cfg(not(miri))]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The C program under court. Scratch only: written to a `/tmp` directory at
/// test time, removed with the directory at teardown. It includes the
/// committed header and links the real cdylib, so the contract it exercises is
/// the header's, byte for byte.
const SMOKE_C: &str = r##"
#include <assert.h>
#include <stdio.h>
#include <string.h>
#include "stardf_arrow.h"

static size_t rows_in_array(const struct ArrowArray* a) {
  /* Flat string / utf8 arrays: length IS the row count. */
  return (size_t)a->length;
}

int main(void) {
  /* 1. version: non-empty, NUL-terminated. */
  const char* version = stardf_arrow_version();
  assert(version != NULL);
  assert(version[0] != '\0');
  printf("version=%s\n", version);

  /* 2. arena: capacity 0 must be refused (documented NULL return). */
  assert(stardf_arena_new(0) == NULL);

  StardfArena* arena = stardf_arena_new(1 << 20);
  assert(arena != NULL);

  /* 3. intern two terms; ids start at 1, 0 is the null/error id. */
  uint32_t a = stardf_arena_intern(arena, (const uint8_t*)"subject-1", 9);
  uint32_t b = stardf_arena_intern(arena, (const uint8_t*)"predicate-1", 11);
  assert(a == 1);
  assert(b == 2);
  /* Re-interning is stable (interning, not appending). */
  assert(stardf_arena_intern(arena, (const uint8_t*)"subject-1", 9) == a);

  /* 4. append 3 quads over the two interned terms. */
  StardfBatch* batch = NULL;
  for (uint32_t i = 0; i < 3; ++i) {
    batch = stardf_batch_append_quad(arena, a, b, a, b);
    assert(batch != NULL);
  }

  /* 5. export the ArrowArrayStream. */
  struct ArrowArrayStream* stream = NULL;
  int32_t rc = stardf_avec_export_stream(batch, &stream);
  assert(rc == STARDF_OK);
  assert(stream != NULL);

  /* 6. schema: non-null format through the real vtable. */
  struct ArrowSchema schema;
  memset(&schema, 0, sizeof(schema));
  assert(stream->get_schema(stream, &schema) == 0);
  assert(schema.format != NULL);
  printf("schema.format=%s n_children=%lld\n", schema.format,
         (long long)schema.n_children);
  if (schema.release) schema.release(&schema);

  /* 7. exactly one batch of 3 rows, then EOF. */
  int batches = 0;
  size_t total_rows = 0;
  for (;;) {
    struct ArrowArray array;
    memset(&array, 0, sizeof(array));
    int gr = stream->get_next(stream, &array);
    assert(gr == 0);
    if (array.release == NULL) {
      printf("EOF after %d batch(es)\n", batches);
      break; /* EOF: release callback NULL means end of stream. */
    }
    ++batches;
    total_rows += rows_in_array(&array);
    array.release(&array);
  }
  assert(batches == 1);
  assert(total_rows == 3);

  /* 8. clean release: vtable release, exactly once, then NULL. */
  stream->release(stream);
  printf("C SMOKE OK\n");
  return 0;
}
"##;

fn dylib_name() -> String {
    format!(
        "{}stardf_arrow_capi{}",
        std::env::consts::DLL_PREFIX,
        std::env::consts::DLL_SUFFIX
    )
}

/// Locate the freshly built cdylib. `cargo test` does not build a cdylib as a
/// dependency of this test binary, so the test builds it itself, then verifies
/// the artifact exists — existence alone is not trusted, it is produced here.
fn build_dylib(root: &Path) -> PathBuf {
    let lib = root.join("target").join("debug").join(dylib_name());
    let status = Command::new(cargo())
        .args(["build", "-p", "stardf-arrow-capi"])
        .current_dir(root)
        .status()
        .expect("spawn cargo build -p stardf-arrow-capi");
    assert!(status.success(), "cargo build -p stardf-arrow-capi failed");
    assert!(
        lib.exists(),
        "{} not found at {} after building",
        dylib_name(),
        lib.display()
    );
    lib
}

fn cargo() -> String {
    std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string())
}

#[test]
fn c_abi_smoke() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest.parent().unwrap().parent().unwrap().to_path_buf();
    let header_dir = manifest.join("include");
    let cc = std::env::var("CC").unwrap_or_else(|_| "cc".to_string());

    let dylib = build_dylib(&root);

    // Scratch build dir under /tmp, removed unconditionally at the end.
    let scratch = std::env::temp_dir().join(format!("stardf-arrow-c-smoke-{}", std::process::id()));
    let _ = fs::remove_dir_all(&scratch);
    fs::create_dir_all(&scratch).expect("create scratch dir");
    let smoke_c = scratch.join("smoke.c");
    fs::write(&smoke_c, SMOKE_C).expect("write smoke.c");
    let bin = scratch.join("stardf_arrow_c_smoke");

    let compiled = Command::new(&cc)
        .arg(&smoke_c)
        .arg("-std=c11")
        .arg(format!("-I{}", header_dir.display()))
        .arg(&dylib)
        .arg("-o")
        .arg(&bin)
        .status()
        .expect("spawn cc");
    assert!(compiled.success(), "cc compile/link failed");

    // Loader search path is platform-specific.
    let lib_dir = dylib.parent().unwrap();
    let loader_var = if cfg!(target_os = "macos") {
        "DYLD_LIBRARY_PATH"
    } else if cfg!(target_os = "windows") {
        "PATH"
    } else {
        "LD_LIBRARY_PATH"
    };

    let run = Command::new(&bin)
        .env(loader_var, lib_dir)
        .output()
        .expect("run C smoke binary");
    let stdout = String::from_utf8_lossy(&run.stdout);
    let stderr = String::from_utf8_lossy(&run.stderr);

    // Teardown first so the scratch never outlives an assertion failure.
    let _ = fs::remove_dir_all(&scratch);

    assert!(
        run.status.success(),
        "C smoke binary failed:\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stdout.contains("C SMOKE OK"),
        "missing OK marker:\n{stdout}"
    );
    assert!(
        !stdout.contains("version=\n") && stdout.contains("version="),
        "version line missing:\n{stdout}"
    );
    assert!(
        stdout.contains("EOF after 1 batch(es)"),
        "expected exactly one batch then EOF:\n{stdout}"
    );
    // The scratch dir is gone — the court leaves no residue.
    assert!(
        !scratch.exists(),
        "scratch dir survived: {}",
        scratch.display()
    );
}
