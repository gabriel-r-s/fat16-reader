#[derive(Debug, Clone, Copy)]
#[repr(packed)]
#[repr(C)]
pub struct DirEntry {
    name: [u8; 8],
    ext: [u8; 3],
    attr: u8,
    _reserved: u8,
    creat_hundreds: u8,
    creat_time: u16,
    creat_date: u16,
    touch_date: u16,
    cluster_high: u16,
    modif_time: u16,
    modif_date: u16,
    cluster: u16,
    size: u32,
}

impl DirEntry {
    pub fn new() -> Self {
        Self {
            name: [0; 8],
            ext: [0; 3],
            attr: 0,
            _reserved: 0,
            creat_hundreds: 0,
            creat_time: 0,
            creat_date: 0,
            touch_date: 0,
            cluster_high: 0,
            modif_time: 0,
            modif_date: 0,
            cluster: 0,
            size: 0,
        }
    }

    pub fn name(&self) -> [u8; 8] {
        self.name
    }

    pub fn ext(&self) -> [u8; 3] {
        self.ext
    }

    pub fn attr_readonly(&self) -> bool {
        self.attr & 0x01 != 0
    }

    pub fn attr_hidden(&self) -> bool {
        self.attr & 0x02 != 0
    }

    pub fn attr_system(&self) -> bool {
        self.attr & 0x04 != 0
    }

    pub fn attr_volumeid(&self) -> bool {
        self.attr & 0x08 != 0
    }

    pub fn attr_directory(&self) -> bool {
        self.attr & 0x10 != 0
    }

    pub fn attr_archive(&self) -> bool {
        self.attr & 0x20 != 0
    }

    pub fn attr_lfn(&self) -> bool {
        self.attr & 0x0F == 0x0F
    }

    pub fn first_cluster(&self) -> u16 {
        self.cluster
    }

    pub fn size(&self) -> u32 {
        self.size
    }
}

fn fmt_date(date: u16) {
    let day = date & 0x_1F;
    let month = (date >> 5) & 0xF;
    let year = (date >> 9) & 0x7F;

    // print!("{:02}/{:02}/{:04}", day, month, year);
    print!("{:02}/{:02}/{:04}", day, month, year);
}

fn fmt_time(time: u16) {
    let sec = (time & 0x1F) * 2;
    let min = (time >> 5) & 0x3F;
    let hour = (time >> 11) & 0x1F;

    print!("{:02}:{:02}:{:02}", hour, min, sec);
}

pub fn ls_entry(entry: DirEntry) {
    if entry.attr_lfn() || entry.attr_hidden() {
        return;
    }
    let size = entry.size;

    print!("---- ");

    if entry.attr_directory() {
        print!("DIR 0B\t");
    } else {
        let size = entry.size;
        print!("FILE {}B\t", size);
    }

    print!("FIRST_CLUSTER={:04x} | ", entry.first_cluster());

    for byte in entry.name() {
        if byte.is_ascii_whitespace() {
            break;
        }
        print!("{}", byte as char);
    }
    if !entry.attr_directory() {
        print!(".");
        for byte in entry.ext() {
            if byte.is_ascii_whitespace() {
                break;
            }
            print!("{}", byte as char);
        }
    }
    print!("\n");

    // print!("TOUCH ");
    // fmt_date(entry.touch_date);
    // print!(" ");

    // print!("MOD   ");
    // fmt_date(entry.modif_date);
    // print!(" ");
    // fmt_time(entry.modif_time);
    // print!(" ");

    // print!("CREAT ");
    // fmt_date(entry.creat_date);
    // print!(" ");
    // fmt_time(entry.creat_time);
    // println!();
}
