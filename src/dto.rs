pub mod pb {
    tonic::include_proto!("mod");
}

use agglayer_tries::roots::LocalExitRoot;
use agglayer_types::{Address, Digest, Signature, U256};
use agglayer_types::{Certificate, Height, Metadata, aggchain_proof::AggchainData};
use eyre::Result;
use eyre::{Error, eyre};
use unified_bridge::{
    BridgeExit, ClaimFromMainnet, ClaimFromRollup, ImportedBridgeExit, L1InfoTreeLeaf,
    L1InfoTreeLeafInner, LETMerkleProof, MerkleProof,
};
use unified_bridge::{Claim, NetworkId};
use unified_bridge::{GlobalIndex, LeafType};

use pb::agglayer::interop::types::v1::BridgeExit as RpcBridgeExit;
use pb::agglayer::interop::types::v1::FixedBytes20;
use pb::agglayer::interop::types::v1::FixedBytes32;
use pb::agglayer::interop::types::v1::ImportedBridgeExit as RpcImportedBridgeExit;
use pb::agglayer::interop::types::v1::L1InfoTreeLeaf as RpcL1InfoTreeLeaf;
use pb::agglayer::interop::types::v1::L1InfoTreeLeafWithContext as RpcL1InfoTreeLeafWithContext;
use pb::agglayer::interop::types::v1::MerkleProof as RpcMerkleProof;
use pb::agglayer::interop::types::v1::imported_bridge_exit::Claim as RpcClaim;
use pb::agglayer::node::types::v1::Certificate as RpcCertificate;
use unified_bridge::TokenInfo;

// Helper macro used by the rest of this module
macro_rules! must {
    ($from:expr, $field:ident) => {
        $from
            .$field
            .ok_or(eyre!("Missing field {}", stringify!($field)))?
    };
}

impl TryFrom<FixedBytes32> for U256 {
    type Error = Error;
    fn try_from(value: FixedBytes32) -> Result<Self, Self::Error> {
        let bytes: [u8; 32] = value
            .value
            .try_into()
            .map_err(|_| eyre!("Failed to convert FixedBytes32 to U256"))?;
        Ok(U256::from_be_bytes(bytes))
    }
}

impl TryFrom<FixedBytes32> for Digest {
    type Error = Error;
    fn try_from(value: FixedBytes32) -> Result<Self, Self::Error> {
        let bytes: [u8; 32] = value
            .value
            .try_into()
            .map_err(|_| eyre!("Failed to convert FixedBytes32 to Digest"))?;
        Ok(Digest::from(bytes))
    }
}

impl TryFrom<FixedBytes20> for Address {
    type Error = Error;
    fn try_from(value: FixedBytes20) -> Result<Self, Self::Error> {
        let bytes: [u8; 20] = value
            .value
            .try_into()
            .map_err(|_| eyre!("Failed to convert FixedBytes20 to Address"))?;
        Ok(Address::new(bytes))
    }
}

impl TryFrom<RpcMerkleProof> for MerkleProof {
    type Error = Error;
    fn try_from(value: RpcMerkleProof) -> Result<Self, Self::Error> {
        let siblings: Result<Vec<Digest>, Error> =
            value.siblings.into_iter().map(TryInto::try_into).collect();

        let proof = LETMerkleProof {
            siblings: siblings?
                .try_into()
                .map_err(|_| eyre!("Invalid merkle proof len"))?,
        };
        Ok(MerkleProof {
            proof,
            root: must!(value, root).try_into()?,
        })
    }
}

impl TryFrom<RpcL1InfoTreeLeaf> for L1InfoTreeLeafInner {
    type Error = Error;
    fn try_from(value: RpcL1InfoTreeLeaf) -> Result<Self, Self::Error> {
        Ok(L1InfoTreeLeafInner {
            global_exit_root: must!(value, global_exit_root).try_into()?,
            block_hash: must!(value, block_hash).try_into()?,
            timestamp: value.timestamp,
        })
    }
}

