//! File
//!
//! reference: <https://wiki.osdev.org/FAT#Directories_on_FAT12.2F16.2F32>

use super::*;

#[derive(Debug, Clone)]
pub struct File {
    /// The current offset in the file
    offset: usize,
    /// The current cluster of this file
    current_cluster: Cluster,
    /// DirEntry of this file
    entry: DirEntry,
    /// The file system handle that contains this file
    handle: Fat16Handle,
}

impl File {
    pub fn new(handle: Fat16Handle, entry: DirEntry) -> Self {
        Self {
            offset: 0,
            current_cluster: entry.cluster,
            entry,
            handle,
        }
    }

    pub fn length(&self) -> usize {
        self.entry.size as usize
    }
}

impl Read for File {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        // FIXME: read file content from disk
        //      CAUTION: file length / buffer size / offset
        //
        //      - `self.offset` is the current offset in the file in bytes
        //      - use `self.handle` to read the blocks
        //      - use `self.entry` to get the file's cluster
        //      - use `self.handle.cluster_to_sector` to convert cluster to sector
        //      - update `self.offset` after reading
        //      - update `self.cluster` with FAT if necessary
        let mut total_read = 0;

        // 在文件结尾或者缓冲区已满时停止
        while total_read < buf.len() && self.offset < self.entry.size as usize {
            let sector_size = self.handle.bpb.bytes_per_sector() as usize;
            let cluster_size = sector_size * self.handle.bpb.sectors_per_cluster() as usize;
            let cluster_offset = self.offset % cluster_size;
            let sector_offset = cluster_offset / sector_size;
            let in_sector_offset = self.offset % sector_size;

            // 获取当前读取位置的扇区号
            let sector = self.handle.cluster_to_sector(&self.current_cluster) + sector_offset;

            let mut sector_buf = Block::new(&[0u8; BLOCK_SIZE]);
            self.handle.inner.read_block(sector, &mut sector_buf)?;

            // 计算可以从当前扇区读取的字节数
            let remaining_in_sector = sector_size - in_sector_offset;
            let remaining_in_file = self.entry.size as usize - self.offset;
            let to_read = remaining_in_sector.min(buf.len() - total_read).min(remaining_in_file);

            // 从扇区缓冲区复制数据到输出缓冲区
            buf[total_read..total_read + to_read].copy_from_slice(&sector_buf[in_sector_offset..in_sector_offset + to_read]);

            // 更新偏移和总读取量
            self.offset += to_read;
            total_read += to_read;

            // 如果在簇的末尾，移动到下一个簇
            if cluster_offset + to_read == cluster_size {
                match self.handle.read_next_cluster(self.current_cluster) {
                    Ok(new_cluster) if new_cluster != Cluster::END_OF_FILE => self.current_cluster = new_cluster,
                    _ => break, // 文件结尾或出错
                }
            }
        }

        Ok(total_read)
    }
}

// NOTE: `Seek` trait is not required for this lab
impl Seek for File {
    fn seek(&mut self, _pos: SeekFrom) -> Result<usize> {
        unimplemented!()
    }
}

// NOTE: `Write` trait is not required for this lab
impl Write for File {
    fn write(&mut self, _buf: &[u8]) -> Result<usize> {
        unimplemented!()
    }

    fn flush(&mut self) -> Result<()> {
        unimplemented!()
    }
}
