import { writeCursor } from "./cursor.js";
import { insertTokenData, insertTransferData, updateCurrentBlockNumber, getTokenData, updateTokenData } from "./clickhouseWrapper.js";
/*
    Handle BlockScopedData messages.
    You will receive this message whenever a new block is created in the blockchain.
*/
const handleBlockScopedDataMessage = async (response, registry, substreamPackage) => {
    const output = response.output?.mapOutput;
    const cursor = response.cursor;

    if (output !== undefined) {
        const message = output.unpack(registry);
        if (message === undefined) {
            throw new Error(`Failed to unpack output of type ${output.typeUrl}`);
        }
        const outputAsJson = response.output.toJson({typeRegistry: registry});
        const blockValueInHash = response.clock.number;
        const blockNumber =  convertBigIntToNumber(blockValueInHash);
       
        if(outputAsJson.name === "map_filter_transactions" && Array.isArray(outputAsJson.mapOutput.transactions)) {
            const newTokens = [];
            for (const tokenObject of outputAsJson?.mapOutput?.transactions) {
                if (tokenObject.status === "0") {
                    // Collect new tokens (status 0) for bulk insert
                    newTokens.push({
                        id: tokenObject.id,
                        symbol: tokenObject.symbol,
                        name: tokenObject.name,
                        created_at: tokenObject.createdAt,
                        dev_wallet: tokenObject.devWallet,
                        total_supply: tokenObject.totalSupply,
                        launched_tx_hash: tokenObject.txHash,
                        created_block_number: tokenObject.createdBlockNumber,
                        status: tokenObject.status
                    });
                } else if (tokenObject.status === "1") {
                    // For graduated tokens (status 1)
                    // First load existing token data from Clickhouse
                    const existingToken = await getTokenData(tokenObject.id);
                    if (existingToken) {
                        // Update the existing token with graduation information
                        updateTokenData({
                            id: tokenObject.id,
                            graduated_tx_hash: tokenObject.txHash,
                            graduated_block_number: tokenObject.createdBlockNumber,
                            graduated_at: tokenObject.createdAt,
                            status: tokenObject.status
                        });
                    }
                }
            }
            // Bulk insert all new tokens at once
            if (newTokens.length > 0) {
                insertTokenData(newTokens);
            }
        }
        if(outputAsJson.name === "map_solana_token_events" && Array.isArray(outputAsJson.mapOutput.transfers)) {
            const transfers = [];
            let transferIndex = 1;
            for (const transferObject of outputAsJson?.mapOutput?.transfers) {
                // Collect transfers for bulk insert
                transfers.push({
                    id: transferObject.id,
                    tx_hash: transferObject.txHash, 
                    timestamp: transferObject.timestamp,
                    from: transferObject.from,
                    to: transferObject.to,
                    amount: transferObject.amount,
                    token_address: transferObject.tokenAddress,
                    block_number: transferObject.blockNumber,
                    transfer_index: transferIndex++,
                });
            }
            // Bulk insert all transfers at once
            if (transfers.length > 0) {
                insertTransferData(transfers);
            }
        }
        await updateCurrentBlockNumber(parseInt(blockNumber));
        await writeCursor(cursor, substreamPackage, blockNumber);
       
    }
}

/*
    Handle BlockUndoSignal messages.
    You will receive this message after a fork has happened in the blockchain.

    Because of the fork, you have probably read incorrect blocks in the "handleBlockScopedDataMessage" function,
    so you must rewind back to the last valid block.
*/
const handleBlockUndoSignalMessage = async (response, substreamPackage) => {
    const lastValidBlock = response.lastValidBlock;
    const lastValidCursor = response.lastValidCursor;
    
    /* The blockchain you are streaming from undo 1 or more blocks and you must now handle that case.
       The field `response.message.<last_valid_block>` contains the last valid block, you must undo whatever
       has been done prior that (so for data where `block_number > last_valid_block`). Once undo, you must also
       write the `response.message.<last_valid_cursor>`. In this example, we just print the undo signal and write the cursor.
    */
    // console.log(`Blockchain undo 1 or more blocks, returning to valid block #${lastValidBlock.num} (${lastValidBlock.id})`);

    // Update the current block number in the database
  await updateCurrentBlockNumber(parseInt(lastValidBlock.num));
    await writeCursor(lastValidCursor, substreamPackage, lastValidBlock.num);
    
}

export const handleResponseMessage = async (message, registry, substreamPackage) => {
    switch(message.case) {
        case "blockScopedData":
            handleBlockScopedDataMessage(message.value, registry, substreamPackage);
            break;

        case "blockUndoSignal":
            handleBlockUndoSignalMessage(message.value, substreamPackage);
            break;
        case "progress":
            handleProgressMessage(message.value)
    }
}

export const handleProgressMessage = progress => {
    // console.log(`Progress: ${objectToJsonString(progress)}`)
}

function convertBigIntToNumber(bigIntValue) {
    return Number(bigIntValue);
}
