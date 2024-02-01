pragma solidity ^0.8.23;

import "solidity-bytes-utils/BytesLib.sol";

import {IMembershipVerifier} from "../../../contracts/core/IMembershipVerifier.sol";
import {IZKVerifierV2} from "../../../contracts/core/IZKVerifierV2.sol";
import {CometblsClient} from "../../../contracts/clients/CometblsClientV2.sol";
import {IBCConnectionLib} from "../../../contracts/core/03-connection/IBCConnection.sol";
import {IBCMsgs} from "../../../contracts/core/25-handler/IBCMsgs.sol";
import {IbcCoreConnectionV1ConnectionEnd as ConnectionEnd, IbcCoreConnectionV1Counterparty as ConnectionCounterparty, IbcCoreConnectionV1GlobalEnums as ConnectionEnums} from "../../../contracts/proto/ibc/core/connection/v1/connection.sol";
import {ILightClient} from "../../../contracts/core/02-client/ILightClient.sol";
import {IBCCommitment} from "../../../contracts/core/24-host/IBCCommitment.sol";
import {IbcCoreCommitmentV1MerklePrefix as CommitmentMerklePrefix} from "../../../contracts/proto/ibc/core/commitment/v1/commitment.sol";
import {TendermintTypesSignedHeader} from "../../../contracts/proto/tendermint/types/canonical.sol";
import {TendermintTypesCommit, TendermintTypesHeader, TendermintTypesSignedHeader, TendermintVersionConsensus, TendermintTypesCommitSig, TendermintTypesBlockID, TendermintTypesPartSetHeader} from "../../../contracts/proto/tendermint/types/types.sol";

import "../TestPlus.sol";

contract TestVerifier is IZKVerifierV2 {
    function verifyProof(
        uint256[8] memory proof,
        uint256[2] memory proofCommitment,
        uint256[2] calldata proofCommitmentPOK,
        uint256[4] calldata input
    ) external returns (bool) {
        return true;
    }
}

contract TestMembershipVerifier is IMembershipVerifier {
    uint256 calls;
    mapping(uint256 => bool) validProof;

    function reset() public {
        calls = 0;
    }

    function pushValid(uint256 index) public {
        validProof[index] = true;
    }

    function verifyMembership(
        bytes memory root,
        bytes calldata proof,
        bytes memory prefix,
        bytes calldata path,
        bytes calldata value
    ) external returns (bool) {
        bool valid = validProof[calls];
        validProof[calls] = false;
        calls++;
        return valid;
    }

    function verifyNonMembership(
        bytes memory root,
        bytes calldata proof,
        bytes calldata prefix,
        bytes calldata path
    ) external returns (bool) {
        bool valid = validProof[calls];
        validProof[calls] = false;
        calls++;
        return valid;
    }
}

