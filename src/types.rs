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
    extent_sw_lat: f64,
    extent_sw_lon: f64,
    extent_nw_lat: f64,
    extent_nw_lon: f64,
    extent_ne_lat: f64,
    extent_ne_lon: f64,
    extent_se_lat: f64,
    extent_se_lon: f64,
}

#[allow(dead_code)]
#[derive(Debug)]
#[repr(C, packed)]
pub struct OsencFeatureIdentificationRecordPayload {
    feature_type_code: u16,
    feature_id: u16,
    feature_primitive: u8,
}
