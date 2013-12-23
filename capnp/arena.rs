/*
 * Copyright (c) 2013, David Renshaw (dwrenshaw@gmail.com)
 *
 * See the LICENSE file in the capnproto-rust root directory.
 */

use std;
use common::*;
use common::ptr_sub;
use message;

pub type SegmentId = u32;

pub struct SegmentReader<'a> {
    arena : ArenaPtr<'a>,
    ptr : * Word,
    size : WordCount
}

impl <'a> SegmentReader<'a> {

    pub unsafe fn get_start_ptr(&self) -> *Word {
        self.ptr.offset(0)
    }

    pub unsafe fn contains_interval(&self, from : *Word, to : *Word) -> bool {
        let fromAddr : uint = std::cast::transmute(from);
        let toAddr : uint = std::cast::transmute(to);
        let thisBegin : uint = std::cast::transmute(self.ptr);
        let thisEnd : uint = std::cast::transmute(self.ptr.offset(self.size as int));
        return (fromAddr >= thisBegin && toAddr <= thisEnd);
        // TODO readLimiter
    }
}

pub struct SegmentBuilder<'a> {
    reader : SegmentReader<'a>,
    id : SegmentId,
    pos : *mut Word,
}

impl <'a> SegmentBuilder<'a> {

    pub fn new<'b>(arena : *mut BuilderArena<'b>,
                   id : SegmentId,
                   ptr : *mut Word,
                   size : WordCount) -> SegmentBuilder<'b> {
        SegmentBuilder {
            reader : SegmentReader {
                arena : BuilderArenaPtr(arena),
                ptr : unsafe {std::cast::transmute(ptr)},
                size : size
            },
            id : id,
            pos : ptr,
        }
    }

    pub fn get_word_offset_to(&mut self, ptr : *mut Word) -> WordCount {
        let thisAddr : uint = self.reader.ptr.to_uint();
        let ptrAddr : uint = ptr.to_uint();
        assert!(ptrAddr >= thisAddr);
        let result = (ptrAddr - thisAddr) / BYTES_PER_WORD;
        return result;
    }

    pub fn current_size(&self) -> WordCount {
        ptr_sub(self.pos, self.reader.ptr)
    }

    pub fn allocate(&mut self, amount : WordCount) -> Option<*mut Word> {
        if (amount > self.reader.size - self.current_size()) {
            return None;
        } else {
            let result = self.pos;
            self.pos = unsafe { self.pos.offset(amount as int) };
            return Some(result);
        }
    }

    pub fn available(&self) -> WordCount {
        self.reader.size - ptr_sub(self.pos, self.reader.ptr)
    }

    #[inline]
    pub unsafe fn get_ptr_unchecked(&mut self, offset : WordCount) -> *mut Word {
        std::cast::transmute_mut_unsafe(self.reader.ptr.offset(offset as int))
    }

    #[inline]
    pub fn get_segment_id(&self) -> SegmentId { self.id }

    pub fn get_arena(&self) -> *mut BuilderArena<'a> {
        match self.reader.arena {
            BuilderArenaPtr(b) => b,
            _ => fail!()
        }
    }
}

// ----------------
// The following stuff is currently unused.

pub struct ReaderArena<'a> {
//    message : *message::MessageReader<'a>,
    segment0 : SegmentReader<'a>,

    more_segments : Option<~[SegmentReader<'a>]>
    //XXX should this be a map as in capnproto-c++?
}

pub struct BuilderArena<'a> {
    message : *mut message::MessageBuilder<'a>,
    segment0 : SegmentBuilder<'a>,
    more_segments : Option<~[~SegmentBuilder<'a>]>,
}

impl <'a> BuilderArena<'a> {

    #[inline]
    pub fn allocate(&mut self, amount : WordCount) -> (*mut SegmentBuilder<'a>, *mut Word) {
        unsafe {
            match self.segment0.allocate(amount) {
                Some(result) => { return (std::ptr::to_mut_unsafe_ptr(&mut self.segment0), result) }
                None => {}
            }

            match self.more_segments {
                Some(_) => {}
                None() => {}
            }

            fail!()
        }
    }

    pub fn get_segment(&mut self, id : SegmentId) -> *mut SegmentBuilder<'a> {
        if (id == 0) {
            std::ptr::to_mut_unsafe_ptr(&mut self.segment0)
        } else {
            fail!()
        }
    }


    pub fn get_segments_for_output<T>(&self, cont : |&[&[Word]]| -> T) -> T {
        unsafe {
            match self.more_segments {
                None => {
                    std::vec::raw::buf_as_slice::<Word, T>(
                        self.segment0.reader.ptr,
                        self.segment0.current_size(),
                        |v| cont([v]) )
                }
                Some(_) => {
                    fail!()
                }
            }
        }
    }
}

pub enum ArenaPtr<'a> {
    ReaderArenaPtr(*ReaderArena<'a>),
    BuilderArenaPtr(*mut BuilderArena<'a>),
    Null
}

impl <'a> ArenaPtr<'a>  {
    pub fn try_get_segment(&self, id : SegmentId) -> *SegmentReader<'a> {
        unsafe {
            match self {
                &ReaderArenaPtr(reader) => {
                    if (id == 0) {
                        return std::ptr::to_unsafe_ptr(&(*reader).segment0);
                    } else {
                        match (*reader).more_segments {
                            None => {fail!("no segments!")}
                            Some(ref segs) => {
                                unsafe {segs.unsafe_ref(id as uint - 1)}
                            }
                        }
                    }
                }
                &BuilderArenaPtr(builder) => {
                    if (id == 0) {
                        std::ptr::to_unsafe_ptr(&(*builder).segment0.reader)
                    } else {
                        match (*builder).more_segments {
                            None => {fail!("no more segments!")}
                            Some(ref segs) => {
                               std::ptr::to_unsafe_ptr(&segs[id as uint - 1].reader)
                            }
                        }
                    }
                }
                &Null => {
                    fail!()
                }
            }
        }
    }
}
