## Code Quality

* Long or complex functions must include detailed Japanese rustdoc comments.
* Source files should stay around 1000 lines or less. Split files into smaller, cohesive modules before they grow beyond that.
* Keep documentation files focused and reasonably sized. Split large documents by topic rather than allowing a single document to grow indefinitely.
* Do not add `clone()`, `Arc`, `Mutex`, `Box`, or `dyn Trait` solely to silence ownership or borrowing errors.
* Do not discard errors with `let _ =`, `.ok()`, or empty match arms unless intentionally documented.
* Encode domain concepts with enums, structs, and newtypes rather than raw strings, integers, or boolean flags.
* Use private visibility by default, `pub(crate)` for crate-internal APIs, and `pub` only for deliberate external APIs.

## Document

See [docs/README.md](docs/README.md)
