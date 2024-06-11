use super::*;

impl Fat16Impl {
    pub fn new(inner: impl BlockDevice<Block512>) -> Self {
        let mut block = Block::default();
        let block_size = Block512::size();

        inner.read_block(0, &mut block).unwrap();
        let bpb = Fat16Bpb::new(block.as_ref()).unwrap();

        trace!("Loading Fat16 Volume: {:#?}", bpb);

        // HINT: FirstDataSector = BPB_ResvdSecCnt + (BPB_NumFATs * FATSz) + RootDirSectors;
        let fat_start = bpb.reserved_sector_count() as usize;
        let root_dir_size = { 
            /* FIXME: get the size of root dir from bpb */ 
            let root_dir_entries = bpb.root_entries_count() as usize;
            (root_dir_entries * DirEntry::LEN + block_size - 1) / block_size // 向上取整
        };
        let first_root_dir_sector = { 
            /* FIXME: calculate the first root dir sector */ 
            fat_start + bpb.fat_count() as usize * bpb.sectors_per_fat() as usize
        };
        let first_data_sector = first_root_dir_sector + root_dir_size;

        Self {
            bpb,
            inner: Box::new(inner),
            fat_start,
            first_data_sector,
            first_root_dir_sector,
        }
    }

    pub fn cluster_to_sector(&self, cluster: &Cluster) -> usize {
        match *cluster {
            Cluster::ROOT_DIR => self.first_root_dir_sector,
            Cluster(c) => {
                // FIXME: calculate the first sector of the cluster
                // HINT: FirstSectorofCluster = ((N – 2) * BPB_SecPerClus) + FirstDataSector;
                let cluster_num = c as usize;
                let data_sector = ((cluster_num - 2) * self.bpb.sectors_per_cluster() as usize)
                    + self.first_data_sector;
                data_sector
            }
        }
    }

    // FIXME: YOU NEED TO IMPLEMENT THE FILE SYSTEM OPERATIONS HERE
    //      - read the FAT and get next cluster
    //      - traverse the cluster chain and read the data
    //      - parse the path
    //      - open the root directory
    //      - ...
    //      - finally, implement the FileSystem trait for Fat16 with `self.handle`
    pub fn read_next_cluster(&self, current_cluster: Cluster) -> Result<Cluster> {
        let fat_offset = current_cluster.0 as usize * 2; // 每个 FAT16 表项 2 字节
        let fat_sector = self.fat_start + fat_offset / Block512::size();
        let ent_offset = fat_offset % Block512::size();
    
        let mut block = Block::default();
        self.inner.read_block(fat_sector, &mut block)?;
    
        let next_cluster = u16::from_le_bytes([
            block[ent_offset],
            block[ent_offset + 1],
        ]);
    
        if next_cluster >= 0xFFF8 {
            Err(FsError::EndOfFile) 
        } else {
            Ok(Cluster(next_cluster.into()))
        }
    }

    // 基于目录项名称查找 DirEntry
    fn find_entry_by_name(&self, dir: Option<&Directory>, name: &str) -> Result<DirEntry> {
        let mut cluster = if let Some(dir) = dir {
            dir.cluster
        } else {
            Cluster::ROOT_DIR
        };

        loop {
            let sector = self.cluster_to_sector(&cluster);
            let mut sector_data = Block::new(&[0u8; BLOCK_SIZE]);

            // 读取整个扇区
            for entry_offset in (0..BLOCK_SIZE).step_by(DirEntry::LEN) {
                self.inner.read_block(sector, &mut sector_data)?;

                let entry = &sector_data[entry_offset..entry_offset + DirEntry::LEN];
                // 结束此目录的读取，如果我们遇到了0x00，代表此后不再有有效条目
                if entry[0] == 0x00 {
                    break;
                }
                
                // 如果目录项被删除了，跳过
                if entry[0] == 0xE5 {
                    continue;
                }

                let current_entry = DirEntry::parse(entry)?;
                if current_entry.is_valid() && current_entry.filename() == name {
                    return Ok(current_entry);
                }
            }

            // 检查是否到达了簇链的末尾
            cluster = match self.read_next_cluster(cluster) {
                Ok(next_cluster) if next_cluster != Cluster::END_OF_FILE => next_cluster,
                _ => break,
            };
        }

        Err(FsError::FileNotFound)
    }

