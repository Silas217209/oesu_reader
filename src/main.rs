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
    ffi::OsStr,
    fs::{self, File},
    io::BufReader,
};

use chartfile::ChartFile;

mod chartfile;
mod s57;
mod types;

fn main() {
    let paths = fs::read_dir("/home/silas/Downloads/exported/").expect("count not open dir");
    for path in paths {
        let path = path.unwrap();

        if path.path().is_dir() {
            continue;
        }

        if path.path().extension().and_then(OsStr::to_str) != Some("oesu") {
            continue;
        }

        let file = File::open("/home/silas/Downloads/exported/OC-49-M11SO4.oesu")
            .expect("couldnt open file");
        let mut reader = BufReader::new(file);
        let result = ChartFile::parse_file(&mut reader);
        if let Ok(_) = result {
            println!("successfully read {}", path.file_name().to_str().unwrap());
        } else {
            println!(
                "failed to read {} with err {}",
                path.file_name().to_str().unwrap(),
                result.err().unwrap()
            );
        }
    }
}
