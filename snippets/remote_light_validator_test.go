package validator

import (
	"fmt"
	"testing"
	"time"

	agglayertypes "github.com/agglayer/aggkit/agglayer/types"
	"github.com/agglayer/aggkit/aggsender/mocks"
	configtypes "github.com/agglayer/aggkit/config/types"
	"github.com/agglayer/aggkit/grpc"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/crypto"
	"github.com/stretchr/testify/require"
)

func TestRemoteClient_ValidateCertificateLight(t *testing.T) {
	ctx := t.Context()

	mockStorage := mocks.NewAggSenderStorage(t)

	clientConfig := &grpc.ClientConfig{
		URL:               "0.0.0.0:50051",
		MinConnectTimeout: configtypes.NewDuration(time.Second * 5),
		RequestTimeout:    configtypes.NewDuration(time.Second * 5),
		UseTLS:            false,
		Retry:             grpc.DefaultConfig().Retry,
	}

	signerAddress := common.HexToAddress("0x21Df7D616f54845F7aAF3Fe049390DeC9ba9d5eB")
	remoteValidator, err := NewRemoteValidator(clientConfig, mockStorage, signerAddress, 0)
	require.Equal(t, err, nil)

	certificate := &agglayertypes.Certificate{
		Height: 0,
	}

	sig, err := remoteValidator.ValidateAndSignCertificate(ctx, certificate, 0)
	hash, err := HashCertificateToSign(certificate)

	if sig[crypto.RecoveryIDOffset] == 27 || sig[crypto.RecoveryIDOffset] == 28 {
		sig[crypto.RecoveryIDOffset] -= 27
	}
	recoveredPublicKey, err := crypto.SigToPub(hash[:], sig)
	require.Equal(t, err, nil)

	recoveredAddress := crypto.PubkeyToAddress(*recoveredPublicKey)

	require.Equal(t, signerAddress, recoveredAddress)

	fmt.Println("signature is", signature)
}
