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

use crate::s57::Position;

#[allow(dead_code)]
#[derive(Debug)]
#[repr(C, packed)]
pub struct OsencRecordBase {
    record_type: u16,
    record_len: u32,
}

#[allow(dead_code)]
impl OsencRecordBase {
    pub fn get_record_type(&self) -> u16 {
        return self.record_type;
    }
    pub fn get_record_len(&self) -> u32 {
        return self.record_len;
    }
}

#[allow(dead_code)]
#[derive(Debug)]
#[repr(C, packed)]
pub struct OsencServerstatRecordPayload {
    server_status: u16,
    decrypt_status: u16,
    expire_status: u16,
    expire_days_remaining: u16,
    grace_days_allowed: u16,
    grace_days_remaining: u16,
}

#[allow(dead_code)]
impl OsencServerstatRecordPayload {
    pub fn get_server_status(&self) -> u16 {
        self.server_status
    }

    pub fn get_decrypt_status(&self) -> u16 {
        self.decrypt_status
    }

    pub fn get_expire_status(&self) -> u16 {
        self.expire_status
    }

    pub fn get_expire_days_remaining(&self) -> u16 {
        self.expire_days_remaining
    }

    pub fn get_grace_days_allowed(&self) -> u16 {
        self.grace_days_allowed
    }

    pub fn get_grace_days_remaining(&self) -> u16 {
        self.grace_days_remaining
    }
}

#[allow(dead_code)]
#[derive(Debug)]
#[repr(C, packed)]
pub struct OsencExtentRecordPayload {
    pub extent_sw_lat: f64,
    pub extent_sw_lon: f64,
    pub extent_nw_lat: f64,
    pub extent_nw_lon: f64,
    pub extent_ne_lat: f64,
    pub extent_ne_lon: f64,
    pub extent_se_lat: f64,
    pub extent_se_lon: f64,
}

#[allow(dead_code)]
#[derive(Debug)]
#[repr(C, packed)]
pub struct OsencFeatureIdentificationRecordPayload {
    feature_type_code: u16,
    feature_id: u16,
    feature_primitive: u8,
}

impl OsencFeatureIdentificationRecordPayload {
    pub fn get_feature_type_code(&self) -> u16 {
        return self.feature_type_code;
    }
}

#[repr(C)]
#[repr(packed)]
pub struct OsencAttributeRecordPayload {
    attribute_type_code: u16,
    attribute_value_type: u8,
    attribute_value: OsencAttributeValue,
}

#[allow(dead_code)]
impl OsencAttributeRecordPayload {
    pub fn get_attribute_type_code(&self) -> u16 {
        return self.attribute_type_code;
    }
    pub fn get_attribute_value_type(&self) -> u8 {
        return self.attribute_value_type;
    }
    pub fn get_attribute_value(&self) -> OsencAttributeValue {
        return self.attribute_value;
    }
}

#[repr(C)]
#[repr(packed)]
#[derive(Clone, Copy)]
pub union OsencAttributeValue {
    attribute_value_int: u32,
    attribute_value_double: f64,
    attribute_value_char_ptr: *const u8,
}

#[allow(dead_code)]
impl OsencAttributeValue {
    pub fn get_int(&self) -> u32 {
        unsafe {
            return self.attribute_value_int;
        }
    }

    pub fn get_double(&self) -> f64 {
        unsafe {
            return self.attribute_value_double;
        }
    }

    pub fn get_char_ptr(&self) -> *const u8 {
        unsafe {
            return self.attribute_value_char_ptr;
        }
    }
}

#[derive(Debug)]
#[repr(C, packed)]
#[allow(dead_code)]
pub struct OsencPointGeometryRecordPayload {
    lat: f64,
    lon: f64,
}

impl Into<Position> for OsencPointGeometryRecordPayload {
    fn into(self) -> Position {
        return Position {
            lat: self.lat,
            lon: self.lon,
        };
    }
}

#[allow(dead_code)]
#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct OsencAreaGeometryRecordPayload {
    extent_s_lat: f64,
    extent_n_lat: f64,
    extent_w_lon: f64,
    extent_e_lon: f64,
    contour_count: u32,
    triprim_count: u32,
    edgevector_count: u32,
}

#[allow(dead_code)]
impl OsencAreaGeometryRecordPayload {
    pub fn get_extent_s_lat(&self) -> f64 {
        self.extent_s_lat
    }
    pub fn get_extent_n_lat(&self) -> f64 {
        self.extent_n_lat
    }

    pub fn get_extent_w_lon(&self) -> f64 {
        self.extent_w_lon
    }

    pub fn get_extent_e_lon(&self) -> f64 {
        self.extent_e_lon
    }

    pub fn get_contour_count(&self) -> u32 {
        self.contour_count
    }

    pub fn get_triprim_count(&self) -> u32 {
        self.triprim_count
    }

    pub fn get_edgevector_count(&self) -> u32 {
        self.edgevector_count
    }
}

#[derive(Debug)]
#[repr(C, packed)]
#[allow(dead_code)]
pub struct OsencLineGeometryRecordPayload {
    extent_s_lat: f64,
    extent_n_lat: f64,
    extent_w_lon: f64,
    extent_e_lon: f64,
    edgevector_count: u32,
}

#[derive(Debug)]
#[repr(C, packed)]
#[allow(dead_code)]
pub struct OsencMultipointGeometryRecordPayload {
    extent_s_lat: f64,
    extent_n_lat: f64,
    extent_w_lon: f64,
    extent_e_lon: f64,
    pub point_count: u32,
}
