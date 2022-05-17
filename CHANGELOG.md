# Changelog

## 0.3.1
### Added
- `AnyVec::remove`, `AnyVec::remove_into`, `AnyVecTyped::remove`

## 0.3.0
### Changed
- Major API change. All typed operations placed into `AnyVecTyped<T>`.

## 0.2.2
### Optimized
- `swap_remove` further optimized. 
- `swap_take` on par with `Vec::swap_remove` now.

### Fixed
- Fixed UB in `swap_remove` that happens if element destructor panics.

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