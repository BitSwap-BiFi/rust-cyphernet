// Set of libraries for privacy-preserving networking apps
//
// SPDX-License-Identifier: Apache-2.0
//
// Written in 2019-2023 by
//     Dr. Maxim Orlovsky <orlovsky@cyphernet.org>
//
// Copyright 2022-2023 Cyphernet Association, Switzerland
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

mod ceremony;
mod handshake;

pub use handshake::NoiseXkState;

mod init {
    use ed25519::x25519::{KeyPair, PublicKey, SecretKey};

    use super::NoiseXkState;
    use crate::noise::framing::NoiseTranscoder;

    impl NoiseTranscoder<NoiseXkState> {
        pub fn with_xk_initiator(local_key: SecretKey, remote_key: PublicKey) -> Self {
            let ephemeral_key = KeyPair::generate().sk;
            let state = NoiseXkState::new_initiator(local_key, remote_key, ephemeral_key);
            NoiseTranscoder { state }
        }

        pub fn with_xk_responder(local_key: SecretKey) -> Self {
            let ephemeral_key = KeyPair::generate().sk;
            let state = NoiseXkState::new_responder(local_key, ephemeral_key);
            NoiseTranscoder { state }
        }
    }
}
