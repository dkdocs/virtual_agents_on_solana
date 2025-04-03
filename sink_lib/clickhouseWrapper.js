import { createClient } from '@clickhouse/client';
import { logger } from './logger.js';

const clickhouse = createClient({
  host: process.env.CLICKHOUSE_HOST,
  username: process.env.CLICKHOUSE_USERNAME,
  password: process.env.CLICKHOUSE_PASSWORD,
  database: process.env.CLICKHOUSE_DATABASE,
});

/**
 * Inserts token data into ClickHouse.
 * @param {Object|Object[]} tokenData - Single token object or array of token objects.
 */
export const insertTokenData = async (tokenData) => {
  logger.debug(`Token data to insert: ${JSON.stringify(tokenData)}`);
    const values = Array.isArray(tokenData) ? tokenData : [tokenData];
    try {
        await clickhouse.insert({
            table: 'virtual_agents_on_solana.tokens',
            values,
            format: 'JSONEachRow',
        });

        logger.info(`✅ Inserted ${values.length} record(s) into ClickHouse.`);
    } catch (error) {
        logger.error(`❌ Error inserting data: ${error}`);
    }
};

/**
 * Inserts transfer data into ClickHouse.
 * @param {Object|Object[]} transferData - Single transfer object or array of token objects.
 */
export const insertTransferData = async(transferData) => {
    logger.debug(`Transfer data to insert: ${JSON.stringify(transferData)}`);
    const values = Array.isArray(transferData) ? transferData : [transferData];
    try {
        await clickhouse.insert({
            table: 'virtual_agents_on_solana.transfers',
            values,
            format: 'JSONEachRow',
        });

        logger.info(`✅ Inserted ${values.length} record(s) into ClickHouse.`);
    } catch (error) {
        logger.error(`❌ Error inserting data: ${error}`);
    }
}

/**
 * Fetches a comma-separated string of tokens from the tokens table.
 * @returns {Promise<string>} Comma-separated token string.
 */
export const getCommaSeparatedTokens = async () => {
    try {
        const resultSet = await clickhouse.query({
            query: 'SELECT groupArray(id) AS tokens FROM virtual_agents_on_solana.tokens',
            format: 'JSON',
        });

        const data = await resultSet.json();
        const tokens = data?.data?.[0]?.tokens || [];
        return tokens.join(',');
    } catch (error) {
        logger.error("❌ Error fetching tokens: " + error);
        return '';
    }
};

export const getTokenCursor = async () => {
    try {
        const resultSet = await clickhouse.query({
          query: 'SELECT cursor, blockNumber FROM virtual_agents_on_solana.cursors_token ORDER BY blockNumber ASC LIMIT 1',
          format: 'JSON',
        });

        const { data } = await resultSet.json();

        if (data && data.length > 0) {
          return data[0].cursor;
        } else {
          logger.info('No data found');
          return undefined;
        }
      } catch (error) {
        logger.error(`Error fetching first token cursor: ${error}`);
        throw error;
      }
  };

  export const getTransferCursor = async () => {
    try {
        const resultSet = await clickhouse.query({
          query: 'SELECT cursor, blockNumber FROM virtual_agents_on_solana.cursors_transfer ORDER BY blockNumber ASC LIMIT 1',
          format: 'JSON',
        });

        const { data } = await resultSet.json();

        if (data && data.length > 0) {
          return data[0].cursor;
        } else {
          logger.info('No data found');
          return undefined;
        }
      } catch (error) {
        logger.error('Error fetching first token cursor: ' + error);
        throw error;
      }
  };

  export const updateTokenCursor = async (newCursor, newBlockNumber) => {
    try {
        // Delete the first row directly without condition
        await clickhouse.command({
            query: 'TRUNCATE TABLE virtual_agents_on_solana.cursors_token'
          });

        // Insert new data
        await clickhouse.insert({
          table: 'virtual_agents_on_solana.cursors_token',
          values: [{ cursor: newCursor, blockNumber: newBlockNumber }],
          format: 'JSONEachRow',
        });

        logger.info('✅ First token cursor updated successfully');
      } catch (error) {
        logger.error(`❌ Error updating first token cursor: ${error}`);
        throw error;
      }
  };

  export const updateTransferCursor = async (newCursor, newBlockNumber) => {
    try {
        // Delete the first row directly without condition
        await clickhouse.command({
            query: 'TRUNCATE TABLE virtual_agents_on_solana.cursors_transfer'
          });

        // Insert new data
        await clickhouse.insert({
          table: 'virtual_agents_on_solana.cursors_transfer',
          values: [{ cursor: newCursor, blockNumber: newBlockNumber }],
          format: 'JSONEachRow',
        });

        logger.info('✅ First transfer cursor updated successfully');
      } catch (error) {
        logger.error('❌ Error updating first transfer cursor: ' + error);
        throw error;
      }
  };

export const getCurrentBlockNumber = async () => {
  try {
    const resultSet = await clickhouse.query({
      query: 'SELECT current_block_number FROM virtual_agents_on_solana.block_tracking WHERE id = \'main\'',
      format: 'JSON',
    });

    const { data } = await resultSet.json();
    if (data && data.length > 0) {
      return data[0].current_block_number;
    }
    return null;
  } catch (error) {
    logger.error('Error fetching current block number: ' + error);
    return null;
  }
};

export const updateCurrentBlockNumber = async (blockNumber) => {
  try {
    await clickhouse.insert({
      table: 'virtual_agents_on_solana.block_tracking',
      values: [{
        id: 'main',
        current_block_number: blockNumber
      }],
      format: 'JSONEachRow',
    });
    logger.debug('✅ Current block number updated successfully');
  } catch (error) {
    logger.error(`❌ Error updating current block number: ${error}`);
    throw error;
  }
};

/**
 * Fetches token data from ClickHouse by token ID.
 * @param {string} tokenId - The ID of the token to fetch.
 * @returns {Promise<Object|null>} The token data or null if not found.
 */
export const getTokenData = async (tokenId) => {
    try {
        const resultSet = await clickhouse.query({
            query: 'SELECT * FROM virtual_agents_on_solana.tokens WHERE id = {tokenId:String}',
            query_params: { tokenId },
            format: 'JSON',
        });

        const { data } = await resultSet.json();
        if (data && data.length > 0) {
            return data[0];
        }
        return null;
    } catch (error) {
        logger.error(`❌ Error fetching token data: ${error}`);
        return null;
    }
};

/**
 * Updates token data in ClickHouse with graduation information.
 * @param {Object} tokenData - The token data to update.
 */
export const updateTokenData = async (tokenData) => {
    try {
        await clickhouse.query({
            query: `
                ALTER TABLE virtual_agents_on_solana.tokens
                UPDATE 
                    graduated_tx_hash = {graduated_tx_hash:String},
                    graduated_block_number = {graduated_block_number:UInt64},
                    graduated_at = {graduated_at:UInt64},
                    status = {status:String}
                WHERE id = {id:String}
            `,
            query_params: {
                id: tokenData.id,
                graduated_tx_hash: tokenData.graduated_tx_hash,
                graduated_block_number: tokenData.graduated_block_number,
                graduated_at: tokenData.graduated_at,
                status: tokenData.status
            },
        });

        logger.info(`✅ Updated token data for ID: ${tokenData.id}`);
    } catch (error) {
        logger.error(`❌ Error updating token data: ${error}`);
        throw error;
    }
};
