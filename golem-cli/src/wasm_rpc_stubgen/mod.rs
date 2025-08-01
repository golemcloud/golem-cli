// Copyright 2024-2025 Golem Cloud
//
// Licensed under the Golem Source License v1.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://license.golem.cloud/LICENSE
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

pub mod cargo;
pub mod commands;
pub mod compilation;
pub mod naming;
pub mod rust;
pub mod stub;
pub mod wit_encode;
pub mod wit_generate;
pub mod wit_resolve;

pub const WIT_BINDGEN_VERSION: &str = "0.40.0";
pub const WASI_WIT_VERSION: &str = "0.2.3";
pub const GOLEM_RPC_WIT_VERSION: &str = "0.2.2";
pub const GOLEM_API_VERSION: &str = "1.1.7";
