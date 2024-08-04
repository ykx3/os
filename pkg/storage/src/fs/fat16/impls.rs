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
    fn find_entry_by_name(&self, dir: &Directory, name: &str) -> Result<DirEntry> {
        let mut entries = Vec::new();

        self.iterate_dir(&dir, |entry| {
            if entry.filename() == name {
                entries.push(entry.clone());
            }
        })?;

        match entries.pop() {
            Some(entry) => Ok(entry),
            _ => Err(FsError::FileNotFound)
        }
    }

    pub fn find_entry(&self, path: &str) -> Result<DirEntry> {
        debug!("Searching for {:?}",path);

        let parts: Vec<String> = path.split('/')
            .filter(|p| !p.is_empty())
            .map(|p| p.to_uppercase()) 
            .collect();
        
        // 开始于根目录 None 表示根目录
        let mut current_dir = Directory::root();

        for part in parts.iter() {
            // 在当前目录中查找名为 part 的条目
            match self.find_entry_by_name(&current_dir, part) {
                Ok(entry) => {
                    if part == parts.last().unwrap() {
                        // 如果这是路径的最后一部分，返回这个目录项
                        return Ok(entry);
                    } else if entry.attributes.contains(Attributes::DIRECTORY) {
                        // 如果找到的是目录，更新 current_dir 以供下一轮查找
                        current_dir = Directory::from_entry(entry);
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

    pub fn iterate_dir<F>(&self, dir: &directory::Directory, mut func: F) -> Result<()>
    where
        F: FnMut(&DirEntry),
    {
        if let Some(entry) = &dir.entry {
            trace!("Iterating directory: {}", entry.filename());
        }

        let mut current_cluster = Some(dir.cluster);
        let mut dir_sector_num = self.cluster_to_sector(&dir.cluster);
        let dir_size = match dir.cluster {
            Cluster::ROOT_DIR => self.first_data_sector - self.first_root_dir_sector,
            _ => self.bpb.sectors_per_cluster() as usize,
        };
        trace!("Directory size: {}", dir_size);

        let mut block = Block::default();
        let block_size = Block512::size();
        while let Some(cluster) = current_cluster {
            for sector in dir_sector_num..dir_sector_num + dir_size {
                self.inner.read_block(sector, &mut block).unwrap();
                for entry in 0..block_size / DirEntry::LEN {
                    let start = entry * DirEntry::LEN;
                    let end = (entry + 1) * DirEntry::LEN;

                    let dir_entry = DirEntry::parse(&block[start..end])?;

                    if dir_entry.filename.is_eod() {
                        return Ok(());
                    } else if dir_entry.is_valid() && !dir_entry.is_long_name() {
                        func(&dir_entry);
                    }
                }
            }
            current_cluster = if cluster != Cluster::ROOT_DIR {
                match self.read_next_cluster(cluster) {
                    Ok(n) => {
                        dir_sector_num = self.cluster_to_sector(&n);
                        Some(n)
                    }
                    _ => None,
                }
            } else {
                None
            }
        }
        Ok(())
    }

    fn read_dir(&self, dir: &Directory) -> Result<Vec<Metadata>> {
        let mut entries = Vec::new();

        self.iterate_dir(&dir, |entry| {
            entries.push(entry.as_meta());
        })?;
        
        Ok(entries)
    }
}

impl FileSystem for Fat16 {
    fn read_dir(&self, path: &str) -> Result<Box<dyn Iterator<Item = Metadata> + Send>> {
        // FIXME: read dir and return an iterator for all entries
        let entries = if path.is_empty() {
            self.handle.read_dir(&Directory::root())?
        } else {
            let entry = self.handle.find_entry(path)?;
            let dir = Directory::from_entry(entry);
            self.handle.read_dir(&dir)?
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
