// Copyright 2021 Datafuse Labs
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use chrono_tz::Tz;
use common_ast::ast::Identifier;
use common_ast::ast::InsertSource;
use common_ast::ast::InsertStmt;
use common_expression::types::DataType;
use common_expression::Column;
use common_expression::ScalarRef;
use common_formats::field_encoder::FieldEncoderRowBased;
use common_formats::field_encoder::FieldEncoderValues;
use common_formats::CommonSettings;
use common_io::constants::FALSE_BYTES_LOWER;
use common_io::constants::INF_BYTES_LOWER;
use common_io::constants::NAN_BYTES_LOWER;
use common_io::constants::NULL_BYTES_UPPER;
use common_io::constants::TRUE_BYTES_LOWER;
use itertools::join;
use rand::Rng;
use roaring::RoaringTreemap;

use crate::sql_gen::SqlGenerator;
use crate::sql_gen::Table;

impl<'a, R: Rng> SqlGenerator<'a, R> {
    pub(crate) fn gen_insert(&mut self, table: &Table, row_count: usize) -> InsertStmt {
        let table_name = Identifier::from_name(table.name.clone());
        let data_types = table
            .schema
            .fields()
            .iter()
            .map(|f| (&f.data_type).into())
            .collect::<Vec<DataType>>();
        let source = self.gen_insert_source(&data_types, row_count);

        InsertStmt {
            // TODO
            hints: None,
            catalog: None,
            database: None,
            table: table_name,
            // TODO
            columns: vec![],
            source,
            // TODO
            overwrite: false,
        }
    }

    fn gen_insert_source(&mut self, data_types: &[DataType], row_count: usize) -> InsertSource {
        match self.rng.gen_range(0..=9) {
            0..=9 => {
                let columns = self.gen_columns(data_types, row_count);
                let mut buf = Vec::new();
                let encoder = FieldEncoderValues {
                    common_settings: CommonSettings {
                        true_bytes: TRUE_BYTES_LOWER.as_bytes().to_vec(),
                        false_bytes: FALSE_BYTES_LOWER.as_bytes().to_vec(),
                        null_bytes: NULL_BYTES_UPPER.as_bytes().to_vec(),
                        nan_bytes: NAN_BYTES_LOWER.as_bytes().to_vec(),
                        inf_bytes: INF_BYTES_LOWER.as_bytes().to_vec(),
                        timezone: Tz::UTC,
                        disable_variant_check: false,
                    },
                    quote_char: b'\'',
                };

                for i in 0..row_count {
                    if i > 0 {
                        buf.extend_from_slice(b",");
                    }
                    buf.extend_from_slice(b"(");
                    for (j, column) in columns.iter().enumerate() {
                        if j > 0 {
                            buf.extend_from_slice(b",");
                        }
                        if column.data_type().remove_nullable() == DataType::Bitmap {
                            // convert binary bitmap to string
                            match unsafe { column.index_unchecked(i) } {
                                ScalarRef::Null => {
                                    buf.extend_from_slice(NULL_BYTES_UPPER.as_bytes());
                                }
                                ScalarRef::Bitmap(v) => {
                                    let rb = RoaringTreemap::deserialize_from(v).unwrap();
                                    let vals = rb.into_iter().collect::<Vec<_>>();
                                    let s = join(vals.iter(), ",");
                                    buf.push(b'\'');
                                    buf.extend_from_slice(s.as_bytes());
                                    buf.push(b'\'');
                                }
                                _ => unreachable!(),
                            }
                        } else {
                            encoder.write_field(column, i, &mut buf, false);
                        }
                    }
                    buf.extend_from_slice(b")");
                }
                InsertSource::Values {
                    rest_str: unsafe { String::from_utf8_unchecked(buf) },
                    start: 0,
                }
            }
            // TODO
            _ => unreachable!(),
        }
    }

    fn gen_columns(&mut self, data_types: &[DataType], row_count: usize) -> Vec<Column> {
        data_types
            .iter()
            .map(|ty| Column::random(ty, row_count))
            .collect()
    }
}
