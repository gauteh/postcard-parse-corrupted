use std::path::Path;
use half::f16;

pub const SAMPLE_SZ: usize = 3;
pub const AXL_SZ: usize = SAMPLE_SZ * 1024;

/// Max size of `AxlPacket` serialized using postcard with COBS. Set with some margin since
/// postcard messages are not fixed size.
pub const AXL_POSTCARD_SZ: usize = 1024 * 8;

#[derive(serde::Serialize, serde::Deserialize, PartialEq)]
pub struct AxlPacket {
    /// Timestamp of sample at `offset` in ms.
    pub timestamp: i64,

    /// Offset in IMU FIFO at time of timestamp.
    pub offset: u16,

    /// ID on SD-card. This one is not necessarily unique. Will not be set
    /// before package has been written to SD-card.
    pub storage_id: Option<u32>,

    /// Time of position in seconds.
    pub position_time: u32,
    pub lon: f64,
    pub lat: f64,

    /// Frequency of data.
    pub freq: f32,

    /// IMU data. This is moved to the payload when transmitting.
    pub data: heapless::Vec<f16, { AXL_SZ }>,
}

impl core::fmt::Debug for AxlPacket {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::write!(fmt, "AxlPacket(timestamp: {}, offset: {}, storage_id: {:?}, position_time: {}, lon: {}, lat: {}, freq: {}, data (length): {}))",
            self.timestamp,
            self.offset,
            self.storage_id,
            self.position_time,
            self.lon,
            self.lat,
            self.freq,
            self.data.len()
            )
    }
}

#[derive(Debug)]
struct Collection {
    pub pcks: Vec<AxlPacket>,
}

impl Collection {
    pub fn from_file(p: impl AsRef<Path>) -> anyhow::Result<Collection> {
        let p = p.as_ref();
        let mut b = std::fs::read(p)?;

        if (b.len() % AXL_POSTCARD_SZ) != 0 {
            eprintln!("Warning, collection consists of non-integer number of packages.");
        }

        let n = b.len() / AXL_POSTCARD_SZ;

        eprintln!(
            "Parsing {} bytes of packages into {} packages..",
            b.len(),
            n
        );
        let pcks = b
            .chunks_exact_mut(AXL_POSTCARD_SZ)
            .filter_map(|p| match postcard::from_bytes_cobs(p) {
                Ok(p) => Some(p),
                Err(e) => {
                    eprintln!("failed to parse package: {:?}", e);
                    None
                }
            })
            .collect::<Vec<_>>();

        Ok(Collection { pcks })
    }
}

fn main() {
    println!("Hello, world!");

    eprintln!("Parsing correct file..");
    let c0 = Collection::from_file("98.1").unwrap();
    for p in &c0.pcks {
        eprintln!("{p:?}");
    }

    eprintln!("Parsing partially corrupted file..");
    let c1 = Collection::from_file("27.1").unwrap();
    for p in &c1.pcks {
        eprintln!("{p:?}");
    }
}
