use std::{
    fs::File,
    io::{BufReader, ErrorKind, Read},
};

use types::{OsencExtentRecordPayload, OsencRecordBase, OsencServerstatRecordPayload};

mod s57;
mod types;

const HEADER_SENC_VERSION: u16 = 1;
const HEADER_CELL_NAME: u16 = 2;
const HEADER_CELL_PUBLISHDATE: u16 = 3;
const HEADER_CELL_EDITION: u16 = 4;
const HEADER_CELL_UPDATEDATE: u16 = 5;
const HEADER_CELL_UPDATE: u16 = 6;
const HEADER_CELL_NATIVESCALE: u16 = 7;
const HEADER_CELL_SENCCREATEDATE: u16 = 8;
const HEADER_CELL_SOUNDINGDATUM: u16 = 9;

const FEATURE_ID_RECORD: u16 = 64;
const FEATURE_ATTRIBUTE_RECORD: u16 = 65;

const FEATURE_GEOMETRY_RECORD_POINT: u16 = 80;
const FEATURE_GEOMETRY_RECORD_LINE: u16 = 81;
const FEATURE_GEOMETRY_RECORD_AREA: u16 = 82;
const FEATURE_GEOMETRY_RECORD_MULTIPOINT: u16 = 83;
const FEATURE_GEOMETRY_RECORD_AREA_EXT: u16 = 84;

const VECTOR_EDGE_NODE_TABLE_EXT_RECORD: u16 = 85;
const VECTOR_CONNECTED_NODE_TABLE_EXT_RECORD: u16 = 86;

const VECTOR_EDGE_NODE_TABLE_RECORD: u16 = 96;
const VECTOR_CONNECTED_NODE_TABLE_RECORD: u16 = 97;

const CELL_COVR_RECORD: u16 = 98;
const CELL_NOCOVR_RECORD: u16 = 99;
const CELL_EXTENT_RECORD: u16 = 100;
const CELL_TXTDSC_INFO_FILE_RECORD: u16 = 101;

const SERVER_STATUS_RECORD: u16 = 200;

fn main() {
    let result = parse_file();
    if let Ok(res) = result {
        println!("succesfully read file");
    } else {
        println!("{}", result.err().unwrap());
    }
}

