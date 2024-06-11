//! ATA Drive
//!
//! reference: https://wiki.osdev.org/IDE
//! reference: https://wiki.osdev.org/ATA_PIO_Mode
//! reference: https://github.com/theseus-os/Theseus/blob/HEAD/kernel/ata/src/lib.rs

mod bus;
mod consts;

use alloc::{boxed::Box, string::String};
use bus::AtaBus;
use consts::AtaDeviceType;
use spin::Mutex;
use crate::alloc::borrow::ToOwned;

lazy_static! {
    pub static ref BUSES: [Mutex<AtaBus>; 2] = {
        let buses = [
            Mutex::new(AtaBus::new(0, 14, 0x1F0, 0x3F6)),
            Mutex::new(AtaBus::new(1, 15, 0x170, 0x376)),
        ];

        info!("Initialized ATA Buses.");

        buses
    };
}

#[derive(Clone)]
pub struct AtaDrive {
    pub bus: u8,
    pub drive: u8,
    blocks: u32,
    model: Box<str>,
    serial: Box<str>,
}

impl AtaDrive {
    pub fn open(bus: u8, drive: u8) -> Option<Self> {
        trace!("Opening drive {}@{}...", bus, drive);

        // we only support PATA drives
        if let Ok(AtaDeviceType::Pata(res)) = BUSES[bus as usize].lock().identify_drive(drive) {
            let buf = res.map(u16::to_be_bytes).concat();
            let serial = { 
                /* FIXME: get the serial from buf */
                let data = &buf[10 * 2..20 * 2];
                String::from_utf8_lossy(&data).trim().to_owned().into_boxed_str()
            };
            let model = { 
                /* FIXME: get the model from buf */ 
                let data = &buf[27 * 2..47 * 2];
                String::from_utf8_lossy(&data).trim().to_owned().replace(" ", "").into_boxed_str()
            };
            let blocks = { 
                /* FIXME: get the block count from buf */ 
                let low = u16::from_be_bytes([buf[60 * 2], buf[60 * 2 + 1]]);
                let high = u16::from_be_bytes([buf[61 * 2], buf[61 * 2 + 1]]);
                u32::from(low) + (u32::from(high) << 16)
            };
            let ata_drive = Self {
                bus,
                drive,
                model,
                serial,
                blocks,
            };
            info!("Drive {} opened", ata_drive);
            Some(ata_drive)
        } else {
            warn!("Drive {}@{} is not a PATA drive", bus, drive);
            None
        }
    }

    fn humanized_size(&self) -> (f64, &'static str) {
        let size = self.block_size();
        let count = self.block_count().unwrap();
        let bytes = size * count;

        crate::humanized_size(bytes as u64)
    }
}

impl core::fmt::Display for AtaDrive {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let (size, unit) = self.humanized_size();
        write!(f, "{} {} ({} {})", self.model, self.serial, size, unit)
    }
}

use storage::{Block512, BlockDevice, FsError};

impl BlockDevice<Block512> for AtaDrive {
    fn block_count(&self) -> storage::Result<usize> {
        // FIXME: return the block count
        Ok(self.blocks as usize)
    }

    fn read_block(&self, offset: usize, block: &mut Block512) -> storage::Result<()> {
        // FIXME: read the block
        //      - use `BUSES` and `self` to get bus
        //      - use `read_pio` to get data
        if offset >= self.blocks as usize {
            return Err(FsError::NotInSector);
        }

        let buffer: &mut [u8] = block.as_mut();
        let result = BUSES[self.bus as usize]
            .lock()
            .read_pio(self.drive, offset as u32, buffer);

        result.map_err(|e| e.into())
    }

    fn write_block(&self, offset: usize, block: &Block512) -> storage::Result<()> {
        // FIXME: write the block
        //      - use `BUSES` and `self` to get bus
        //      - use `write_pio` to write data
        if offset >= self.blocks as usize {
            return Err(FsError::NotInSector);
        }

        let buffer: &[u8] = block.as_ref();
        let result = BUSES[self.bus as usize]
            .lock()
            .write_pio(self.drive, offset as u32, buffer);

        result.map_err(|e| e.into())
    }
}
