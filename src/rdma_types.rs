use std::{convert::TryInto, clone::Clone, mem::{size_of, forget, MaybeUninit}};

use async_rdma::{LocalMr, LocalMrReadAccess};
use serde::Serialize;
use sha2::{Sha256, Digest};

//use async_rdma::
#[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
#[repr(C)]
pub struct MemChunkSha {
    sha: [u8; 32],
}

impl MemChunkSha {
    pub fn sha(&self) -> &[u8; 32] {
        &self.sha
    }    
}

impl From<[u8; 32]> for MemChunkSha {
    fn from(value: [u8; 32]) -> Self {
        MemChunkSha { sha: value }
    }
}

type M<T: Copy, const N: usize> = [T; N];
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct MemChunk<T: Copy, const N: usize> 
{
    // The memory for the chunk.
    mem: M<T,N>,
    // ID for this chunk.  Used to re-assemble chunks after transfer.
    id: Option<u32>,
    // The sha256 for this chunks memory.
    sha: MemChunkSha,
}

impl <T: Serialize + Clone + Copy, const N: usize> MemChunk<T, N> 
{
    pub fn get_sha(&self) -> &MemChunkSha {
        &self.sha
    }

    pub fn get_id(&self) -> Option<u32> {
        self.id
    }

    pub fn set_id(&mut self, id: u32) {
        self.id = Some(id);
    }

    pub fn check(&self) -> bool {
        let mut hasher = Sha256::new();
        let data = self.mem.clone();
        let data = Vec::from(data);
        let data = bincode::serialize(&data).unwrap();
        hasher.update(data);

        let sha: MemChunkSha = Into::<[u8; 32]>::into(hasher.finalize()).into();
        if sha != self.sha {
            return false;
        }

        true
    }
}

impl <T: Serialize + Copy, const N: usize> TryFrom<LocalMr> for MemChunk<T, N>
{
    type Error = LocalMr;

    fn try_from(value: LocalMr) -> Result<Self, Self::Error> {
        let len = value.as_slice()[..].len();
        let chk = size_of::<MemChunk<T, N>>();

        tracing::debug!("try_from({:?}): {}:{}", value, len, chk);

        if chk != len {
            tracing::error!("Invalid LocalMr: Wrong size: {} != {}", chk, len);
            return Err(value);
        }

        Ok(unsafe { *(*value.as_ptr() as *const MemChunk<T, N>) })
    }
}

//type Thing<ST> = [ST];
// struct Thing<'a, ST>(&'a mut [ST]);

// impl <ST, DT, const N: usize> From<Thing<'_, ST>> for M<DT,N> {
//     fn from(value: Thing<'_, ST>) -> Self {
        
//     }
// }

// impl <ST: Serialize + Default + Copy, DT: Serialize + Default + Copy, const N: usize> TryFrom<Vec<ST>> for MemChunk<DT, N>
// {
//     type Error = Vec<ST>;

//     fn try_from(value: Vec<ST>) -> Result<Self, Self::Error> {
//         // Take the allocated mem from the vec.  
//         // Make sure to extend/contract it to the right size.
//         //value.resize_with(size_of::<M<DT, N>>(), || { ST::default() });
        
//         let cap = value.capacity();
//         let len = value.len();
//         if cap != len {tracing::error!("Vec must be full.")

//         }
//         let mut hasher = Sha256::new();
        
//         hasher.update(bincode::serialize(&value).unwrap());

//         let src_mem_ref = value.leak();
//         let src_mem_len = len * size_of::<ST>();
//         let src_mem_ptr = src_mem_ref.as_mut_ptr();

//         let sha: [u8; 32] = hasher.finalize().into();
        

//         let mut mc = MemChunk {id: None, sha: sha.into(), mem: [DT::default(); N] };

//         let dst_mem_ptr = mc.mem.to_vec();

//         forget(mc.mem);
//         unsafe {
//             let old = dst_mem_ptr;
//             //let dmc.mem.to_vec();
//         }
//         Ok(mc)
//     }
// }

// impl <ST: Serialize + Default + Copy, DT: Serialize + Default + Copy, const N: usize> TryFrom<MemChunk<ST, N>> for Vec<DT> {
//     type Error = MemChunk<ST, N>;

//     fn try_from(value: MemChunk<ST, N>) -> Result<Self, Self::Error> {
//         let mem_ptr = value.mem.as_ptr();
//         let mem_len =  size_of::<M<ST,N>>();
//         // Can't drop it, we just moved the pointer.
//         forget(value.mem);
//         Ok(unsafe {
//             Vec::from_raw_parts(mem_ptr as *mut DT, mem_len, mem_len)
//         }) 
//     }
// }

// impl <T, const N: usize> Into<Vec<u8>> for MemChunk<T, N>
// {
//     fn into(self) -> Vec<u8> {
//         //  Take ownership of self.mem as a pointer to the array.
//         let mem_ptr = self.mem.as_ptr();
        
//         // Get the size in bytes of the array we just got a pointer to.
//         let mem_len = size_of::<M<T,N>>();

//         // Now that we own the memory part, drop everything else to free up the fields not tranferred to the Vec.
//         drop(self);
//         // The below should be safe as we transferred ownership of self.mem to the new vec.
//         // The length and capacity are the same since this is a preallocated array.
//         unsafe { Vec::from_raw_parts(mem_ptr as *mut u8, mem_len, mem_len) }
//     }
// }
