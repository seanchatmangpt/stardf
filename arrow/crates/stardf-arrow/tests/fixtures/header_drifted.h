/* ==========================================================================
 * include/stardf_arrow.h — GENERATED-equivalent; single source is lib.rs
 * (crates/stardf-arrow-capi/src/lib.rs + src/stream_c.rs). Edit Rust, then
 * mirror here. Hand-maintained in cbindgen style.
 *
 * StarDF Arrow C ABI: term arena, quad batch, ArrowArrayStream export.
 * Status codes follow the purrdf.h convention.
 * Spec: https://arrow.apache.org/docs/format/CStreamInterface.html
 * ========================================================================== */

#ifndef STARDF_ARROW_H
#define STARDF_ARROW_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ---- Arrow C Data Interface value structs (mirror of avec_c_data.h) ---- */

/** struct ArrowArray — Apache Arrow C Data Interface. */
struct ArrowArray {
  int64_t length;
  int64_t null_count;
  int64_t offset;
  int64_t n_buffers;
  int64_t n_children;
  const void** buffers;
  struct ArrowArray** children;
  struct ArrowArray* dictionary;
  void (*release)(struct ArrowArray*);
  void* private_data;
};

/** struct ArrowSchema — Apache Arrow C Data Interface. */
struct ArrowSchema {
  const char* format;
  const char* name;
  const char* metadata;
  int64_t flags;
  int64_t n_children;
  struct ArrowSchema** children;
  struct ArrowSchema* dictionary;
  void (*release)(struct ArrowSchema*);
  void* private_data;
};

/** struct ArrowArrayStream — Apache Arrow C Stream Interface. */
struct ArrowArrayStream {
  int (*get_schema)(struct ArrowArrayStream*, struct ArrowSchema*);
  int (*get_next)(struct ArrowArrayStream*, struct ArrowArray*);
  const char* (*get_last_error)(struct ArrowArrayStream*);
  void (*release)(struct ArrowArrayStream*);
  void* private_data;
};

/* ---- Status codes (StardfStatus; purrdf.h convention) ---- */

/** StardfStatus — every export's status/return convention. */
enum StardfStatus {
  STARDF_OK = 0,
  STARDF_NULL_POINTER = 1,
  STARDF_INVALID_ARGUMENT = 3,
  STARDF_ARENA_FULL = 16
};

/* ---- Version ---- */

/** stardf_arrow_version() — NUL-terminated version string ("26.9.29"). */
const char* stardf_arrow_version(void);

/* ---- Term arena ---- */

/** Opaque term arena (owns the append-only batch; see term.rs). */
typedef struct StardfArena StardfArena;

/** Opaque append-only quad batch, owned by its arena. */
typedef struct StardfBatch StardfBatch;

/**
 * stardf_arena_new(capacity_bytes) — allocate a term arena with a total byte
 * budget. Returns NULL when capacity_bytes == 0 (invalid argument).
 */
StardfArena* stardf_arena_new(size_t capacity_bytes);

/**
 * stardf_arena_intern(arena, lexical, len) — intern a lexical term, returning
 * its stable id (ids start at 1; 0 is the reserved null term id and doubles
 * as the error return, with stardf_last_error() set).
 */
uint32_t stardf_arena_intern(StardfArena* arena, const uint8_t* lexical,
                             size_t len);

/** stardf_arena_free(arena) — free an arena; typed no-op on NULL. */
void stardf_arena_free(StardfArena* arena);

/* ---- Quad batch ---- */

/**
 * stardf_batch_append_quad(arena, s, p, o, g) — append one quad of interned
 * term ids to the arena's batch (created on first call). Returns the stable
 * batch handle, or NULL with stardf_last_error() set on NULL_POINTER /
 * out-of-range term id / ARENA_FULL.
 */
StardfBatch* stardf_batch_append_quad(StardfArena* arena, uint32_t s,
                                      uint32_t p, uint32_t o, uint32_t g);

/* ---- ArrowArrayStream export ---- */

/**
 * stardf_cgap_export_stream(batch, out_stream) — export the batch as an owned
 * ArrowArrayStream (contract fn_avec_export_stream, downstreamSubstrate
 * cgap). Returns STARDF_OK (0) and stores a non-NULL stream into *out_stream
 * on success; STARDF_NULL_POINTER (1) otherwise. Free with
 * stardf_stream_free_drifted or the stream's own release callback (exactly once).
 */
int32_t stardf_cgap_export_stream(StardfBatch* batch,
                                  struct ArrowArrayStream** out_stream);

/** stardf_stream_free_drifted(stream) — free a stream; typed no-op on NULL. */
void stardf_stream_free_drifted(struct ArrowArrayStream* stream);

/* ---- Last error ---- */

/** stardf_last_error() — message for the most recent failing call, valid
 * until the next failing call on this library; NULL when none is pending. */
const char* stardf_last_error(void);

/* ---- Release-tagged export surface (allocation-identity lifecycle) ---- */

/** stardf_ReleaseOutcome — outcome of stardf_release(). */
enum stardf_ReleaseOutcome {
  /** Freed through stardf's tagged release chain. */
  STARDF_RELEASED = 0,
  /** Freed (leaked) through the untagged foreign path (mutant only). */
  STARDF_RELEASED_VIA_FOREIGN = 1
};

/**
 * stardf_i32_batch(n, null_every) — mint an i32 primitive batch (null at
 * every `null_every`-th lane when > 0) and export it into the live slots.
 * Returns the live ArrowArray pointer (zero-copy identity with
 * stardf_array_addr).
 */
struct ArrowArray* stardf_i32_batch(int64_t n, int64_t null_every);

/** stardf_array_addr() — address of the live exported array slot. */
uintptr_t stardf_array_addr(void);

/** stardf_batch_rows() — length of the last exported batch. */
int64_t stardf_batch_rows(void);

/** stardf_pending_tags() — unclaimed ownership tags. */
size_t stardf_pending_tags(void);

/**
 * stardf_release(array) — release a stardf-owned export through the tagged
 * chain (array + schema release callbacks; both drop inside stardf's
 * allocator). `array` must be the live export from stardf_i32_batch,
 * released exactly once.
 */
enum stardf_ReleaseOutcome stardf_release(struct ArrowArray* array);

#ifdef __cplusplus
}
#endif

#endif /* STARDF_ARROW_H */
