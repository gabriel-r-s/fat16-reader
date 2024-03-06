mod dir;
mod fat;

fn main() {
    let img_name = std::env::args()
        .skip(1)
        .next()
        .expect("USAGE: fat16-reader /path/to/image");
    let img = std::fs::File::open(img_name).expect("File not found");
    let mut img = fat::Fat16Img::new(img);

    let mut read_first = false;
    let mut read_entries = 0;
    'readdir: loop {
        let entries = img.read_root_dir(read_entries);
        for entry in entries {
            if entry.name()[0] == 0x00 {
                break 'readdir;
            }
            if entry.name()[0] == 0xE5 || entry.attr_lfn() {
                continue;
            }
            dir::ls_entry(entry);
            if entry.attr_archive() {
                let mut cluster = entry.first_cluster();
                let mut size = entry.size();

                if size == 0 || read_first {
                    continue;
                }
                'clusters: loop {
                    let cluster_data = img.read_cluster(cluster);
                    read_first = true;
                    for &byte in cluster_data {
                        if size == 0 {
                            break 'clusters;
                        }
                        if !byte.is_ascii()
                            || (!byte.is_ascii_whitespace() && byte.is_ascii_control())
                        {
                            print!("\\{:02X}", byte);
                        } else {
                            print!("{}", byte as char);
                        }
                        size -= 1;
                    }
                    cluster = img.next_cluster(cluster);
                    if cluster >= 0xFFF8 {
                        break 'clusters;
                    }
                }
            }
            print!("\n\n");
        }
        read_entries += fat::ENTRY_BUF_SIZE as u64;
        if read_entries >= img.root_dir_entries() as u64 {
            break;
        }
    }
}
