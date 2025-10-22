use super::dto::pb::*;

use agglayer_types::Certificate;
use alloy::hex;
use alloy::hex::ToHexExt;
use alloy::signers::local::LocalSigner;
use eyre::Result;
use log::info;
use tonic::{Request, Response, Status, transport::Server};

use aggkit::aggsender::validator::v1::HealthCheckResponse;
use aggkit::aggsender::validator::v1::ValidateCertificateRequest;
use aggkit::aggsender::validator::v1::ValidateCertificateResponse;
use aggkit::aggsender::validator::v1::aggsender_validator_server::AggsenderValidator;
use aggkit::aggsender::validator::v1::aggsender_validator_server::AggsenderValidatorServer;
use agglayer::interop::types::v1::FixedBytes65;
use alloy::signers::Signer;

struct CertificateValidatorService {
    signer: LocalSigner<alloy::signers::k256::ecdsa::SigningKey>,
}
impl CertificateValidatorService {
    async fn sign(&self, certificate: &Certificate) -> Result<[u8; 65]> {
        let hash = certificate
            .signature_commitment_values()
            .multisig_commitment();

        info!("Hash is: {}", hash.encode_hex());

        let signature = self.signer.sign_hash(&hash.into()).await?.as_bytes();

        Ok(signature)
    }
}

#[tonic::async_trait]
impl AggsenderValidator for CertificateValidatorService {
    async fn validate_certificate(
        &self,
        req: Request<ValidateCertificateRequest>,
    ) -> Result<Response<ValidateCertificateResponse>, Status> {
        info!("Got certificate {:?}", req);

        let req = req.into_inner();
        let certificate = req.certificate.ok_or(Status::invalid_argument(
            "CertificateValidatorService: Unable to get certificate",
        ))?;
        let certificate: Certificate = certificate.try_into().map_err(|err| {
            Status::invalid_argument(format!(
                "CertificateValidatorService: Unable to parse certificate: {}",
                err
            ))
        })?;

        let signature = self
            .sign(&certificate)
            .await
            .map_err(|_| Status::internal("CertificateValidatorService: Unable to sign"))?;

        let res = ValidateCertificateResponse {
            signature: Some(FixedBytes65 {
                value: signature.into(),
            }),
        };
        Ok(Response::new(res))
    }
    async fn health_check(
        &self,
        _req: Request<()>,
    ) -> Result<Response<HealthCheckResponse>, Status> {
        Ok(Response::new(HealthCheckResponse {
            version: "light-certificate-validator-0.0.1".into(),
            status: "OK".into(),
            reason: "ALL systems ok".into(),
        }))
    }
}

pub async fn run_server() -> Result<()> {
    info!("Server started");
    let addr = "0.0.0.0:50051".parse().unwrap();
    let sk: [u8; 32] = hex!("b417f3004733a5890c75ea097ae4bb11129acf831bb00509193ddd8832d3adce");
    let signer: LocalSigner<alloy::signers::k256::ecdsa::SigningKey> =
        LocalSigner::from_slice(&sk)?;

    info!("Listening in {}", addr);
    info!("Signer address is {}", signer.address());

    let certificate_validator = CertificateValidatorService { signer };
    Server::builder()
        .add_service(AggsenderValidatorServer::new(certificate_validator))
        .serve(addr)
        .await?;
    Ok(())
}
