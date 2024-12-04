/*
 * Copyright © 2024 Silas Pachali
 *
 * Licensed under the EUPL, Version 1.2 or – as soon they will be
 * approved by the European Commission - subsequent versions of the
 * EUPL (the "Licence");
 * You may not use this work except in compliance with the Licence.
 * You may obtain a copy of the Licence at:
 *
 * https://joinup.ec.europa.eu/software/page/eupl
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the Licence is distributed on an
 * "AS IS" basis, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND,
 * either express or implied. See the Licence for the specific
 * language governing permissions and limitations under the Licence.
 */

use std::{
    ffi::{c_char, CStr},
    io::{ErrorKind, Read, Seek, SeekFrom},
};

use crate::{
    s57::{self, LineElement, PointGeometry, Position, Rect, S57Attribute, S57},
    types::{
        OsencAreaGeometryRecordPayload, OsencAttributeRecordPayload, OsencExtentRecordPayload,
        OsencFeatureIdentificationRecordPayload, OsencLineGeometryRecordPayload,
        OsencMultipointGeometryRecordPayload, OsencPointGeometryRecordPayload, OsencRecordBase,
        OsencServerstatRecordPayload,
    },
};

#[allow(dead_code)]
pub struct ChartFile {
    extent: Rect,
    s57: Vec<S57>,
    name: String,
    publishdate: String,
    edition: u16,
    updatedate: String,
    update: u16,
    nativescale: u32,
    soundingdatum: String,
}

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

impl ChartFile {
    pub fn parse_file<R: Read + Seek>(reader: &mut R) -> std::io::Result<ChartFile> {
        let mut extent: Rect = Rect {
            top_left: Position { lat: 0.0, lon: 0.0 },
            bottom_right: Position { lat: 0.0, lon: 0.0 },
        };
        let mut name = String::new();
        let mut publishdate = String::new();
        let mut s57_vector: Vec<S57> = Vec::new();
        let mut edition = 0u16;
        let mut updatedate = String::new();
        let mut update = 0u16;
        let mut nativescale = 0u32;
        let mut soundingdatum = String::new();

        // let vector_edges: HashMap<u16, VectorEdge> = HashMap::new();
        // let connected_nodes: HashMap<u16, ConnectedNode> = HashMap::new();

        let mut current_s57: Option<&mut S57> = None;

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

                    let buf_size = record_base.get_record_len() as usize
                        - std::mem::size_of::<OsencRecordBase>();

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

                    let buf_size = record_base.get_record_len() as usize
                        - std::mem::size_of::<OsencRecordBase>();

                    assert_eq!(buf_size, std::mem::size_of::<u16>());
                    let mut buf = [0u8; std::mem::size_of::<u16>()];

                    reader.read_exact(&mut buf)?;

                    let version: u16 = unsafe { std::mem::transmute(buf) };

                    if version < 201 {
                        return Err(std::io::Error::new(ErrorKind::Other, "Unsupported Version"));
                    }
                }
                HEADER_CELL_NAME => {
                    let buf_size = record_base.get_record_len() as usize
                        - std::mem::size_of::<OsencRecordBase>();

                    let mut buf = vec![0u8; buf_size];

                    reader.read_exact(&mut buf)?;

                    if let Ok(cell_name) = String::from_utf8(buf) {
                        name = cell_name;
                    }
                }

                HEADER_CELL_PUBLISHDATE => {
                    let buf_size = record_base.get_record_len() as usize
                        - std::mem::size_of::<OsencRecordBase>();

                    let mut buf = vec![0u8; buf_size];

                    reader.read_exact(&mut buf)?;

                    if let Ok(cell_publishdate) = String::from_utf8(buf) {
                        publishdate = cell_publishdate;
                    }
                }
                HEADER_CELL_EDITION => {
                    let buf_size = record_base.get_record_len() as usize
                        - std::mem::size_of::<OsencRecordBase>();

                    assert_eq!(buf_size, std::mem::size_of::<u16>());

                    let mut buf = [0u8; std::mem::size_of::<u16>()];

                    reader.read_exact(&mut buf)?;

                    let cell_edition: u16 = unsafe { std::mem::transmute(buf) };

                    edition = cell_edition;
                }
                HEADER_CELL_UPDATEDATE => {
                    let buf_size = record_base.get_record_len() as usize
                        - std::mem::size_of::<OsencRecordBase>();

                    let mut buf = vec![0u8; buf_size];

                    reader.read_exact(&mut buf)?;

                    if let Ok(cell_updatedate) = String::from_utf8(buf) {
                        updatedate = cell_updatedate;
                    }
                }
                HEADER_CELL_UPDATE => {
                    let buf_size = record_base.get_record_len() as usize
                        - std::mem::size_of::<OsencRecordBase>();

                    assert_eq!(buf_size, std::mem::size_of::<u16>());

                    let mut buf = [0u8; std::mem::size_of::<u16>()];

                    reader.read_exact(&mut buf)?;

                    let cell_update: u16 = unsafe { std::mem::transmute(buf) };

                    update = cell_update;
                }
                HEADER_CELL_NATIVESCALE => {
                    let buf_size = record_base.get_record_len() as usize
                        - std::mem::size_of::<OsencRecordBase>();

                    assert_eq!(buf_size, std::mem::size_of::<u32>());

                    let mut buf = [0u8; std::mem::size_of::<u32>()];

                    reader.read_exact(&mut buf)?;

                    let cell_nativescale: u32 = unsafe { std::mem::transmute(buf) };

                    nativescale = cell_nativescale;
                }

