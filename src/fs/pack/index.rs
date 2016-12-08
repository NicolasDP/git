use std::{fmt, convert, io, cmp};
use std::collections::BTreeSet;
use nom;

use ::protocol::Hash;
use ::error::{Result};
use ::fs::util::get_all_files_in;
use ::fs::GitFS;
use super::PackRef;

// magic + version + fanout
const INDEX_HEADER_SIZE : usize = 4 + 4 + 256 * 4;
const INDEX_HASH_OFFSET : usize = INDEX_HEADER_SIZE;

#[derive(Copy)]
pub struct Header {
    magic:   u32,
    version: u32,
    fanouts:  [u32;256]
}
impl Clone for Header {
    fn clone(&self) -> Self { Header { magic: self.magic, version: self.version, fanouts: self.fanouts} }
}
impl PartialEq for Header {
    fn eq(&self, rhs: &Self) -> bool {
        return self.version == rhs.version &&
               self.magic   == self.magic &&
              &self.fanouts[..] == &rhs.fanouts[..]
    }
}
impl fmt::Debug for Header {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Header {{ magic: {:?}, version: {:?}, fanouts: {:?} }}", self.magic, self.version, &self.fanouts[..])
    }

}
impl Eq for Header {}
impl PartialOrd for Header {
    fn partial_cmp(&self, rhs: &Self) -> Option<cmp::Ordering> {
        return (&self.magic, &self.version, &self.fanouts[..]).partial_cmp(&(&rhs.magic, &rhs.version, &rhs.fanouts[..]))
    }
}
impl Ord for Header {
    fn cmp(&self, rhs: &Self) -> cmp::Ordering {
        return (&self.magic, &self.version, &self.fanouts[..]).cmp(&(&rhs.magic, &rhs.version, &rhs.fanouts[..]))
    }
}
impl Header {
    fn new(magic: u32, version: u32, fanouts: [u32;256]) -> Self {
        Header {
            magic:   magic,
            version: version,
            fanouts: fanouts
        }
    }
    pub fn size(&self) -> usize { self.fanouts[255] as usize }

    pub fn numbers_with_prefix(&self, n: u8) -> usize {
        if n == 0u8 {
            self.fanouts[0] as usize
        } else {
            (self.fanouts[n as usize] - self.fanouts[(n as usize) -1]) as usize
        }
    }
    pub fn version(&self) -> u32 { self.version }

    pub fn offsets<H: Hash>(&self) -> (usize, usize, usize) {
        let sz = self.size();
        let hash_table_size = sz * H::digest_size();
        let crc_table_size = 4 * sz;
        let crc_offset = INDEX_HASH_OFFSET + hash_table_size;
        let pack_offset = crc_offset + crc_table_size;
        (INDEX_HASH_OFFSET, crc_offset, pack_offset)
    }
}

named!(nom_parse_index_header_magic<u32>, u32!(nom::Endianness::Big));
named!(nom_parse_index_header_version<u32>, u32!(nom::Endianness::Big));
named!(nom_parse_index_header_fanaout<u32>, u32!(nom::Endianness::Big));
named!(nom_parse_index_header_fanouts<[u32;256]>, count_fixed!(u32, nom_parse_index_header_fanaout, 256));
named!(
    nom_parse_index_header<Header>,
    do_parse!(
        magic:   nom_parse_index_header_magic >>
        version: nom_parse_index_header_version >>
        fanouts: nom_parse_index_header_fanouts >>
        (Header::new(magic, version, fanouts))
    )
);

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
pub struct IndexRef<H: Hash>(H);
impl<H: Hash> IndexRef<H> {
    pub fn new(h: H) -> Self { IndexRef(h) }
}
impl<H: Hash + fmt::Display> fmt::Display for IndexRef<H> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { fmt::Display::fmt(&self.0, f) }
}
impl<H: Hash> Hash for IndexRef<H> {
    fn hash<R: io::BufRead>(data: &mut R) -> io::Result<Self> {
        H::hash(data).map(|h| IndexRef(h))
    }

    fn from_bytes(v: Vec<u8>) -> Option<Self> {
        H::from_bytes(v).map(|h| IndexRef(h))
    }

    #[inline]
    fn digest_size() -> usize { H::digest_size() }

    #[inline]
    fn as_bytes(&self) -> &[u8] { self.0.as_bytes() }
}
impl<H: Hash> convert::AsRef<H> for IndexRef<H> {
    fn as_ref(&self) -> &H { &self.0 }
}

pub fn list_indexes<H: Ord+Hash>(git: &GitFS) -> Result<BTreeSet<IndexRef<H>>> {
    get_all_files_in(
        git.objs_dir().join("pack"),
        & |path| {
            let path_str = format!("{}", path.display());
            if !path_str.starts_with("pack-") || !path_str.ends_with(".idx") {
                Ok(None)
            } else {
                let data = &path_str.as_str()[5..path_str.len()-4];
                Ok(IndexRef::<H>::from_hex(data))
            }
        }
    )
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
pub struct Index<H: Hash> {
    header:   Header,
    hashes:   Vec<H>,
    crcs:     Vec<u32>,
    offsets:  Vec<usize>,
    pack:     PackRef<H>,
    index:    IndexRef<H>
}
impl<H: Hash> Index<H> {
    fn new( header: Header
          , hashes: Vec<H>
          , crcs: Vec<u32>
          , offsets: Vec<usize>
          , pack: PackRef<H>
          , index: IndexRef<H>
          ) -> Self
    {
        Index {
            header:  header,
            hashes:  hashes,
            crcs:    crcs,
            offsets: offsets,
            pack:    pack,
            index:   index
        }
    }
}

pub fn parse_index<H:Hash>(i: &[u8]) -> nom::IResult<&[u8], Index<H>> {
    let (i, header)  = try_parse!(i, nom_parse_index_header);
    let (i, hashes)  = try_parse!(i, count!(H::decode_bytes, header.size()));
    let (i, crcs)    = try_parse!(i, count!(u32!(nom::Endianness::Big), header.size()));
    let (mut i, mut offsets) = try_parse!(i, count!(map!(u32!(nom::Endianness::Big), |v| v as usize), header.size()));
    for offset in offsets.iter_mut() {
        let u = *offset as u32;
        if (u & 0x80000000) != 0 {
            let (i_, large_offset) = try_parse!(i, map!(u64!(nom::Endianness::Big), |v| v as usize));
            *offset = large_offset;
            i = i_;
        }
    }
    let (i, pack) = try_parse!(i, PackRef::<H>::decode_bytes);
    let (i, index) = try_parse!(i, IndexRef::<H>::decode_bytes);
    nom::IResult::Done(i, Index::new(header, hashes, crcs, offsets, pack, index))
}