impl TryFrom<RpcL1InfoTreeLeafWithContext> for L1InfoTreeLeaf {
    type Error = Error;
    fn try_from(value: RpcL1InfoTreeLeafWithContext) -> Result<Self, Self::Error> {
        Ok(L1InfoTreeLeaf {
            l1_info_tree_index: value.l1_info_tree_index,
            rer: must!(value, rer).try_into()?,
            mer: must!(value, mer).try_into()?,
            inner: must!(value, inner).try_into()?,
        })
    }
}

impl TryFrom<RpcBridgeExit> for BridgeExit {
    type Error = Error;
    fn try_from(value: RpcBridgeExit) -> Result<Self, Self::Error> {
        let token_info = must!(value, token_info);
        Ok(BridgeExit {
            leaf_type: LeafType::try_from(value.leaf_type as u8)?,
            token_info: TokenInfo {
                origin_network: NetworkId::new(token_info.origin_network),
                origin_token_address: must!(token_info, origin_token_address).try_into()?,
            },
            dest_network: NetworkId::new(value.dest_network),
            dest_address: must!(value, dest_address).try_into()?,
            amount: must!(value, amount).try_into()?,
            metadata: value.metadata.map(|m| m.try_into()).transpose()?,
        })
    }
}

impl TryFrom<RpcClaim> for Claim {
    type Error = Error;
    fn try_from(value: RpcClaim) -> Result<Self, Self::Error> {
        Ok(match value {
            RpcClaim::Mainnet(mainnet) => Claim::Mainnet(Box::new(ClaimFromMainnet {
                proof_leaf_mer: must!(mainnet, proof_leaf_mer).try_into()?,
                proof_ger_l1root: must!(mainnet, proof_ger_l1root).try_into()?,
                l1_leaf: must!(mainnet, l1_leaf).try_into()?,
            })),
            RpcClaim::Rollup(rollup) => Claim::Rollup(Box::new(ClaimFromRollup {
                proof_leaf_ler: must!(rollup, proof_leaf_ler).try_into()?,
                proof_ler_rer: must!(rollup, proof_ler_rer).try_into()?,
                proof_ger_l1root: must!(rollup, proof_ger_l1root).try_into()?,
                l1_leaf: must!(rollup, l1_leaf).try_into()?,
            })),
        })
    }
}

impl TryFrom<RpcImportedBridgeExit> for ImportedBridgeExit {
    type Error = Error;
    fn try_from(value: RpcImportedBridgeExit) -> Result<Self, Self::Error> {
        let global_index: U256 = must!(value, global_index).try_into()?;

        Ok(ImportedBridgeExit {
            bridge_exit: must!(value, bridge_exit).try_into()?,
            claim_data: must!(value, claim).try_into()?,
            global_index: GlobalIndex::try_from(global_index)?,
        })
    }
}

impl TryFrom<RpcCertificate> for Certificate {
    type Error = Error;

    fn try_from(value: RpcCertificate) -> Result<Self, Self::Error> {
        let certificate = Certificate {
            network_id: NetworkId::new(value.network_id),
            height: Height::new(value.height),
            prev_local_exit_root: LocalExitRoot::new(
                must!(value, prev_local_exit_root).try_into()?,
            ),
            new_local_exit_root: LocalExitRoot::new(must!(value, new_local_exit_root).try_into()?),
            bridge_exits: value
                .bridge_exits
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()
                .map_err(|e: Error| eyre!("bridge_exits {}", e))?,
            imported_bridge_exits: value
                .imported_bridge_exits
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()
                .map_err(|e: Error| eyre!("imported_bridge_exits {}", e))?,
            metadata: if let Some(metadata) = value.metadata {
                Metadata::new(metadata.try_into()?)
            } else {
                Metadata::default()
            },
            custom_chain_data: value.custom_chain_data.to_vec(),
            l1_info_tree_leaf_count: value.l1_info_tree_leaf_count,
            aggchain_data: AggchainData::ECDSA {
                signature: Signature::new(U256::ZERO, U256::ZERO, false),
            },
        };

        Ok(certificate)
    }
}
