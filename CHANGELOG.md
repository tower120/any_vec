# Changelog

## 0.2.1
### Optimized
- All remove operations become faster. Destructor function is not 
called now, if type does not need drop.

### Changed
- `AnyVec` is now `Option`-friendly. (by @SabrinaJewson)
- It is now possible for `AnyVec` to have zero capacity. 
- `AnyVec::new()` now starts with zero capacity.

### Fixed
- Fixed UB with construction with ZST. (by @SabrinaJewson)
- Fixed UB in `clear()` that happens if dropping panics. (by @SabrinaJewson)
- as_mut_slice family now take `&mut self` instead of `&self`. (by @SabrinaJewson)

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