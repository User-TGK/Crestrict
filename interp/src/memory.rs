use std::collections::{BTreeSet, HashMap};

use crate::restrict::Base;

/// Created for block identifiers (b in CompCert, SymBase in K-semantics)
pub type AllocId = u64;

/// Pointer offset
pub type Offset = i32;

/// A simple location without provenance.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct SimpleLoc {
    pub block: AllocId,
    offset: Offset,
}

impl SimpleLoc {
    pub fn new(block: AllocId, offset: Offset) -> Self {
        Self { block, offset }
    }
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct Loc {
    loc: SimpleLoc,
    prov: BTreeSet<Base>,
}

impl Loc {
    pub fn new(block: AllocId, offset: Offset) -> Self {
        Self {
            loc: SimpleLoc::new(block, offset),
            prov: BTreeSet::new(),
        }
    }

    pub fn add_prov(&mut self, base: Base) {
        self.prov.insert(base);
    }

    pub fn strip_prov(&self) -> SimpleLoc {
        self.loc.clone()
    }

    pub fn get_bases(&self) -> &BTreeSet<Base> {
        &self.prov
    }

    pub fn add(&self, rhs: i32) -> Self {
        Self {
            loc: SimpleLoc::new(self.loc.block, self.loc.offset + rhs),
            prov: self.prov.clone(),
        }
    }

    pub fn block(&self) -> u64 {
        self.loc.block
    }

    pub fn offset(&self) -> i32 {
        self.loc.offset
    }

    pub fn simple(&self) -> SimpleLoc {
        self.loc
    }

    pub fn remove_inactive_bases(&mut self, active_scopes: &HashMap<i32, bool>) {
        self.prov
            .retain(|&(_, scope)| active_scopes[&scope] == true);
    }
}

// TODO: values could become 'abstract bytes' or something else
// TODO: note that offsets are not checked for alignment and are casted from i32 to u32
use crate::ast::Val;

// (CompCert model I/II) A block is associated with either
//      - A global variable of the program
//      - An addressable local variable of every active invocation of a function of the program
//      - Invocation of malloc

#[derive(Clone, Debug)]
pub struct Block {
    // The values contained in the block
    pub values: HashMap<u32, Val>,

    // Note: this is CompCert's block representation, unsure yet if
    // we need this.

    // Valid offsets of a block range between low_bound and high_bound
    low_bound: i32,      // inclusive, always 0.
    pub high_bound: i32, // exclusive

    // True if this block was created by an invocation of Malloc in the
    // program (CompCert EF_malloc)
    dynamically_allocated: bool,
}

impl Block {
    pub fn new() -> Self {
        Self {
            values: HashMap::from([(0, Val::Undef)]),
            low_bound: 0,
            high_bound: 1,
            dynamically_allocated: false,
        }
    }

    pub fn is_dynamically_allocated(mut self) -> Self {
        self.dynamically_allocated = true;

        self
    }

    pub fn with_high_bound(mut self, high_bound: i32) -> Self {
        self.high_bound = high_bound;
        self
    }

    // pub fn is_valid(&self) -> bool {
    //     self.low_bound != self.high_bound
    // }
}

// Memory representation, inspired by the CompCert memory model.
// The block/offset layout is useful if our language will allow pointer
// arithmetic, which will probably be essential.
//
// Support for offsets within blocks is prepared but not yet used
// for this iteration. The memory API should eventually account for this
// (whenever pointer arithmetic is added to the language). Also, valid offsets
// should then be checked.
//

#[derive(Clone, Debug)]
pub struct Memory {
    contents: HashMap<AllocId, Block>,
    alloc_id_counter: AllocId,
}

impl Memory {
    // Create a new memory instance
    pub fn new() -> Self {
        Self {
            contents: HashMap::new(),
            alloc_id_counter: 0,
        }
    }

    // Allocation :: Memory -> hi -> Memory x Block
    pub fn alloc(&mut self, high_bound: i32) -> AllocId {
        let id = self.new_alloc_id();
        self.contents
            .insert(id, Block::new().with_high_bound(high_bound));

        id
    }

    // Allocation :: Memory -> Size -> Memory x Block
    pub fn malloc(&mut self, size: i32) -> AllocId {
        let id = self.new_alloc_id();
        let mut block = Block::new()
            .is_dynamically_allocated()
            .with_high_bound(size);

        for i in 1..(size - 1) as u32 {
            block.values.insert(i, Val::Undef);
            block.high_bound = size
        }

        self.contents.insert(id, block);

        id
    }

    // Free :: Memory -> Block -> Memory
    // Note that this does not release the block identifier.
    pub fn free(&mut self, block_id: AllocId) {
        let block = self.contents.get_mut(&block_id).expect(&format!(
            "Attempted to free an unknown block '{}'.",
            block_id
        ));

        block.low_bound = 0;
        block.high_bound = 0;

        log::debug!("Freed block {}", block_id);
    }

    // Free :: Memory -> Block -> Memory
    // Free a block which was dynamically allocated by the user (with malloc).
    // Note that this does not release the block identifier.
    pub fn manual_free(&mut self, block_id: AllocId) -> Result<(), String> {
        let block = self
            .contents
            .get_mut(&block_id)
            .unwrap_or_else(|| panic!("Attempted to free an unknown block '{}'.", block_id));

        if block.dynamically_allocated {
            block.low_bound = 0;
            block.high_bound = 0;
            Ok(())
        } else {
            Err(format!(
                "Program tried to free block {} which was not created by a call to malloc",
                block_id
            ))
        }
    }

    // Load :: Memory -> Block -> Option Val
    pub fn load(&self, block_id: AllocId, offset: Offset) -> Result<Val, String> {
        if let Some(block) = self.contents.get(&block_id) {
            let v = block.values.get(&(offset as u32)).cloned();

            if offset < block.low_bound || offset >= block.high_bound {
                Err(format!(
                    "UB: attempted to read unaccessable bytes in block {} at offset {} outside range {}-{}",
                    block_id,
                    offset, block.low_bound, block.high_bound
                ))
            } else if let Some(Val::Undef) = v {
                Err(format!(
                    "UB: uninitialized memory read from block {} at offset {}",
                    block_id, offset
                ))
            } else {
                log::trace!(
                    "Read value {:?} from block {} at offset {}",
                    v,
                    block_id,
                    offset
                );

                v.ok_or(format!(
                    "Unallocated cell in block {} at {}. The block is {:?}",
                    block_id, offset, block
                ))
            }
        } else {
            Err(String::from("Unallocated block"))
        }
    }

    // Store :: Memory -> Block -> Val -> Option Mem
    pub fn store(&mut self, block_id: AllocId, offset: Offset, value: Val) -> Result<(), String> {
        if let Some(block) = self.contents.get_mut(&block_id) {
            if offset < block.low_bound || offset >= block.high_bound {
                Err(format!(
                    "UB: attempted to store to unaccessable bytes in block {} at offset {}",
                    block_id, offset
                ))
            } else {
                log::trace!(
                    "Stored value {:?} in block {} at offset {}",
                    value,
                    block_id,
                    offset
                );

                block.values.insert(offset as u32, value);

                log::trace!("{:#?}", self.contents);

                Ok(())
            }
        } else {
            Err(String::from("Attempted to store in an unknown block."))
        }
    }

    // Create a new allocation identifier
    fn new_alloc_id(&mut self) -> u64 {
        let id = self.alloc_id_counter;
        self.alloc_id_counter += 1;

        id
    }
}
