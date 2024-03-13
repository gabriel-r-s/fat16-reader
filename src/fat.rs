use crate::dir::DirEntry;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

// struct para desserialização
#[derive(Debug)]
#[repr(packed)]
#[repr(C)]
pub struct BootRecord {
    _jmp: [u8; 3], // EB 3C 90 = JMP SHORT 3C NOP
    _oem: [u8; 8], // mkdosfs\0
    bytes_per_sector: u16,
    sectors_per_cluster: u8,
    reserved_sectors: u16,
    fats: u8,
    root_dir_entries: u16,
    total_sectors: u16,
    _media_descriptor_type: u8,
    sectors_per_fat: u16,
    _sectors_per_track: u16,
    _heads_sides: u16,
    _hidden_sectors: u32,
    _large_sectors: u32,
}

// struct para armazenar apenas dados relevantes
#[derive(Debug)]
pub struct Fat16Img {
    img: File,
    cluster_size: usize,
    offset_fat1: u64,
    offset_root: u64,
    offset_data: u64,
    root_dir_entries: u16,
    buf: Vec<u8>,
}

pub const ENTRY_BUF_SIZE: usize = 8;

impl Fat16Img {
    pub fn new(mut img: File) -> Self {
        let mut buf = vec![0_u8; 2048];
        img.read_exact(&mut buf[..std::mem::size_of::<BootRecord>()])
            .expect("Bad image");
        let br = unsafe { buf[0..].as_ptr().cast::<BootRecord>().read() };
        let offset_fat1 = br.reserved_sectors;
        let fat_count = br.fats as u16;
        let offset_root = br.reserved_sectors + fat_count * br.sectors_per_fat;
        let root_dir_sectors = (br.root_dir_entries * 32) / br.bytes_per_sector;
        let offset_data =
            br.reserved_sectors + (br.fats as u16 * br.sectors_per_fat + root_dir_sectors);

        let bytes_per_sector = br.bytes_per_sector as u64;
        let sectors_per_cluster = br.sectors_per_cluster as u64;
        let cluster_size = bytes_per_sector * sectors_per_cluster;

        let reserved_sectors = br.reserved_sectors;
        let root_dir_entries = br.root_dir_entries;
        let sectors_per_fat = br.sectors_per_fat;

        println!("**METADADOS IMAGEM FAT**");
        println!("BYTES POR SETOR: {bytes_per_sector}");
        println!("SETORES RESERVADOS: {reserved_sectors}",);
        println!("SETORES POR CLUSTER: {sectors_per_cluster}");
        println!("NUMERO DE FATs: {fat_count}");
        println!("SETORES POR FATs: {sectors_per_fat}");
        println!("ENTRADAS DO DIRETÓRIO RAIZ: {root_dir_entries}",);

        buf.resize(cluster_size as usize, 0);

        Self {
            img,
            cluster_size: (bytes_per_sector * sectors_per_cluster) as usize,
            offset_fat1: u64::from(offset_fat1) * u64::from(bytes_per_sector),
            offset_root: u64::from(offset_root) * u64::from(bytes_per_sector),
            offset_data: u64::from(offset_data) * u64::from(bytes_per_sector),
            root_dir_entries: u16::from(br.root_dir_entries),
            buf,
        }
    }

    pub fn root_dir_entries(&self) -> u16 {
        self.root_dir_entries
    }

    // acessar FAT para encontrar o cluster seguinte
    pub fn next_cluster(&mut self, cluster: u16) -> u16 {
        let cluster = cluster as u64;
        self.img
            .seek(SeekFrom::Start(self.offset_fat1 + cluster * 2))
            .expect("Bad image");
        let mut next = 0_u16;
        let next_buf = unsafe {
            std::slice::from_raw_parts_mut(
                std::ptr::addr_of_mut!(next).cast::<u8>(),
                std::mem::size_of::<u16>(),
            )
        };
        self.img.read_exact(next_buf).expect("Bad image");
        next
    }

    // ler diretorio em `offset[start]`
    pub fn read_dir(&mut self, offset: u64, start: u64) -> [DirEntry; ENTRY_BUF_SIZE] {
        let mut entries = [DirEntry::new(); ENTRY_BUF_SIZE];
        let offset = offset + start * (std::mem::size_of::<DirEntry>() as u64);
        self.img.seek(SeekFrom::Start(offset)).expect("Bad image");
        let buf = unsafe {
            std::slice::from_raw_parts_mut(
                entries[..].as_mut_ptr().cast::<u8>(),
                std::mem::size_of::<DirEntry>() * ENTRY_BUF_SIZE,
            )
        };
        self.img.read_exact(buf).expect("Bad image");
        entries
    }

    pub fn read_root_dir(&mut self, start: u64) -> [DirEntry; ENTRY_BUF_SIZE] {
        self.read_dir(self.offset_root, start)
    }

    pub fn read_cluster(&mut self, cluster: u16) -> &[u8] {
        let offset = (cluster - 2) as u64 * self.cluster_size as u64;
        self.img
            .seek(SeekFrom::Start(self.offset_data + offset))
            .expect("Bad image");
        self.img
            .read_exact(&mut self.buf[..self.cluster_size])
            .expect("Bad image");
        &self.buf[..self.cluster_size]
    }
}