                HEADER_CELL_SOUNDINGDATUM => {
                    let buf_size = record_base.get_record_len() as usize
                        - std::mem::size_of::<OsencRecordBase>();

                    let mut buf = vec![0u8; buf_size];

                    reader.read_exact(&mut buf)?;

                    if let Ok(cell_soundingdatum) = String::from_utf8(buf) {
                        soundingdatum = cell_soundingdatum;
                    }
                }

                HEADER_CELL_SENCCREATEDATE => {
                    let buf_size = record_base.get_record_len() as usize
                        - std::mem::size_of::<OsencRecordBase>();

                    reader.seek(SeekFrom::Current(buf_size as i64))?;
                }

                CELL_EXTENT_RECORD => {
                    let buf_size = record_base.get_record_len() as usize
                        - std::mem::size_of::<OsencRecordBase>();

                    assert_eq!(buf_size, std::mem::size_of::<OsencExtentRecordPayload>());

                    let mut buf = [0u8; std::mem::size_of::<OsencExtentRecordPayload>()];

                    reader.read_exact(&mut buf)?;

                    let cell_extent_record: OsencExtentRecordPayload =
                        unsafe { std::mem::transmute(buf) };

                    extent.top_left = Position {
                        lat: cell_extent_record.extent_nw_lat,
                        lon: cell_extent_record.extent_nw_lon,
                    };

                    extent.bottom_right = Position {
                        lat: cell_extent_record.extent_se_lat,
                        lon: cell_extent_record.extent_se_lon,
                    };
                }

