// Copyright 2022 Datafuse Labs.
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

use std::sync::Arc;

use common_exception::Result;

use crate::base::GlobalInstance;
use crate::runtime::Runtime;

pub struct GlobalIORuntime;

pub struct GlobalHttpQueryRuntime(pub Runtime);

impl GlobalHttpQueryRuntime {
    #[inline(always)]
    pub fn runtime<'a>(self: &'a Arc<Self>) -> &'a Runtime {
        &self.0
    }
}

impl GlobalIORuntime {
    pub fn init(num_cpus: usize) -> Result<()> {
        let thread_num = std::cmp::max(num_cpus, num_cpus::get() / 2);
        let thread_num = std::cmp::max(2, thread_num);

        GlobalInstance::set(Arc::new(Runtime::with_worker_threads(
            thread_num,
            Some("IO-worker".to_owned()),
        )?));
        Ok(())
    }

    pub fn instance() -> Arc<Runtime> {
        GlobalInstance::get()
    }
}

impl GlobalHttpQueryRuntime {
    pub fn init(num_cpus: usize) -> Result<()> {
        let thread_num = std::cmp::max(num_cpus, num_cpus::get() / 2);
        let thread_num = std::cmp::max(2, thread_num);

        let rt =
            Runtime::with_worker_threads(thread_num, Some("http-query-ctx-worker".to_owned()))?;
        GlobalInstance::set(Arc::new(GlobalHttpQueryRuntime(rt)));
        Ok(())
    }

    pub fn instance() -> Arc<GlobalHttpQueryRuntime> {
        GlobalInstance::get()
    }
}
