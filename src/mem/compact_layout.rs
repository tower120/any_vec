use std::alloc::Layout;
use std::num::NonZeroUsize;

const ALIGN_BITS: u32 = 6;
/// ALIGN_BITS bits 0, all other 1
const MASK: usize = usize::MAX << ALIGN_BITS;

/// Use 6 bits for power-of-two align exponent (2^6 = 64).
/// All other - size.
///
/// Since size is used most often, we put it closer to most significant
/// part, so it can be extracted with one bit shift operation.
#[derive(Copy, Clone)]
pub struct CompactLayout{
    packed: usize
}

#[inline]
pub fn pack(layout: &Layout) -> CompactLayout{
    // align is guaranteed to be power-of-two and non null
    // ilog2 very fast
    let align_pot_exponent = unsafe{
        NonZeroUsize::new_unchecked(layout.align())
        .ilog2() as usize
    };

    let mut packed = layout.size() << ALIGN_BITS;
    packed |= align_pot_exponent;
    CompactLayout{ packed }
}

#[inline]
pub fn unpack(this: CompactLayout) -> Layout{
    let align_pot_exponent = this.packed & !MASK;
    let align = 1 << align_pot_exponent;

    let size = this.packed >> ALIGN_BITS;
    unsafe{
        Layout::from_size_align_unchecked(size, align)
    }
}

#[cfg(test)]
mod test{
    use super::*;

    #[test]
    fn regression_test1(){
        let align = 1;
        let size  = 1;

        let layout = unsafe{
            Layout::from_size_align_unchecked(size, align)
        };

        let packed   = pack(&layout);
        let unpacked = unpack(packed);

        assert_eq!(unpacked.size(),  size);
        assert_eq!(unpacked.align(), align);
    }    

    #[test]
    fn fuzzy_test(){
        for align_pot_exponent in 0..64{
            let align = 1<<align_pot_exponent;
            // TODO: RANDOM
            for size in 0..(u16::MAX as usize - u8::MAX as usize){
                let layout = unsafe{
                    Layout::from_size_align_unchecked(size, align)
                };

                let packed   = pack(&layout);
                let unpacked = unpack(packed);

                assert_eq!(unpacked.size(),  size);
                assert_eq!(unpacked.align(), align);
            }
        }
    }
}