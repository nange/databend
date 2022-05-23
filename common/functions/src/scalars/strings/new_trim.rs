// Copyright 2021 Datafuse Labs.
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

use std::fmt;
use std::marker::PhantomData;

use common_datavalues::prelude::*;
use common_exception::Result;

use crate::scalars::assert_string;
use crate::scalars::Function;
use crate::scalars::FunctionContext;
use crate::scalars::FunctionDescription;
use crate::scalars::FunctionFeatures;

pub trait TrimOperator: Send + Sync + Clone + Default + 'static {
    fn apply<'a>(&'a mut self, _str: &'a [u8], _trim_str: &'a [u8], _buf: &mut Vec<u8>);
}

#[derive(Clone)]
pub struct TrimFunction<T> {
    display_name: String,
    _marker: PhantomData<T>,
}

impl<T: TrimOperator> TrimFunction<T> {
    pub fn try_create(display_name: &str, args: &[&DataTypeImpl]) -> Result<Box<dyn Function>> {
        for arg in args {
            assert_string(*arg)?;
        }
        Ok(Box::new(Self {
            display_name: display_name.to_string(),
            _marker: PhantomData,
        }))
    }

    pub fn desc() -> FunctionDescription {
        FunctionDescription::creator(Box::new(Self::try_create))
            .features(FunctionFeatures::default().deterministic().num_arguments(2))
    }
}

impl<T: TrimOperator> Function for TrimFunction<T> {
    fn name(&self) -> &str {
        &*self.display_name
    }

    fn return_type(&self) -> DataTypeImpl {
        StringType::new_impl()
    }

    fn eval(
        &self,
        _func_ctx: FunctionContext,
        columns: &ColumnsWithField,
        input_rows: usize,
    ) -> Result<ColumnRef> {
        let mut op = T::default();
        let column: &StringColumn = Series::check_get(columns[0].column())?;
        let estimate_bytes = column.values().len();

        let view0 = Vu8::try_create_viewer(columns[0].column())?;
        let view1 = Vu8::try_create_viewer(columns[1].column())?;

        let mut values = Vec::with_capacity(estimate_bytes);
        let mut offsets = Vec::with_capacity(input_rows + 1);
        offsets.push(0i64);

        for row in 0..input_rows {
            op.apply(view0.value_at(row), view1.value_at(row), &mut values);
            offsets.push(values.len() as i64);
        }
        let mut builder = MutableStringColumn::from_data(values, offsets);
        Ok(builder.to_column())
    }
}

impl<T: TrimOperator> fmt::Display for TrimFunction<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.display_name)
    }
}

#[derive(Clone, Default)]
pub struct TrimLeading;

impl TrimOperator for TrimLeading {
    #[inline]
    fn apply<'a>(&'a mut self, str: &'a [u8], trim_str: &'a [u8], buffer: &mut Vec<u8>) {
        let chunk_size = trim_str.len();
        for (idx, chunk) in str.chunks(chunk_size).enumerate() {
            if chunk != trim_str {
                buffer.extend_from_slice(&str[idx * chunk_size..]);
                return;
            }
        }
    }
}

#[derive(Clone, Default)]
pub struct TrimTrailing;

impl TrimOperator for TrimTrailing {
    // impl TrimOperator for TrimTrialer {
    fn apply<'a>(&'a mut self, str: &'a [u8], trim_str: &'a [u8], buffer: &mut Vec<u8>) {
        let chunk_size = trim_str.len();
        for (idx, chunk) in str.rchunks(chunk_size).enumerate() {
            if chunk != trim_str {
                let trim_bytes = idx * chunk_size;
                buffer.extend_from_slice(&str[..str.len() - trim_bytes]);
                return;
            }
        }
    }
}

#[derive(Clone, Default)]
pub struct TrimBoth;

impl TrimOperator for TrimBoth {
    // impl TrimOperator for TrimBoth {
    fn apply<'a>(&'a mut self, str: &'a [u8], trim_str: &'a [u8], buffer: &mut Vec<u8>) {
        let chunk_size = trim_str.len();
        let start_idx = str
            .chunks(chunk_size)
            .enumerate()
            .find(|(_, chunk)| chunk != &trim_str)
            .map(|(idx, _)| idx);

        // Trim all
        if matches!(start_idx, Some(idx) if idx * chunk_size == str.len()) {
            return;
        }

        let end_idx = str
            .rchunks(chunk_size)
            .enumerate()
            .find(|(_, chunk)| chunk != &trim_str)
            .map(|(idx, _)| idx);

        match (start_idx, end_idx) {
            (Some(start_index), Some(end_index)) => {
                buffer.extend_from_slice(
                    &str[start_index * chunk_size..str.len() - end_index * chunk_size],
                );
            }
            _ => {}
        }
    }
}

pub type TrimLeadingFunction = TrimFunction<TrimLeading>;
pub type TrimTrailingFunction = TrimFunction<TrimTrailing>;
pub type TrimBothFunction = TrimFunction<TrimBoth>;
