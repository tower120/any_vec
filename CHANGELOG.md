# Changelog

## 0.2.0
### Added
- `AnyVec::with_capacity` added.
- push benchmark added.
### Changed
- `AnyVec::element_size` removed, `AnyVec::element_layout` added instead.
### Optimized
- `push` family performance improved.
### Fixed
- Fixed Zero-Size-Type memory allocation.

## 0.1.0
### Added
- Initial working implementation with `push`, `swap_remove` and `as_slice`.