    pub fn find_entry(&self, path: &str) -> Result<DirEntry> {
        debug!("Searching for {:?}",path);

        let parts: Vec<String> = path.split('/')
            .filter(|p| !p.is_empty())
            .map(|p| p.to_uppercase()) 
            .collect();
        
        // 开始于根目录 None 表示根目录
        let mut current_dir: Option<Directory> = None;

        for part in parts.iter() {
            // 在当前目录中查找名为 part 的条目
            match self.find_entry_by_name(current_dir.as_ref(), part) {
                Ok(entry) => {
                    if part == parts.last().unwrap() {
                        // 如果这是路径的最后一部分，返回这个目录项
                        return Ok(entry);
                    } else if entry.attributes.contains(Attributes::DIRECTORY) {
                        // 如果找到的是目录，更新 current_dir 以供下一轮查找
                        current_dir = Some(Directory::from_entry(entry));
                    }  else {
                        // 路径中非最后部分的条目不是目录
                        return Err(FsError::NotADirectory);
                    }
                },
                Err(_) => return Err(FsError::FileNotFound),
            }
        }

        // 如果循环结束没有找到，返回错误
        Err(FsError::FileNotFound)
    }

    // 遍历目录，并返回一个包含所有文件元数据的向量
    fn read_dir(&self, dir: Option<&Directory>) -> Result<Vec<Metadata>> {
        let mut cluster = if let Some(dir) = dir {
            dir.cluster
        } else {
            Cluster::ROOT_DIR
        };
        let mut entries = Vec::new();

        loop {
            let sector = self.cluster_to_sector(&cluster);
            let mut sector_data = Block::new(&[0u8; BLOCK_SIZE]);

            // 读取整个扇区
            for entry_offset in (0..BLOCK_SIZE).step_by(DirEntry::LEN) {
                self.inner.read_block(sector, &mut sector_data)?;

                let entry = &sector_data[entry_offset..entry_offset + DirEntry::LEN];
                // 结束此目录的读取，如果我们遇到了0x00，代表此后不再有有效条目
                if entry[0] == 0x00 {
                    break;
                }
                
                // 如果目录项被删除了，跳过
                if entry[0] == 0xE5 {
                    continue;
                }

                let current_entry = DirEntry::parse(entry)?;
                if current_entry.is_valid() && !current_entry.filename().contains("unknow") {
                    entries.push((&current_entry).into());
                }
            }

            // 检查是否到达了簇链的末尾
            cluster = match self.read_next_cluster(cluster) {
                Ok(next_cluster) if next_cluster != Cluster::END_OF_FILE => next_cluster,
                _ => break,
            };
        }

        Ok(entries)
    }
}

impl FileSystem for Fat16 {
    fn read_dir(&self, path: &str) -> Result<Box<dyn Iterator<Item = Metadata> + Send>> {
        // FIXME: read dir and return an iterator for all entries
        let entries = if path.is_empty() {
            self.handle.read_dir(None)?
        } else {
            let entry = self.handle.find_entry(path)?;
            let dir = Directory::from_entry(entry);
            self.handle.read_dir(Some(&dir))?
        };
        let iter = entries.into_iter();
        Ok(Box::new(iter))
    }

    fn open_file(&self, path: &str) -> Result<FileHandle> {
        // FIXME: open file and return a file handle
        let entry = self.handle.find_entry(path)?;
        let meta = (&entry).into();
        let file = File::new(self.handle.clone(), entry);
        Ok(FileHandle::new(meta, Box::new(file)))
    }

    fn metadata(&self, path: &str) -> Result<Metadata> {
        // FIXME: read metadata of the file / dir
        let entry = self.handle.find_entry(path)?;
        Ok((&entry).into())
    }

    fn exists(&self, path: &str) -> Result<bool> {
        // FIXME: check if the file / dir exists
        if let Ok(_) = self.handle.find_entry(path) {
            Ok(true)
        }else {
            Ok(false)
        }
    }
}
