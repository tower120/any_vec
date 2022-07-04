# Changelog

## Unreleased
### Added
- `Debug` implemented for `AnyVec`, `AnyVecTyped`.

### Fixed
- Stacked Borrow friendly now.

## 0.9.1
### Changed
- `impls` dependency dropped. 
- minimum rust version 1.61 requirement dropped.

### Fixed
- Empty `HeapMem` used wrong aligned dangling pointer. Fixed. 
- `AnyVec` Send/Sync -ability now MemBuilder dependent. 
- Fixed UB with `StackMem::as_ptr`.

## 0.9.0
### Added
- `MemBuilder` + `Mem` = Allocator.
- `Stack` Mem/Allocator.
- `AnyVec::clone_empty_in`
- `reserve`
- `reserve_exact`
- `shrink_to_fit`
- `shrink_to`
- `pop`
- `is_empty`

## 0.8.0
### Added
- Added `AnyVec::at` - ergonomic version of `get`.
- `AnyVecRef` now cloneable.
- `ElementRef` now cloneable.
- non-consuming iterators now cloneable.
- `AnyVec::drain`.
- `AnyVecTyped::drain`.
- `AnyVec::slice`.
- `AnyVecTyped::slice`.
- `AnyVec` iterators.
- `AnyVec::clone_empty`, to construct `AnyVec` of the same type.
- `IntoIterator` implemented.

### Changed
- `crate::refs` being implementation details, hided.

## 0.7.0
### Added
- `AnyValueClonable` and `LazyClone` added.
- `AnyVec` getters added.
- `AnyValueMut` added. All remove operations now return `AnyValueMut` + `AnyValueClonable`. 

### Changed
- `any_value::AnyValue::downcast<T>` now return `Option<T>`.
- `traits::EmptyTrait` renamed to `traits::None`.
- `AnyValue` interface changed.

### Optimized
- Performance of all remove operations slightly increased.

## 0.6.0
### Added
- `AnyVec` now can be Sync, Send, Clone. 

### Changed
- `any_value::AnyValueTemp` moved to `ops::AnyValueTemp`
- `any_vec::Unknown` -> `any_vec::any_value::Unknown` 

### Fixed
- `AnyVec::insert` now check type again.
- `AnyValueWrapper::downcast` UB fx.

## 0.5.0
### Changed
- Major type erased API change. All `AnyVec` operations now safe.

### Added
- Introduced `any_value` family, which provide type erased safe operations.

## 0.4.0
### Added
- `AnyVec::remove` 
- `AnyVec::remove_into` 
- `AnyVecTyped::remove`
- `AnyVec::insert_uninit`
- `AnyVecTyped::insert`

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