                CELL_COVR_RECORD => {
                    let buf_size = record_base.get_record_len() as usize
                        - std::mem::size_of::<OsencRecordBase>();

                    reader.seek(SeekFrom::Current(buf_size as i64))?;
                }
                CELL_NOCOVR_RECORD => {
                    let buf_size = record_base.get_record_len() as usize
                        - std::mem::size_of::<OsencRecordBase>();

                    reader.seek(SeekFrom::Current(buf_size as i64))?;
                }
                FEATURE_ID_RECORD => {
                    let buf_size = record_base.get_record_len() as usize
                        - std::mem::size_of::<OsencRecordBase>();

                    assert_eq!(
                        buf_size,
                        std::mem::size_of::<OsencFeatureIdentificationRecordPayload>()
                    );

                    let mut buf =
                        [0u8; std::mem::size_of::<OsencFeatureIdentificationRecordPayload>()];

                    reader.read_exact(&mut buf)?;

                    let payload: OsencFeatureIdentificationRecordPayload =
                        unsafe { std::mem::transmute(buf) };

                    s57_vector.push(S57::from_type_code(payload.get_feature_type_code()));
                    current_s57 = s57_vector.last_mut();
                }
                FEATURE_ATTRIBUTE_RECORD => {
                    let buf_size = record_base.get_record_len() as usize
                        - std::mem::size_of::<OsencRecordBase>();

                    //assert_eq!(buf_size, std::mem::size_of::<OsencAttributeRecordPayload>());

                    let mut buf = vec![0u8; buf_size];

                    reader.read_exact(&mut buf)?;

                    let payload = unsafe {
                        // https://github.com/bdbcat/o-charts_pi/blob/e10fc5c3e9da31a1d19b264df1ac11e39d9226bb/src/Osenc.cpp#L1500
                        // WARNING: Intentionally mimics buggy(?) C++ implementation
                        // Original code inconsistently reads buffer of varying lengths (5-12 bytes)
                        // into a fixed 11-byte struct, suggesting a potential memory handling bug
                        // in the original C++ code that miraculously "worked"
                        std::ptr::read_unaligned(buf.as_ptr() as *const OsencAttributeRecordPayload)
                    };

                    let attribute_value_type = payload.get_attribute_value_type();
                    let attribute = S57Attribute::from_type_code(payload.get_attribute_type_code());

                    if attribute == S57Attribute::Unknown {
                        continue;
                    }

                    match attribute_value_type {
                        0 => {
                            if let Some(ref mut s57) = current_s57 {
                                s57.set_attribute(
                                    attribute,
                                    s57::AttributeValue::UInt32(
                                        payload.get_attribute_value().get_int(),
                                    ),
                                );
                            }
                        }
                        2 => {
                            if let Some(ref mut s57) = current_s57 {
                                s57.set_attribute(
                                    attribute,
                                    s57::AttributeValue::Double(
                                        payload.get_attribute_value().get_double(),
                                    ),
                                );
                            }
                        }
                        4 => {
                            if let Some(ref mut s57) = current_s57 {
                                let char_ptr = buf.as_ptr() as *const c_char;

                                let string_offset =
                                    std::mem::size_of::<OsencAttributeRecordPayload>()
                                        - std::mem::size_of::<*const c_char>();

                                let c_str = unsafe {
                                    CStr::from_ptr(
                                        (char_ptr as *const u8).add(string_offset) as *const c_char
                                    )
                                };

                                if let Ok(str) = c_str.to_str() {
                                    s57.set_attribute(
                                        attribute,
                                        s57::AttributeValue::String(str.to_string()),
                                    );
                                }
                            }
                        }
                        _ => {}
                    }
                }
                FEATURE_GEOMETRY_RECORD_POINT => {
                    let buf_size = record_base.get_record_len() as usize
                        - std::mem::size_of::<OsencRecordBase>();

                    assert_eq!(
                        buf_size,
                        std::mem::size_of::<OsencPointGeometryRecordPayload>()
                    );

                    let mut buf = [0u8; std::mem::size_of::<OsencPointGeometryRecordPayload>()];

                    reader.read_exact(&mut buf)?;

                    let point: OsencPointGeometryRecordPayload =
                        unsafe { std::mem::transmute(buf) };
                    if let Some(ref mut s57) = current_s57 {
                        s57.set_point_geometry(point.into());
                    }
                }
                FEATURE_GEOMETRY_RECORD_AREA => {
                    let payload_size = record_base.get_record_len() as usize
                        - std::mem::size_of::<OsencRecordBase>();
                    let mut payload_buffer = vec![0u8; payload_size];
                    reader.read_exact(&mut payload_buffer)?;

                    let mut cursor = std::io::Cursor::new(&payload_buffer);

                    let mut record_buf =
                        [0u8; std::mem::size_of::<OsencAreaGeometryRecordPayload>()];
                    cursor.read_exact(&mut record_buf)?;

                    let record: OsencAreaGeometryRecordPayload =
                        unsafe { std::mem::transmute(record_buf) };

                    // skip tesselation data
                    let triprim_count = record.get_triprim_count();
                    let countour_count = record.get_contour_count();

                    cursor.seek(SeekFrom::Current(
                        countour_count as i64 * std::mem::size_of::<i32>() as i64,
                    ))?;

                    for _ in 0..triprim_count {
                        cursor.seek(SeekFrom::Current(1))?;

                        let mut data_nvert = [0u8; std::mem::size_of::<u32>()];
                        cursor.read_exact(&mut data_nvert)?;

                        let nvert: u32 = unsafe { std::mem::transmute(data_nvert) };
                        let byte_size = nvert as i64 * 2 * std::mem::size_of::<f32>() as i64;

                        cursor.seek(SeekFrom::Current(4 * std::mem::size_of::<f64>() as i64))?;
                        cursor.seek(SeekFrom::Current(byte_size))?;
                    }

                    let remaining_size = payload_size - cursor.position() as usize;

                    assert_eq!(remaining_size % std::mem::size_of::<LineElement>(), 0);

                    let mut line_data: Vec<u8> = vec![0u8; remaining_size];
                    cursor.read_exact(&mut line_data)?;

                    let ptr = line_data.as_mut_ptr() as *mut LineElement;
                    let len = line_data.len() / std::mem::size_of::<LineElement>();

                    std::mem::forget(line_data);

                    let lines: Vec<LineElement> = unsafe { Vec::from_raw_parts(ptr, len, len) };
                    if let Some(ref mut s57) = current_s57 {
                        s57.set_polygon_geometry(&lines);
                    }
                }

