use core::ops::{Index, IndexMut};

use alloc::vec::Vec;

use crate::{graphics::Sprite, filesystem::Calculation, rbop_impl::RbopSpriteRenderer, interface::Colour};

/// The sprite cache is an optimization technique which sacrifices memory in order to gain a
/// significant performance boost. Computing and drawing an rbop layout is relatively expensive,
/// so the sprite cache is used to lay out and draw the calculations which we are not editing
/// onto sprites in advance. Calculations not being edited won't change unless we navigate
/// between calculations, so these will be stored until the edited calculation changes. Drawing
/// the sprites onto the screen is significantly faster than recomputing and redrawing the rbop
/// layout.
///
/// Supposing that there are 4 calculations on the screen, one of which is being edited:
///   - Without the sprite cache, every `tick` computes and draws 4 rbop layouts.
///   - With the sprite cache:
///      - The first `tick` after navigating between calculations computes and draws 4 rbop
///        layouts, allocates sprites for them, and performs a pass to mark other calculations
///        as off-screen.
///      - Every subsequent `tick` draws 3 sprites (negligible time) and 1 rbop layout.
#[derive(Clone, Debug)]
pub struct SpriteCache {
    entries: Vec<SpriteCacheEntry>,
}

impl SpriteCache {
    /// Creates an empty sprite cache with no available slots.
    pub fn new() -> Self {
        SpriteCache { entries: Vec::new() }
    }

    /// Completely clears the sprite cache and frees any allocated sprites. Then creates `new_len`
    /// slots, all initialised to `Blank`.
    pub fn clear(&mut self, new_len: usize) {
        // Clear the sprite cache
        self.entries.clear();

        // Fill with "Blank"
        self.entries = Vec::with_capacity(new_len);
        for _ in 0..new_len {
            self.entries.push(SpriteCacheEntry::Blank);
        }
    }

    /// If the given index in the sprite cache is `Blank`, renders a sprite from the provided
    /// `Calculation` list, and saves it into the sprite cache at the index.
    pub fn create_if_blank(&mut self, index: usize, calculations: &mut [Calculation]) {
        if self.entries[index].is_blank() {
            // This entry does not exist
            // Grab calculation
            let root = &mut calculations[index].root;

            // Draw onto sprite, but with:
            //   - No viewport needed since it's not on the screen
            //   - No navpath, so no cursor shows up
            let sprite = RbopSpriteRenderer::draw_to_sprite(
                root,
                None,
                None,
                Colour::BLACK,
            );

            self.entries[index] = SpriteCacheEntry::Entry {
                data: SpriteCacheEntryData::Sprite(sprite),
            }
        }
    }

    /// Retrieves the `SpriteCacheEntryData` for the cached sprite at the given index. If the entry
    /// is clipped, returns `None` instead.
    /// 
    /// Panics if the index is `Blank` or out-of-range.
    pub fn entry_data(&self, index: usize) -> Option<&SpriteCacheEntryData> {
        match &self.entries[index] {
            SpriteCacheEntry::Entry { data } => Some(data),
            SpriteCacheEntry::ClippedOffTop => None,
            SpriteCacheEntry::Blank => panic!("sprite cache miss"),
        }
    }
}

impl Index<usize> for SpriteCache {
    type Output = SpriteCacheEntry;
    fn index(&self, index: usize) -> &Self::Output { &self.entries[index] }
}

impl IndexMut<usize> for SpriteCache {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output { &mut self.entries[index] }
}

#[derive(Clone, Debug)]
/// An entry into the sprite cache.
pub enum SpriteCacheEntry {
    /// The sprite cache has been cleared, and this item hasn't been recomputed yet.
    Blank,

    /// This item was found to be completely off the top of the screen, so has been marked as
    /// clipped. This item does not need to be drawn, and because it is off the top of the screen,
    /// its height does not need to be known for layout calculation.
    ClippedOffTop,

    /// This item has been recomputing since the sprite cache was last cleared, and is either:
    ///   - At least partially visible on the screen, if the wrapped data is `Sprite`
    ///   - Clipped off the bottom of the screen, but therefore has a height required for layout 
    ///     calculation, so the wrapped data is `Height`, without sprite data to save memory
    Entry { data: SpriteCacheEntryData },
}

impl SpriteCacheEntry {
    pub fn is_blank(&self) -> bool {
        matches!(self, SpriteCacheEntry::Blank)
    }
}

#[derive(Clone, Debug)]
pub enum SpriteCacheEntryData {
    Height {
        calculation: u16,
        result: u16,
    },
    Sprite(Sprite),
}
