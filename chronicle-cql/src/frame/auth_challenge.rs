// Copyright 2020 IOTA Stiftung
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with
// the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on
// an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and limitations under the License.

//! This module implements the challenge part of the challenge–response authentication.

use super::decoder::{bytes, Decoder, Frame};

#[derive(Debug)]
/// The Autentication Challenge structure with the token field.
pub struct AuthChallenge {
    token: Option<Vec<u8>>,
}

impl AuthChallenge {
    /// Create a new `AuthChallenge ` from the body of frame.
    pub fn new(decoder: &Decoder) -> Self {
        Self::from(decoder.body())
    }
}

impl From<&[u8]> for AuthChallenge {
    fn from(slice: &[u8]) -> Self {
        let token = bytes(slice);
        Self { token }
    }
}
