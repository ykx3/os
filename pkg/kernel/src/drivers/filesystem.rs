use super::ata::*;
use alloc::boxed::Box;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
// use chrono::DateTime;
use storage::fat16::Fat16;
use storage::mbr::*;
use storage::*;

pub static ROOTFS: spin::Once<Mount> = spin::Once::new();

pub fn get_rootfs() -> &'static Mount {
    ROOTFS.get().unwrap()
}

pub fn init() {
    info!("Opening disk device...");

    let drive = AtaDrive::open(0, 0).expect("Failed to open disk device");

    // only get the first partition
    let part = MbrTable::parse(drive)
        .expect("Failed to parse MBR")
        .partitions()
        .expect("Failed to get partitions")
        .remove(0);

    info!("Mounting filesystem...");

    ROOTFS.call_once(|| Mount::new(Box::new(Fat16::new(part)), "/".into()));

    trace!("Root filesystem: {:#?}", ROOTFS.get().unwrap());

    info!("Initialized Filesystem.");
}

pub fn ls(root_path: &str) {
    println!("{:12} {:12} {:20}", "Name", "Size", "Last Modified");
    let iter = match get_rootfs().read_dir(root_path) {
        Ok(iter) => iter,
        Err(err) => {
            warn!("{:?}", err);
            return;
        }
    };

    // FIXME: format and print the file metadata
    //      - use `for meta in iter` to iterate over the entries
    //      - use `crate::humanized_size_short` for file size
    //      - add '/' to the end of directory names
    //      - format the date as you like
    //      - do not forget to print the table header
    for meta in iter {
        let name = if meta.entry_type == FileType::Directory {
            format!("{}/", meta.name)
        } else {
            meta.name
        };

        let size = crate::humanized_size(meta.len.try_into().unwrap());
        let size = format!("{:.2} {}", size.0, size.1);

        let modified = meta.modified.map_or(String::from("Unknown"), |datetime| {
            datetime.format("%Y-%m-%d %H:%M:%S").to_string()
        });

        println!("{:12} {:12} {:20}", name, size, modified);
    }
}

pub fn cat(path: &str) {
    let fs = get_rootfs();
    let f = fs.open_file(path);

    match f {
        Ok(mut file) => {
            let mut buf = Vec::new(); 
            
            let _ = file.read_all(&mut buf);

            // 将读取到的数据（字节）转换为字符串
            match String::from_utf8(buf) {
                Ok(str) => print!("{}", str),   
                Err(e) => println!("Error converting to UTF-8: {:?}", e), 
            }
        },
        Err(e) => println!("Error opening file: {:?}", e), // 无法打开文件
    }
}