fn parse_file() -> std::io::Result<()> {
    let file = File::open("/home/silas/Downloads/exported/OC-49-M11SO4.oesu")?;
    let mut reader = BufReader::new(file);

    loop {
        let mut buf = [0u8; std::mem::size_of::<OsencRecordBase>()];
        if reader.read_exact(&mut buf).is_err() {
            break;
        }

        let record_base: OsencRecordBase = unsafe { std::mem::transmute(buf) };

        match record_base.get_record_type() {
            0 => {
                // EOF
                break;
            }
            SERVER_STATUS_RECORD => {
                if record_base.get_record_len() >= 20 {
                    return Err(std::io::Error::new(
                        ErrorKind::Other,
                        "Failed to parse header",
                    ));
                }

                let buf_size =
                    record_base.get_record_len() as usize - std::mem::size_of::<OsencRecordBase>();

                assert_eq!(
                    buf_size,
                    std::mem::size_of::<OsencServerstatRecordPayload>()
                );
                let mut buf = [0u8; std::mem::size_of::<OsencServerstatRecordPayload>()];

                reader.read_exact(&mut buf)?;

                let serverstat_record: OsencServerstatRecordPayload =
                    unsafe { std::mem::transmute(buf) };

                if serverstat_record.get_expire_status() == 0 {
                    return Err(std::io::Error::new(ErrorKind::Other, "Chart expired"));
                }

                if serverstat_record.get_decrypt_status() == 0 {
                    return Err(std::io::Error::new(ErrorKind::Other, "Signature failure"));
                }
            }
            HEADER_SENC_VERSION => {
                if record_base.get_record_len() < 6 || record_base.get_record_len() >= 16 {
                    return Err(std::io::Error::new(
                        ErrorKind::Other,
                        "Failed to parse header",
                    ));
                }

                let buf_size =
                    record_base.get_record_len() as usize - std::mem::size_of::<OsencRecordBase>();

                assert_eq!(buf_size, std::mem::size_of::<u16>());
                let mut buf = [0u8; std::mem::size_of::<u16>()];

                reader.read_exact(&mut buf)?;

                let version: u16 = unsafe { std::mem::transmute(buf) };

                if version < 201 {
                    return Err(std::io::Error::new(ErrorKind::Other, "Unsupported Version"));
                }
            }
            HEADER_CELL_NAME => {
                let buf_size =
                    record_base.get_record_len() as usize - std::mem::size_of::<OsencRecordBase>();

                let mut buf = vec![0u8; buf_size];

                reader.read_exact(&mut buf)?;

                if let Ok(cell_name) = String::from_utf8(buf) {
                    println!("cell_name: {}", cell_name);
                }
            }

            HEADER_CELL_PUBLISHDATE => {
                let buf_size =
                    record_base.get_record_len() as usize - std::mem::size_of::<OsencRecordBase>();

                let mut buf = vec![0u8; buf_size];

                reader.read_exact(&mut buf)?;

                if let Ok(cell_publishdate) = String::from_utf8(buf) {
                    println!("cell_publishdate: {}", cell_publishdate);
                }
            }
            HEADER_CELL_EDITION => {
                let buf_size =
                    record_base.get_record_len() as usize - std::mem::size_of::<OsencRecordBase>();

                assert_eq!(buf_size, std::mem::size_of::<u16>());

                let mut buf = [0u8; std::mem::size_of::<u16>()];

                reader.read_exact(&mut buf)?;

                let cell_edition: u16 = unsafe { std::mem::transmute(buf) };

                println!("cell_edition: {}", cell_edition);
            }
            HEADER_CELL_UPDATEDATE => {
                let buf_size =
                    record_base.get_record_len() as usize - std::mem::size_of::<OsencRecordBase>();

                let mut buf = vec![0u8; buf_size];

                reader.read_exact(&mut buf)?;

                if let Ok(cell_updatedate) = String::from_utf8(buf) {
                    println!("cell_updatedate: {}", cell_updatedate);
                }
            }
            HEADER_CELL_UPDATE => {
                let buf_size =
                    record_base.get_record_len() as usize - std::mem::size_of::<OsencRecordBase>();

                assert_eq!(buf_size, std::mem::size_of::<u16>());

                let mut buf = [0u8; std::mem::size_of::<u16>()];

                reader.read_exact(&mut buf)?;

                let cell_update: u16 = unsafe { std::mem::transmute(buf) };

                println!("cell_update: {}", cell_update);
            }
            HEADER_CELL_NATIVESCALE => {
                let buf_size =
                    record_base.get_record_len() as usize - std::mem::size_of::<OsencRecordBase>();

                assert_eq!(buf_size, std::mem::size_of::<u32>());

                let mut buf = [0u8; std::mem::size_of::<u32>()];

                reader.read_exact(&mut buf)?;

                let cell_update: u32 = unsafe { std::mem::transmute(buf) };

                println!("cell_nativescale: {}", cell_update);
            }

            HEADER_CELL_SOUNDINGDATUM => {
                let buf_size =
                    record_base.get_record_len() as usize - std::mem::size_of::<OsencRecordBase>();

                let mut buf = vec![0u8; buf_size];

                reader.read_exact(&mut buf)?;

                if let Ok(cell_soundingdatum) = String::from_utf8(buf) {
                    println!("cell_soundingsdatum: {}", cell_soundingdatum);
                }
            }

            HEADER_CELL_SENCCREATEDATE => {
                let buf_size =
                    record_base.get_record_len() as usize - std::mem::size_of::<OsencRecordBase>();

                let mut buf = vec![0u8; buf_size];

                reader.read_exact(&mut buf)?;
            }

            CELL_EXTENT_RECORD => {
                let buf_size =
                    record_base.get_record_len() as usize - std::mem::size_of::<OsencRecordBase>();

                assert_eq!(buf_size, std::mem::size_of::<OsencExtentRecordPayload>());

                let mut buf = [0u8; std::mem::size_of::<OsencExtentRecordPayload>()];

                reader.read_exact(&mut buf)?;

                let cell_extent_record: OsencExtentRecordPayload =
                    unsafe { std::mem::transmute(buf) };

                println!("cell_extent_record: {:#?}", cell_extent_record);
            }

            CELL_COVR_RECORD => {
                let buf_size =
                    record_base.get_record_len() as usize - std::mem::size_of::<OsencRecordBase>();

                let mut buf = vec![0u8; buf_size];

                reader.read_exact(&mut buf)?;
            }
            CELL_NOCOVR_RECORD => {
                let buf_size =
                    record_base.get_record_len() as usize - std::mem::size_of::<OsencRecordBase>();

                let mut buf = vec![0u8; buf_size];

                reader.read_exact(&mut buf)?;
            }
            _ => {
                break;
            }
        }
    }

    Ok(())
}
