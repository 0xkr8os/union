pragma solidity ^0.8.23;

import "../../proto/ibc/core/connection/v1/connection.sol";
import "../../proto/ibc/core/channel/v1/channel.sol";
import "../02-client/ILightClient.sol";

abstract contract IBCStore {
    // Commitments
    // keccak256(IBC-compatible-store-path) => keccak256(IBC-compatible-commitment)
    mapping(bytes32 => bytes32) public commitments;

    // Store
    mapping(string => address) public clientRegistry;
    mapping(string => string) public clientTypes;
    mapping(string => address) public clientImpls;
    mapping(string => IbcCoreConnectionV1ConnectionEnd.Data) public connections;
    mapping(string => mapping(string => IbcCoreChannelV1Channel.Data))
        public channels;
    mapping(string => mapping(string => uint64)) public nextSequenceSends;
    mapping(string => mapping(string => uint64)) public nextSequenceRecvs;
    mapping(string => mapping(string => uint64)) public nextSequenceAcks;
    mapping(string => mapping(string => mapping(uint64 => uint8)))
        public packetReceipts;
    mapping(string => address) public capabilities;

    // Sequences for identifier
    uint64 public nextClientSequence;
    uint64 public nextConnectionSequence;
    uint64 public nextChannelSequence;

    string public constant COMMITMENT_PREFIX = "ibc";

    // Storage accessors
    function getClient(
        string memory clientId
    ) public view returns (ILightClient) {
        address clientImpl = clientImpls[clientId];
        require(clientImpl != address(0), "getClient: not found");
        return ILightClient(clientImpl);
    }
}