contract IBCConnectionHandlerTests is TestPlus {
    using BytesLib for bytes;
    using ConnectionCounterparty for ConnectionCounterparty.Data;

    string constant CLIENT_TYPE = "mock";

    bytes32 constant ARBITRARY_INITIAL_APP_HASH =
        hex"A8158610DD6858F3D26149CC0DB3339ABD580EA217DE0A151C9C451DED418E35";

    IBCHandler_Testable handler;
    ILightClient client;
    TestVerifier verifier;
    TestMembershipVerifier membershipVerifier;

    function setUp() public {
        handler = new IBCHandler_Testable();
        membershipVerifier = new TestMembershipVerifier();
        verifier = new TestVerifier();
        client = new CometblsClient(
            address(handler),
            verifier,
            membershipVerifier
        );
        handler.registerClient(CLIENT_TYPE, client);
    }

    function getValidHeader()
        internal
        pure
        returns (TendermintTypesSignedHeader.Data memory)
    {
        TendermintTypesHeader.Data memory header = TendermintTypesHeader.Data({
            version: TendermintVersionConsensus.Data({block: 11, app: 0}),
            chain_id: "union-devnet-1",
            height: 139,
            time: Timestamp.Data({secs: 1691496777, nanos: 793918988}),
            last_block_id: TendermintTypesBlockID.Data({
                hash: hex"80DF3A892BF2586E3B22201D2AC5A65EDAB66ECE7BB6F51077F3B50CCE7526E1",
                part_set_header: TendermintTypesPartSetHeader.Data({
                    total: 1,
                    hash: hex"0468D541CAD891D571E2AD1DD9F43480993BDF18A1016F4C956555A417EFE681"
                })
            }),
            last_commit_hash: hex"DA6FCBD48131808D58B54E8B44737AB2B6F3A3DD1AFF946D0F6CEFD25306FD48",
            data_hash: hex"E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855",
            validators_hash: hex"F09E25471B41514B2F8B08B5F4C9093C5D6ED134E107FF491CED2374B947DF60",
            next_validators_hash: hex"F09E25471B41514B2F8B08B5F4C9093C5D6ED134E107FF491CED2374B947DF60",
            consensus_hash: hex"048091BC7DDC283F77BFBF91D73C44DA58C3DF8A9CBC867405D8B7F3DAADA22F",
            app_hash: hex"983EF85676937CEC783601B5B50865733A72C3DF88E4CC0B3F11C108C9688459",
            last_results_hash: hex"357B78587B9CD4469F1F63C29B96EAC1D7F643520B97D396B20A20505122AA01",
            evidence_hash: hex"E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855",
            proposer_address: hex"4CE57693C82B50F830731DAB14FA759327762456"
        });
        return
            TendermintTypesSignedHeader.Data({
                header: header,
                // NOTE: validators are signing the block root which is computed
                // from the above TendermintTypesHeader field. Relayers can tamper
                // the commit but the client ensure that the block_id.hash matches
                // hash(header). Signatures are not required as the ZKP is a proof
                // that the validators signed the correct hash.
                commit: TendermintTypesCommit.Data({
                    height: header.height,
                    round: 0,
                    block_id: TendermintTypesBlockID.Data({
                        hash: hex"90548CD1E2BA8603261FE6400ADFD97EA48150CBD5B24B9828B2542BAB9E27EC",
                        part_set_header: TendermintTypesPartSetHeader.Data({
                            total: 1,
                            hash: hex"153E8B1F5B189A140FE5DA85DAB72FBD4A1DFA7E69C6FE5CE1FD66F0CCB5F6A1"
                        })
                    }),
                    signatures: new TendermintTypesCommitSig.Data[](0)
                })
            });
    }

    function assumeValidProofHeight(uint64 proofHeight) internal {
        vm.assume(
            0 < proofHeight &&
                proofHeight < uint64(getValidHeader().header.height)
        );
    }

    function createClient(uint64 proofHeight) internal returns (string memory) {
        assumeValidProofHeight(proofHeight);
        TendermintTypesSignedHeader.Data memory signedHeader = getValidHeader();
        IBCMsgs.MsgCreateClient memory m = Cometbls.createClient(
            CLIENT_TYPE,
            signedHeader.header.chain_id,
            proofHeight,
            ARBITRARY_INITIAL_APP_HASH,
            signedHeader.header.validators_hash.toBytes32(0),
            uint64(signedHeader.header.time.secs - 10)
        );
        return handler.createClient(m);
    }

    function preInitOk() public {
        membershipVerifier.reset();
    }

    function preAckValidProofs() public {
        membershipVerifier.reset();
        membershipVerifier.pushValid(0);
        membershipVerifier.pushValid(1);
    }

    function preAckInvalidConnectionStateProof() public {
        membershipVerifier.reset();
        vm.expectRevert(IBCConnectionLib.ErrInvalidProof.selector);
    }

    function preAckInvalidClientStateProof() public {
        membershipVerifier.reset();
        membershipVerifier.pushValid(0);
        vm.expectRevert(IBCConnectionLib.ErrInvalidProof.selector);
    }

    function preTryValidProofs() public {
        membershipVerifier.reset();
        membershipVerifier.pushValid(0);
        membershipVerifier.pushValid(1);
    }

    function preTryInvalidConnectionStateProof() public {
        membershipVerifier.reset();
        vm.expectRevert(IBCConnectionLib.ErrInvalidProof.selector);
    }

    function preTryInvalidClientStateProof() public {
        membershipVerifier.reset();
        membershipVerifier.pushValid(0);
        vm.expectRevert(IBCConnectionLib.ErrInvalidProof.selector);
    }

    function preConfirmValidProofs() public {
        membershipVerifier.reset();
        membershipVerifier.pushValid(0);
    }

    function preConfirmInvalidConnectionState() public {
        vm.expectRevert(IBCConnectionLib.ErrInvalidProof.selector);
    }

    function test_handshake_init_ack_ok(uint64 proofHeight) public {
        string memory clientId = createClient(proofHeight);

        IBCMsgs.MsgConnectionOpenInit memory msg_init = MsgMocks
            .connectionOpenInit(clientId);
        preInitOk();
        string memory connId = handler.connectionOpenInit(msg_init);

        (ConnectionEnd.Data memory connection, ) = handler.getConnection(
            connId
        );
        assertEq(connection.client_id, clientId, "clientId mismatch");
        assertEq(
            connection.delay_period,
            msg_init.delayPeriod,
            "delayPeriod mismatch"
        );
        assertEq(
            connection.counterparty.encode(),
            msg_init.counterparty.encode(),
            "counterparty mismatch"
        );
        assert(connection.state == ConnectionEnums.State.STATE_INIT);
        assertEq(connection.versions.length, 1);
        assertEq(connection.versions[0].features.length, 2);
        assertEq(connection.versions[0].identifier, "1");
        assertEq(connection.versions[0].features[0], "ORDER_ORDERED");
        assertEq(connection.versions[0].features[1], "ORDER_UNORDERED");

        assertEq(
            handler.commitments(IBCCommitment.connectionCommitmentKey(connId)),
            keccak256(IbcCoreConnectionV1ConnectionEnd.encode(connection))
        );

        IBCMsgs.MsgConnectionOpenAck memory msg_ack = MsgMocks
            .connectionOpenAck(clientId, connId, proofHeight);
        preAckValidProofs();
        handler.connectionOpenAck(msg_ack);

        ConnectionCounterparty.Data memory expectedCounterparty = msg_init
            .counterparty;
        expectedCounterparty.connection_id = msg_ack.counterpartyConnectionID;

        (connection, ) = handler.getConnection(connId);
        assertEq(connection.client_id, clientId, "clientId mismatch");
        assertEq(
            connection.delay_period,
            msg_init.delayPeriod,
            "delayPeriod mismatch"
        );
        assertEq(
            connection.counterparty.encode(),
            expectedCounterparty.encode(),
            "counterparty mismatch"
        );
        assert(connection.state == ConnectionEnums.State.STATE_OPEN);
        assertEq(connection.versions.length, 1);
        assertEq(connection.versions[0].features.length, 2);
        assertEq(connection.versions[0].identifier, "1");
        assertEq(connection.versions[0].features[0], "ORDER_ORDERED");
        assertEq(connection.versions[0].features[1], "ORDER_UNORDERED");

        assertEq(
            handler.commitments(IBCCommitment.connectionCommitmentKey(connId)),
            keccak256(IbcCoreConnectionV1ConnectionEnd.encode(connection))
        );
    }

    function test_handshake_ack_noInit(uint64 proofHeight) public {
        string memory clientId = createClient(proofHeight);

        IBCMsgs.MsgConnectionOpenAck memory msg_ack = MsgMocks
            .connectionOpenAck(clientId, "", proofHeight);
        preAckValidProofs();
        vm.expectRevert(IBCConnectionLib.ErrInvalidConnectionState.selector);
        handler.connectionOpenAck(msg_ack);
    }

    function test_handshake_init_ack_unsupportedVersion(
        uint64 proofHeight
    ) public {
        string memory clientId = createClient(proofHeight);

        IBCMsgs.MsgConnectionOpenInit memory msg_init = MsgMocks
            .connectionOpenInit(clientId);
        preInitOk();
        string memory connId = handler.connectionOpenInit(msg_init);

        (ConnectionEnd.Data memory connection, ) = handler.getConnection(
            connId
        );
        assertEq(connection.client_id, clientId, "clientId mismatch");
        assertEq(
            connection.delay_period,
            msg_init.delayPeriod,
            "delayPeriod mismatch"
        );
        assertEq(
            connection.counterparty.encode(),
            msg_init.counterparty.encode(),
            "counterparty mismatch"
        );
        assert(connection.state == ConnectionEnums.State.STATE_INIT);
        assertEq(connection.versions.length, 1);
        assertEq(connection.versions[0].features.length, 2);
        assertEq(connection.versions[0].identifier, "1");
        assertEq(connection.versions[0].features[0], "ORDER_ORDERED");
        assertEq(connection.versions[0].features[1], "ORDER_UNORDERED");

        assertEq(
            handler.commitments(IBCCommitment.connectionCommitmentKey(connId)),
            keccak256(IbcCoreConnectionV1ConnectionEnd.encode(connection))
        );

        IBCMsgs.MsgConnectionOpenAck memory msg_ack = MsgMocks
            .connectionOpenAck(clientId, connId, proofHeight);
        msg_ack.version.identifier = "2";
        preAckValidProofs();
        vm.expectRevert(IBCConnectionLib.ErrUnsupportedVersion.selector);
        handler.connectionOpenAck(msg_ack);
    }

    function test_handshake_init_ack_invalidConnectionStateProof(
        uint64 proofHeight
    ) public {
        string memory clientId = createClient(proofHeight);

        IBCMsgs.MsgConnectionOpenInit memory msg_init = MsgMocks
            .connectionOpenInit(clientId);
        preInitOk();
        string memory connId = handler.connectionOpenInit(msg_init);

        (ConnectionEnd.Data memory connection, ) = handler.getConnection(
            connId
        );
        assertEq(connection.client_id, clientId, "clientId mismatch");
        assertEq(
            connection.delay_period,
            msg_init.delayPeriod,
            "delayPeriod mismatch"
        );
        assertEq(
            connection.counterparty.encode(),
            msg_init.counterparty.encode(),
            "counterparty mismatch"
        );
        assert(connection.state == ConnectionEnums.State.STATE_INIT);
        assertEq(connection.versions.length, 1);
        assertEq(connection.versions[0].features.length, 2);
        assertEq(connection.versions[0].identifier, "1");
        assertEq(connection.versions[0].features[0], "ORDER_ORDERED");
        assertEq(connection.versions[0].features[1], "ORDER_UNORDERED");

        assertEq(
            handler.commitments(IBCCommitment.connectionCommitmentKey(connId)),
            keccak256(IbcCoreConnectionV1ConnectionEnd.encode(connection))
        );

        IBCMsgs.MsgConnectionOpenAck memory msg_ack = MsgMocks
            .connectionOpenAck(clientId, connId, proofHeight);
        preAckInvalidConnectionStateProof();
        handler.connectionOpenAck(msg_ack);
    }

    function test_handshake_init_ack_invalidClientStateProof(
        uint64 proofHeight
    ) public {
        string memory clientId = createClient(proofHeight);

        IBCMsgs.MsgConnectionOpenInit memory msg_init = MsgMocks
            .connectionOpenInit(clientId);
        preInitOk();
        string memory connId = handler.connectionOpenInit(msg_init);

        (ConnectionEnd.Data memory connection, ) = handler.getConnection(
            connId
        );
        assertEq(connection.client_id, clientId, "clientId mismatch");
        assertEq(
            connection.delay_period,
            msg_init.delayPeriod,
            "delayPeriod mismatch"
        );
        assertEq(
            connection.counterparty.encode(),
            msg_init.counterparty.encode(),
            "counterparty mismatch"
        );
        assert(connection.state == ConnectionEnums.State.STATE_INIT);
        assertEq(connection.versions.length, 1);
        assertEq(connection.versions[0].features.length, 2);
        assertEq(connection.versions[0].identifier, "1");
        assertEq(connection.versions[0].features[0], "ORDER_ORDERED");
        assertEq(connection.versions[0].features[1], "ORDER_UNORDERED");

        assertEq(
            handler.commitments(IBCCommitment.connectionCommitmentKey(connId)),
            keccak256(IbcCoreConnectionV1ConnectionEnd.encode(connection))
        );

        IBCMsgs.MsgConnectionOpenAck memory msg_ack = MsgMocks
            .connectionOpenAck(clientId, connId, proofHeight);
        preAckInvalidClientStateProof();
        handler.connectionOpenAck(msg_ack);
    }

    function test_handshake_try_confirm_ok(uint64 proofHeight) public {
        string memory clientId = createClient(proofHeight);

        IBCMsgs.MsgConnectionOpenTry memory msg_try = MsgMocks
            .connectionOpenTry(clientId, proofHeight);
        preTryValidProofs();
        string memory connId = handler.connectionOpenTry(msg_try);

        (ConnectionEnd.Data memory connection, ) = handler.getConnection(
            connId
        );
        assertEq(connection.client_id, clientId, "clientId mismatch");
        assertEq(
            connection.delay_period,
            msg_try.delayPeriod,
            "delayPeriod mismatch"
        );
        assertEq(
            connection.counterparty.encode(),
            msg_try.counterparty.encode(),
            "counterparty mismatch"
        );
        assert(connection.state == ConnectionEnums.State.STATE_TRYOPEN);
        assertEq(connection.versions.length, 1);
        assertEq(connection.versions[0].features.length, 2);
        assertEq(connection.versions[0].identifier, "1");
        assertEq(connection.versions[0].features[0], "ORDER_ORDERED");
        assertEq(connection.versions[0].features[1], "ORDER_UNORDERED");

        assertEq(
            handler.commitments(IBCCommitment.connectionCommitmentKey(connId)),
            keccak256(IbcCoreConnectionV1ConnectionEnd.encode(connection))
        );

        IBCMsgs.MsgConnectionOpenConfirm memory msg_confirm = MsgMocks
            .connectionOpenConfirm(clientId, connId, proofHeight);
        preConfirmValidProofs();
        handler.connectionOpenConfirm(msg_confirm);

        (connection, ) = handler.getConnection(connId);
        assertEq(connection.client_id, clientId, "clientId mismatch");
        assertEq(
            connection.delay_period,
            msg_try.delayPeriod,
            "delayPeriod mismatch"
        );
        assertEq(
            connection.counterparty.encode(),
            msg_try.counterparty.encode(),
            "counterparty mismatch"
        );
        assert(connection.state == ConnectionEnums.State.STATE_OPEN);
        assertEq(connection.versions.length, 1);
        assertEq(connection.versions[0].features.length, 2);
        assertEq(connection.versions[0].identifier, "1");
        assertEq(connection.versions[0].features[0], "ORDER_ORDERED");
        assertEq(connection.versions[0].features[1], "ORDER_UNORDERED");

        assertEq(
            handler.commitments(IBCCommitment.connectionCommitmentKey(connId)),
            keccak256(IbcCoreConnectionV1ConnectionEnd.encode(connection))
        );
    }

    function test_handshake_try_unsupportedVersion(uint64 proofHeight) public {
        string memory clientId = createClient(proofHeight);

        IBCMsgs.MsgConnectionOpenTry memory msg_try = MsgMocks
            .connectionOpenTry(clientId, proofHeight);
        msg_try.counterpartyVersions[0].identifier = "4";
        preTryValidProofs();
        vm.expectRevert(IBCConnectionLib.ErrUnsupportedVersion.selector);
        handler.connectionOpenTry(msg_try);
    }

    function test_handshake_try_invalidConnectionStateProof(
        uint64 proofHeight
    ) public {
        TendermintTypesSignedHeader.Data memory signedHeader = getValidHeader();
        vm.assume(
            0 < proofHeight && proofHeight < uint64(signedHeader.header.height)
        );

        IBCMsgs.MsgCreateClient memory m = Cometbls.createClient(
            CLIENT_TYPE,
            signedHeader.header.chain_id,
            proofHeight,
            ARBITRARY_INITIAL_APP_HASH,
            signedHeader.header.validators_hash.toBytes32(0),
            uint64(signedHeader.header.time.secs - 10)
        );
        string memory clientId = handler.createClient(m);

        IBCMsgs.MsgConnectionOpenTry memory msg_try = MsgMocks
            .connectionOpenTry(clientId, proofHeight);
        preTryInvalidConnectionStateProof();
        handler.connectionOpenTry(msg_try);
    }

    function test_handshake_try_invalidClientStateProof(
        uint64 proofHeight
    ) public {
        string memory clientId = createClient(proofHeight);

        IBCMsgs.MsgConnectionOpenTry memory msg_try = MsgMocks
            .connectionOpenTry(clientId, proofHeight);
        preTryInvalidClientStateProof();
        handler.connectionOpenTry(msg_try);
    }

    function test_handshake_try_confirm_invalidClientStateProof(
        uint64 proofHeight
    ) public {
        string memory clientId = createClient(proofHeight);

        IBCMsgs.MsgConnectionOpenTry memory msg_try = MsgMocks
            .connectionOpenTry(clientId, proofHeight);
        preTryValidProofs();
        string memory connId = handler.connectionOpenTry(msg_try);

        (ConnectionEnd.Data memory connection, ) = handler.getConnection(
            connId
        );
        assertEq(connection.client_id, clientId, "clientId mismatch");
        assertEq(
            connection.delay_period,
            msg_try.delayPeriod,
            "delayPeriod mismatch"
        );
        assertEq(
            connection.counterparty.encode(),
            msg_try.counterparty.encode(),
            "counterparty mismatch"
        );
        assert(connection.state == ConnectionEnums.State.STATE_TRYOPEN);
        assertEq(connection.versions.length, 1);
        assertEq(connection.versions[0].features.length, 2);
        assertEq(connection.versions[0].identifier, "1");
        assertEq(connection.versions[0].features[0], "ORDER_ORDERED");
        assertEq(connection.versions[0].features[1], "ORDER_UNORDERED");

        assertEq(
            handler.commitments(IBCCommitment.connectionCommitmentKey(connId)),
            keccak256(IbcCoreConnectionV1ConnectionEnd.encode(connection))
        );

        IBCMsgs.MsgConnectionOpenConfirm memory msg_confirm = MsgMocks
            .connectionOpenConfirm(clientId, connId, proofHeight);
        preConfirmInvalidConnectionState();
        handler.connectionOpenConfirm(msg_confirm);
    }

    function test_handshake_confirm_notTryOpen(uint64 proofHeight) public {
        string memory clientId = createClient(proofHeight);

        IBCMsgs.MsgConnectionOpenConfirm memory msg_confirm = MsgMocks
            .connectionOpenConfirm(clientId, "", proofHeight);
        preConfirmValidProofs();
        vm.expectRevert(IBCConnectionLib.ErrInvalidConnectionState.selector);
        handler.connectionOpenConfirm(msg_confirm);
    }

    function test_handshake_init_uniqueId() public {
        IBCMsgs.MsgConnectionOpenInit memory m = MsgMocks.connectionOpenInit(
            "client-1"
        );
        string memory id = handler.connectionOpenInit(m);
        string memory id2 = handler.connectionOpenInit(m);
        assertStrNotEq(id, id2);
    }
}