                FEATURE_GEOMETRY_RECORD_AREA_EXT => {
                    let buf_size = record_base.get_record_len() as usize
                        - std::mem::size_of::<OsencRecordBase>();

                    reader.seek(SeekFrom::Current(buf_size as i64))?;
                }
                FEATURE_GEOMETRY_RECORD_LINE => {
                    let payload_size = record_base.get_record_len() as usize
                        - std::mem::size_of::<OsencRecordBase>();
                    let mut payload_buffer = vec![0u8; payload_size];

                    reader.read_exact(&mut payload_buffer)?;

                    let mut cursor = std::io::Cursor::new(&payload_buffer);

                    cursor.seek(SeekFrom::Current(
                        std::mem::size_of::<OsencLineGeometryRecordPayload>() as i64,
                    ))?;

                    assert_eq!(
                        (payload_size - cursor.position() as usize)
                            % std::mem::size_of::<LineElement>(),
                        0
                    );

                    let mut line_data = vec![0u8; payload_size - cursor.position() as usize];
                    cursor.read_exact(&mut line_data)?;

                    let ptr = line_data.as_mut_ptr() as *mut LineElement;
                    let len = line_data.len() / std::mem::size_of::<LineElement>();

                    std::mem::forget(line_data);

                    let lines: Vec<LineElement> = unsafe { Vec::from_raw_parts(ptr, len, len) };
                    if let Some(ref mut s57) = current_s57 {
                        s57.set_line_geometry(&lines);
                    }
                }
                FEATURE_GEOMETRY_RECORD_MULTIPOINT => {
                    let payload_size = record_base.get_record_len() as usize
                        - std::mem::size_of::<OsencRecordBase>();
                    let mut payload_buffer = vec![0u8; payload_size];

                    reader.read_exact(&mut payload_buffer)?;

                    let mut cursor = std::io::Cursor::new(&payload_buffer);

                    let mut record_data =
                        [0u8; std::mem::size_of::<OsencMultipointGeometryRecordPayload>()];
                    cursor.read_exact(&mut record_data)?;

                    let record: OsencMultipointGeometryRecordPayload =
                        unsafe { std::mem::transmute(record_data) };

                    let mut multipoint_geometry: Vec<PointGeometry> = Vec::new();

                    let mut point_data =
                        vec![0u8; std::mem::size_of::<f32>() * record.point_count as usize];
                    cursor.read_exact(&mut point_data)?;

                    let ptr = point_data.as_mut_ptr() as *mut f32;
                    let len = record.point_count as usize;

                    std::mem::forget(point_data);

                    let points: Vec<f32> = unsafe { Vec::from_raw_parts(ptr, len * 3, len) };

                    for i in 0..len {
                        let easting = points[i * 3 + 0] as f64;
                        let northing = points[i * 3 + 1] as f64;
                        let depth = points[i * 3 + 2] as f64;

                        let pos =
                            Position::from_simple_mercator(easting, northing, &extent.center());

                        multipoint_geometry.push(PointGeometry {
                            position: pos,
                            value: depth,
                        });
                    }

                    if let Some(ref mut s57) = current_s57 {
                        s57.set_multi_point_geometry(multipoint_geometry);
                    }
                }
                VECTOR_EDGE_NODE_TABLE_RECORD => {
                    // not needed for my data
                    let buf_size = record_base.get_record_len() as usize
                        - std::mem::size_of::<OsencRecordBase>();

                    reader.seek(SeekFrom::Current(buf_size as i64))?;
                }

                VECTOR_EDGE_NODE_TABLE_EXT_RECORD => {
                    // not needed for my data
                    let buf_size = record_base.get_record_len() as usize
                        - std::mem::size_of::<OsencRecordBase>();

                    reader.seek(SeekFrom::Current(buf_size as i64))?;
                }
                VECTOR_CONNECTED_NODE_TABLE_RECORD => {
                    // not needed for my data
                    let buf_size = record_base.get_record_len() as usize
                        - std::mem::size_of::<OsencRecordBase>();

                    reader.seek(SeekFrom::Current(buf_size as i64))?;
                }

                VECTOR_CONNECTED_NODE_TABLE_EXT_RECORD => {
                    // not needed for my data
                    let buf_size = record_base.get_record_len() as usize
                        - std::mem::size_of::<OsencRecordBase>();

                    reader.seek(SeekFrom::Current(buf_size as i64))?;
                }
                CELL_TXTDSC_INFO_FILE_RECORD => {
                    // not needed for my data
                    let buf_size = record_base.get_record_len() as usize
                        - std::mem::size_of::<OsencRecordBase>();

                    reader.seek(SeekFrom::Current(buf_size as i64))?;
                }
                _ => {
                    break;
                }
            }
        }

        Ok(ChartFile {
            extent,
            s57: s57_vector,
            name,
            publishdate,
            edition,
            updatedate,
            update,
            nativescale,
            soundingdatum,
        })
    }
}
